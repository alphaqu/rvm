use crossbeam::channel::Receiver;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use rvm_core::ObjectType;
use rvm_object::MethodIdentifier;
use tracing::{debug, span, Level};

use rvm_reader::JumpKind;
use rvm_runtime::engine::{ThreadCommand, ThreadConfig, ThreadHandle};
use rvm_runtime::Runtime;
pub use stack::{ThreadFrame, ThreadStack};

use crate::code::{Executor, Task};
use crate::method::CompiledMethod;
use crate::value::StackValue;
use crate::BenEngine;

mod frame;
mod stack;

pub fn spawn(
	runtime: Arc<Runtime>,
	config: ThreadConfig,
	size: usize,
	engine: Arc<BenEngine>,
) -> ThreadHandle {
	ThreadHandle::new(runtime.clone(), config, move |thread| {
		let mut out = None;

		let config = thread.config.clone();
		ThreadStack::new(size, |stack| {
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
								stack,
								engine,
								runtime: runtime,
							};

							out = executor.execute(&ty, &method, parameters);
							return;
						}
						ThreadCommand::Exit => {
							return;
						}
					}
				}
			}

			debug!("{:?}: Finished", config.name);
		});

		out
	})
}
