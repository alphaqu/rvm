use either::Either;
use std::sync::Arc;

use tracing::{debug, trace};

use rvm_core::{Kind, MethodDescriptor, ObjectType, Type};
use rvm_reader::JumpKind;
use rvm_runtime::engine::Thread;
use rvm_runtime::gc::{AllocationError, GcMarker, GcSweeper, RootProvider};
use rvm_runtime::{AnyValue, MethodBinding, MethodCode, MethodIdentifier, Reference, Runtime};

use crate::code::{CallType, Task};
use crate::thread::{ThreadFrame, ThreadStack};
use crate::value::StackValue;
use crate::{BenEngine, BenMethod};

pub struct Executor<'a> {
	pub thread: Thread,
	pub stack: &'a mut ThreadStack,
	pub engine: Arc<BenEngine>,
	pub runtime: Arc<Runtime>,
}

pub struct ExecutorFrame<'a> {
	frame: ThreadFrame<'a>,
	method: Arc<BenMethod>,
	cursor: usize,
}

enum CallReturn<'f> {
	Frame(ExecutorFrame<'f>),
	Result(Option<AnyValue>),
}

impl<'a> Executor<'a> {
	fn call<'f>(
		&mut self,
		ty: &ObjectType,
		method: &MethodIdentifier,
		call_ty: CallType,
		mut parameter_getter: impl FnMut() -> StackValue,
	) -> CallReturn<'f> {
		debug!("Creating frame for {ty:?} {method:?}");

		let desc = MethodDescriptor::parse(&method.descriptor).unwrap();
		let mut parameters = Vec::new();
		for v in &desc.parameters {
			let stack_value = parameter_getter();
			let kind = v.kind();
			let value = stack_value
				.convert(kind)
				.ok_or_else(|| format!("{} could not be converted to {}", stack_value, kind))
				.unwrap();
			parameters.push(value);
		}
		parameters.reverse();

		let mut instance: Option<StackValue> = None;
		let is_static = call_ty == CallType::Static;
		if !is_static {
			instance = Some(parameter_getter());
		}

		let class_id = if call_ty == CallType::Interface {
			let value1 = instance.unwrap();
			let reference = value1.to_ref();
			let class_object = reference.to_class().unwrap();
			class_object.class()
		} else {
			self.runtime.cl.resolve_class(&Type::Object(ty.clone()))
		};

		let (method_class, method_id) = self
			.engine
			.resolve_method(&self.runtime, class_id, &method)
			.expect("could not resolve method");

		let method = self
			.engine
			.compile_method(&self.runtime, method_class, method_id);

		match &*method {
			BenMethod::Java(java) => {
				let mut frame = self.stack.create(java.max_stack, java.max_locals);
				let mut i = if is_static { 0 } else { 1 };
				for value in parameters.into_iter() {
					let local_size = value.kind().local_size();
					frame.store(i, StackValue::from_any(value));
					i += local_size as u16;
				}

				if let Some(value) = instance {
					frame.store(0, value);
				}

				CallReturn::Frame(ExecutorFrame {
					frame,
					method,
					cursor: 0,
				})
			}
			BenMethod::Binding(binding) => CallReturn::Result(binding.call(&parameters)),
			BenMethod::Native(native, desc) => {
				let mut linker = self.runtime.linker.lock();
				CallReturn::Result(linker.get(native, |function| unsafe {
					trace!("Calling native function");
					MethodBinding::new(function, desc.clone()).call(&parameters)
				}))
			}
		}
	}

	pub fn execute(
		mut self,
		ty: &ObjectType,
		method: &MethodIdentifier,
		mut parameters: Vec<AnyValue>,
	) -> Option<AnyValue> {
		let call_return = self.call(ty, method, CallType::Static, || {
			StackValue::from_any(parameters.pop().expect("Not enough parameters"))
		});

		let mut frames = match call_return {
			CallReturn::Frame(frame) => {
				vec![frame]
			}
			CallReturn::Result(value) => {
				return value;
			}
		};

		let mut output: Option<Option<(StackValue, Kind)>> = None;

		let mut gc_attempts = 0;
		// We look at the last frame which is the currently running one.
		while let Some(exec_frame) = frames.last_mut() {
			let frame = &mut exec_frame.frame;

			let method = &exec_frame.method;
			let method = match &**method {
				BenMethod::Java(method) => method,
				_ => todo!(),
			};
			loop {
				GcSweeper::yield_gc(&mut self);

				let mut stack_values = Vec::new();
				let mut local_values = Vec::new();
				for i in 0..frame.stack_pos {
					stack_values.push(format!("{}", frame.get_stack_value(i)));
				}
				for i in 0..frame.local_size {
					local_values.push(format!("{}", frame.load(i)));
				}
				let task = &method.tasks[exec_frame.cursor];
				trace!(target: "exe", "s[{}] l[{}] {task}", stack_values.join(","), local_values.join(","));

				match task {
					Task::New(object) => {
						let id = self
							.runtime
							.cl
							.resolve_class(&Type::Object(object.class_name.clone()));

						let class = self.runtime.cl.get(id);
						let object = class.as_instance().unwrap();
						unsafe {
							let result = self.runtime.gc.lock().allocate_instance(id, object);

							match result {
								Ok(object) => {
									frame.push(StackValue::Reference(*object));
								}
								Err(AllocationError::OutOfHeap) => {
									gc_attempts += 1;
									if gc_attempts > 5 {
										panic!("No more memory");
									}
									Runtime::gc(self.runtime.clone());
									GcSweeper::wait_until_gc(&mut self);
									trace!("Forcing gc, and trying again.");
									// try this instruction again, if we fail 5 time, we blow up.
									continue;
								}
								value => {
									value.unwrap();
								}
							}
							gc_attempts = 0;
						}
					}
					Task::Call(task) => {
						// When we first call, the output will be None, it will push a frame onto the stack and start running that method.
						// When that method returns, it will set the output to Some(Option<Value>) and pop itself out of the stack.
						// We will come back here (because we never incremented the pointer) and see that our output is now Some.
						// We push that return value (if it exists) and continue running.
						match output.take() {
							None => {
								let call =
									self.call(&task.object, &task.method, task.ty, || frame.pop());
								match call {
									CallReturn::Frame(frame) => {
										frames.push(frame);
										break;
									}
									CallReturn::Result(value) => {
										if let Some(value) = value {
											frame.push(StackValue::from_any(value));
										}
									}
								}
							}
							Some(value) => {
								if let Some((value, _)) = value {
									frame.push(value);
								}
							}
						}
					}
					Task::Return(v) => {
						output = Some(method.returns.map(|kind| {
							let value = frame.pop();
							(value, kind)
						}));
						let frame = frames.pop().unwrap();
						self.stack.finish(frame.frame);
						break;
					}
					Task::Nop => {}
					Task::Const(v) => v.exec(frame),
					Task::Combine(v) => v.exec(frame),
					Task::Local(v) => v.exec(frame),
					Task::Jump(task) => {
						let condition = match task.kind {
							JumpKind::IF_ICMPEQ | JumpKind::IF_ACMPEQ => {
								let value2 = frame.pop();
								let value1 = frame.pop();
								value1 == value2
							}
							JumpKind::IF_ICMPNE | JumpKind::IF_ACMPNE => {
								let value2 = frame.pop();
								let value1 = frame.pop();
								value1 != value2
							}
							JumpKind::IF_ICMPLT => {
								let value2 = frame.pop().to_int();
								let value1 = frame.pop().to_int();
								value1 < value2
							}
							JumpKind::IF_ICMPGE => {
								let value2 = frame.pop().to_int();
								let value1 = frame.pop().to_int();
								value1 >= value2
							}
							JumpKind::IF_ICMPGT => {
								let value2 = frame.pop().to_int();
								let value1 = frame.pop().to_int();
								value1 > value2
							}
							JumpKind::IF_ICMPLE => {
								let value2 = frame.pop().to_int();
								let value1 = frame.pop().to_int();
								value1 <= value2
							}
							JumpKind::IFEQ => {
								let value = frame.pop().to_int();
								value == 0
							}
							JumpKind::IFNE => {
								let value = frame.pop().to_int();
								value != 0
							}
							JumpKind::IFLT => {
								let value = frame.pop().to_int();
								value < 0
							}
							JumpKind::IFGE => {
								let value = frame.pop().to_int();
								value >= 0
							}
							JumpKind::IFGT => {
								let value = frame.pop().to_int();
								value > 0
							}
							JumpKind::IFLE => {
								let value = frame.pop().to_int();
								value <= 0
							}
							JumpKind::IFNONNULL => {
								let value = frame.pop().to_ref();
								value != Reference::NULL
							}
							JumpKind::IFNULL => {
								let value = frame.pop().to_ref();
								value == Reference::NULL
							}
							JumpKind::GOTO => true,
						};

						if condition {
							exec_frame.cursor = exec_frame
								.cursor
								.checked_add_signed(task.offset as isize)
								.unwrap();
							continue;
						}
					}
					Task::SwitchTable(v) => {
						let offset = v.exec(frame);
						exec_frame.cursor = exec_frame
							.cursor
							.checked_add_signed(offset as isize)
							.unwrap();
						continue;
					}
					Task::Stack(task) => task.exec(frame),
					Task::Field(task) => task.exec(&self.runtime, frame),
					Task::Increment(task) => {
						let value = frame.load(task.local);
						frame.store(
							task.local,
							StackValue::Int(value.to_int() + task.increment as i32),
						);
					}
					Task::ArrayLength(v) => v.exec(frame),
					Task::ArrayLoad(v) => v.exec(frame),
					Task::ArrayStore(v) => v.exec(frame),
					Task::ArrayCreate(v) => v.exec(&self.runtime, frame),
					Task::ArrayCreateRef(v) => v.exec(&self.runtime, frame),
				};
				exec_frame.cursor += 1;
			}
		}

		match output.expect("Method did not return") {
			Some((value, kind)) => value.convert(kind),
			None => None,
		}
	}
}

impl<'a> RootProvider for Executor<'a> {
	fn mark_roots(&mut self, mut visitor: GcMarker) {
		self.stack.visit_frames_mut(|frame| {
			for i in 0..frame.stack_pos {
				if let StackValue::Reference(reference) = frame.get_stack_value(i) {
					visitor.mark(reference);
				}
			}

			for i in 0..frame.local_size {
				if let StackValue::Reference(reference) = frame.load(i) {
					visitor.mark(reference);
				}
			}
		});
	}

	fn remap_roots(&mut self, mut mapper: impl FnMut(Reference) -> Reference) {
		self.stack.visit_frames_mut(|frame| {
			for i in 0..frame.stack_pos {
				if let StackValue::Reference(reference) = frame.get_stack_value(i) {
					frame.set_stack_value(i, StackValue::Reference(mapper(reference)));
				}
			}

			for i in 0..frame.local_size {
				if let StackValue::Reference(reference) = frame.load(i) {
					frame.store(i, StackValue::Reference(reference));
				}
			}
		});
	}

	fn sweeper(&mut self) -> &mut GcSweeper {
		&mut self.thread.gc
	}
}
