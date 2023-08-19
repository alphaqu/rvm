use crate::code::Task;
use crate::method::CompiledMethod;
use crate::thread::{ThreadFrame, ThreadStack};
use crate::value::StackValue;
use crate::BenEngine;
use rvm_core::{Kind, MethodAccessFlags, ObjectType, Reference, Type};
use rvm_object::{DynValue, MethodIdentifier};
use rvm_reader::JumpKind;
use rvm_runtime::engine::Thread;
use rvm_runtime::gc::{AllocationError, GcMarker, GcSweeper, RootProvider};
use rvm_runtime::Runtime;
use std::sync::Arc;
use tracing::{debug, trace};

pub struct Executor<'a> {
	pub thread: Thread,
	pub stack: &'a mut ThreadStack,
	pub engine: Arc<BenEngine>,
	pub runtime: Arc<Runtime>,
}

pub struct ExecutorFrame<'a> {
	frame: ThreadFrame<'a>,
	method: Arc<CompiledMethod>,
	cursor: usize,
}

impl<'a> Executor<'a> {
	fn create_frame<'f>(
		&mut self,
		ty: &ObjectType,
		method: &MethodIdentifier,
		mut parameter_getter: impl FnMut() -> DynValue,
	) -> ExecutorFrame<'f> {
		debug!("Creating frame for {ty:?} {method:?}");
		let method = self
			.engine
			.get_compiled_method(&self.runtime, ty.clone(), method.clone());

		let is_static = method.flags.contains(MethodAccessFlags::STATIC);
		let mut frame = self.stack.create(method.max_stack, method.max_locals);

		for (i, ty) in method.parameters.iter().enumerate().rev() {
			let value = parameter_getter();
			frame.store(
				if is_static { i } else { i + 1 } as u16,
				StackValue::from_dyn(value),
			);
		}

		if !is_static {
			frame.store(0, StackValue::from_dyn(parameter_getter()));
		}

		ExecutorFrame {
			frame,
			method,
			cursor: 0,
		}
	}
	pub fn execute(
		mut self,
		ty: &ObjectType,
		method: &MethodIdentifier,
		mut parameters: Vec<DynValue>,
	) -> Option<DynValue> {
		let mut frames = vec![self.create_frame(ty, method, || {
			parameters.pop().expect("Not enough parameters")
		})];
		let mut output: Option<Option<(StackValue, Kind)>> = None;

		let mut gc_attempts = 0;
		// We look at the last frame which is the currently running one.
		while let Some(exec_frame) = frames.last_mut() {
			let method = &exec_frame.method;
			let frame = &mut exec_frame.frame;
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
							.class_loader
							.get_class_id(&Type::Object(object.class_name.clone()));

						let class = self.runtime.class_loader.get(id);
						let object = class.object().unwrap();
						unsafe {
							let result = self.runtime.gc.lock().allocate_object(id, object);

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
						}
					}
					Task::Call(task) => {
						// When we first call, the output will be None, it will push a frame onto the stack and start running that method.
						// When that method returns, it will set the output to Some(Option<Value>) and pop itself out of the stack.
						// We will come back here (because we never incremented the pointer) and see that our output is now Some.
						// We push that return value (if it exists) and continue running.
						match output.take() {
							None => {
								let executor_frame =
									self.create_frame(&task.object, &task.method, || {
										frame.pop().to_dyn()
									});
								frames.push(executor_frame);
								break;
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
				};

				gc_attempts = 0;
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
