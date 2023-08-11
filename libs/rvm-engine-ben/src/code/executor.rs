use crate::code::Task;
use crate::method::CompiledMethod;
use crate::thread::{ThreadFrame, ThreadStack};
use crate::value::StackValue;
use crate::BenEngine;
use rvm_core::{Kind, ObjectType, Op, StackKind};
use rvm_object::{DynValue, MethodIdentifier};
use rvm_reader::JumpKind;
use rvm_runtime::Runtime;
use std::sync::Arc;
use tracing::{debug, trace};

pub struct Executor<'a> {
	pub stack: &'a mut ThreadStack,
	pub engine: Arc<BenEngine>,
	pub runtime: &'a Runtime,
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
			.get_compiled_method(self.runtime, ty.clone(), method.clone());

		let mut frame = self.stack.create(method.max_stack, method.max_locals);
		for (i, ty) in method.parameters.iter().enumerate().rev() {
			let value = parameter_getter();
			frame.store_dyn(i as u16, StackValue::from_dyn(value));
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
		// We look at the last frame which is the currently running one.
		while let Some(exec_frame) = frames.last_mut() {
			let method = &exec_frame.method;
			let frame = &mut exec_frame.frame;
			loop {
				let task = &method.tasks[exec_frame.cursor];
				trace!("{task:?}");

				match task {
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
								value != 0
							}
							JumpKind::IFNULL => {
								let value = frame.pop().to_ref();
								value == 0
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
