use eyre::Context;
use rvm_core::Id;
use rvm_runtime::engine::{ThreadCommand, ThreadConfig, ThreadHandle};
use rvm_runtime::{Class, Method, Runtime};
use rvm_stack::StackUser;
pub use stack::{ThreadFrame, ThreadStack};
use std::sync::Arc;
use tracing::{debug, span, Level};

use crate::code::{Executor, ExecutorRunner};
use crate::value::StackValue;
use crate::BenEngine;

mod frame;
mod stack;

pub type BenOwnedFrame<'a, 'd> = rvm_stack::FrameGuard<'a, 'd, BenUser>;
pub type BenCallStack<'d> = rvm_stack::CallStack<'d, BenUser>;
pub type BenFrame<'a, 'd> = rvm_stack::Frame<'a, 'd, BenUser>;
pub type BenFrameMut<'a, 'd> = rvm_stack::FrameMut<'a, 'd, BenUser>;
pub type BenFrameTicket = rvm_stack::FrameTicket<BenUser>;

pub struct BenUser;

impl StackUser for BenUser {
	type StackEntry = StackValue;
	type FrameHeader = FrameHeader;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct FrameHeader {
	pub class_id: Id<Class>,
	pub method_id: Id<Method>,
	pub cursor: usize,
}

pub fn spawn(
	runtime: Runtime,
	config: ThreadConfig,
	size: usize,
	engine: Arc<BenEngine>,
) -> ThreadHandle {
	ThreadHandle::new(runtime.clone(), config, move |thread| {
		let config = thread.config.clone();
		let output =
			BenCallStack::new_on_stack(size, |call_stack| {
				let span = span!(Level::DEBUG, "vm-thread");
				let _enter = span.enter();

				loop {
					if let Ok(command) = thread.receiver.recv() {
						match command {
							ThreadCommand::Run {
								ty,
								method,
								parameters,
							} => {
								debug!("Running {ty:?} {method:?}");

								let executor = Executor {
									thread,
									call_stack,
									engine,
									runtime,
									runner: ExecutorRunner::new(),
								};

								let result =
									executor.execute(&ty, &method, parameters).wrap_err_with(
										|| format!("Running in thread \"{}\"", config.name),
									)?;
								return Ok(result);
							}
							ThreadCommand::Exit => {
								return Ok(None);
							}
						}
					}
				}
			});

		output
	})
}
