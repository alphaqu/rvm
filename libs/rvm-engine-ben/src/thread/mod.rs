use eyre::Context;
use std::sync::Arc;
use tracing::{debug, span, Level};

use rvm_runtime::engine::{ThreadCommand, ThreadConfig, ThreadHandle};
use rvm_runtime::Runtime;
pub use stack::{ThreadFrame, ThreadStack};

use crate::code::{Executor, ExecutorRunner};
use crate::BenEngine;

mod frame;
mod stack;

pub fn spawn(
	runtime: Runtime,
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
								runtime,
								runner: ExecutorRunner::new(),
							};

							out = executor
								.execute(&ty, &method, parameters)
								.wrap_err_with(|| format!("Running in thread \"{}\"", config.name))
								.unwrap();
							return;
						}
						ThreadCommand::Exit => {
							return;
						}
					}
				}
			}
		});

		out
	})
}
