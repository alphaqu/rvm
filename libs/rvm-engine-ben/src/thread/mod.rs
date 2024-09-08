use eyre::Context;
use rvm_core::Id;
use rvm_runtime::engine::{ThreadCommand, ThreadConfig, ThreadHandle};
use rvm_runtime::{CallType, Class, Method, ThreadContext, Vm};
use rvm_stack::StackUser;
pub use stack::{ThreadFrame, ThreadStack};
use std::panic;
use std::panic::AssertUnwindSafe;
use std::sync::Arc;
use tracing::{debug, span, Level};

use crate::code::Executor;
use crate::value::StackValue;
use crate::BenEngine;

mod frame;
mod stack;

pub type BenOwnedFrame<'a> = rvm_stack::FrameGuard<'a, BenUser>;
pub type BenCallStack = rvm_stack::CallStack<BenUser>;
pub type BenFrame<'a> = rvm_stack::Frame<'a, BenUser>;
pub type BenFrameMut<'a> = rvm_stack::FrameMut<'a, BenUser>;
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
	runtime: Vm,
	config: ThreadConfig,
	size: usize,
	engine: Arc<BenEngine>,
) -> ThreadHandle {
	ThreadHandle::new(config, move |thread| {
		let config = thread.config.clone();
		let stack = BenCallStack::new_on_heap(size);

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

						let sweeper = runtime.gc.new_sweeper();
						let mut executor = Executor {
							thread,
							call_stack: stack,
							engine,
							vm: runtime,
							java_scopes: vec![],
							sweeper,
							frozen_references: vec![],
						};

						let output = panic::catch_unwind(AssertUnwindSafe(|| {
							executor
								.run(CallType::Static, &ty, &method, parameters)
								.wrap_err_with(|| format!("Running in thread \"{}\"", config.name))
						}));

						executor.vm.gc.remove_sweeper(executor.sweeper);

						match output {
							Ok(value) => return value,
							Err(unwind) => {
								panic::resume_unwind(unwind);
							}
						}
					}
					ThreadCommand::Exit => {
						return Ok(None);
					}
				}
			}
		}
	})
}
