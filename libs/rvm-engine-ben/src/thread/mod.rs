use std::sync::Arc;

use rvm_core::ObjectType;
use rvm_object::MethodIdentifier;
use tracing::{debug, span, Level};

use rvm_reader::JumpKind;
use rvm_runtime::engine::{ThreadCommand, ThreadConfig, ThreadHandle};
use rvm_runtime::Runtime;
pub use stack::{ThreadFrame, ThreadStack};

use crate::code::Task;
use crate::method::CompiledMethod;
use crate::value::StackValue;
use crate::BenEngine;

mod frame;
mod stack;

pub struct Thread<'r> {
	config: Arc<ThreadConfig>,
	engine: Arc<BenEngine>,
	runtime: &'r Runtime,
}

impl<'r> Thread<'r> {
	pub fn spawn(
		runtime: Arc<Runtime>,
		config: ThreadConfig,
		size: usize,
		engine: Arc<BenEngine>,
	) -> ThreadHandle {
		ThreadHandle::new(config, move |config, receiver| {
			let mut out = None;

			ThreadStack::new(size, |stack| {
				let span = span!(Level::DEBUG, "vm-thread");
				let _enter = span.enter();

				let mut thread = Thread {
					config,
					engine,
					runtime: &runtime,
				};

				loop {
					if let Ok(command) = receiver.recv() {
						match command {
							ThreadCommand::Run {
								ty: ty,
								method: method,
								parameters: value,
							} => {
								debug!("Running {ty:?} {method:?}");
								let method =
									thread.engine.get_compiled_method(&runtime, ty, method);
								let option = Self::run_method(thread.stack, &*method);
								debug!("Thread returned {option:?}");

								out = match method.returns {
									Some(kind) => option.unwrap().convert(kind),
									None => None,
								};
								return;
							}
							ThreadCommand::Exit => {
								return;
							}
						}
					}
				}

				debug!("{:?}: Finished", thread.config.name);
			});

			out
		})
	}

	pub fn run(&mut self, ty: &ObjectType, method: &MethodIdentifier, stack: &mut ThreadStack) {
		debug!("Running {ty:?} {method:?}");
		let method = self
			.engine
			.get_compiled_method(&self.runtime, ty.clone(), method.clone());

		stack.scope(method.max_stack, method.max_locals, |stack, mut frame| {
			let mut cursor = 0;
			loop {
				let task = &method.tasks[cursor];
				match task {
					Task::Nop => {}
					Task::Const(v) => v.exec(&mut frame),
					Task::Combine(v) => v.exec(&mut frame),
					Task::Local(v) => v.exec(&mut frame),
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
							cursor = cursor.checked_add_signed(task.offset as isize).unwrap();
							continue;
						}
					}
					Task::Return(v) => {
						let value = v.kind.map(|kind| {
							let value = frame.pop();
							if value.kind() != kind {
								panic!(
									"Return mismatch, expected {:?} but got {:?}",
									kind,
									value.kind()
								);
							}
							value
						});
						stack.finish(frame);
						return value;
					}
				};

				cursor += 1;
			}
		});

		let option = Self::run_method(stack, &*method);
		debug!("Thread returned {option:?}");

		out = match method.returns {
			Some(kind) => option.unwrap().convert(kind),
			None => None,
		};
	}

	pub fn run_method(stack: &mut ThreadStack, method: &CompiledMethod) -> Option<StackValue> {
		let mut frame = stack.create(method.max_stack, method.max_locals);
		let mut cursor = 0;

		loop {
			let task = &method.tasks[cursor];
			match task {
				Task::Nop => {}
				Task::Const(v) => v.exec(&mut frame),
				Task::Combine(v) => v.exec(&mut frame),
				Task::Local(v) => v.exec(&mut frame),
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
						cursor = cursor.checked_add_signed(task.offset as isize).unwrap();
						continue;
					}
				}
				Task::Return(v) => {
					let value = v.kind.map(|kind| {
						let value = frame.pop();
						if value.kind() != kind {
							panic!(
								"Return mismatch, expected {:?} but got {:?}",
								kind,
								value.kind()
							);
						}
						value
					});
					stack.finish(frame);
					return value;
				}
			};

			cursor += 1;
		}

		panic!("Unexpected end")
	}
}
