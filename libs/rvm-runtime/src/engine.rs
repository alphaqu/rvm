use std::ffi::c_void;
use std::pin::Pin;
use std::sync::Arc;
use std::thread::{spawn, JoinHandle};
use std::{panic, thread};

use crossbeam::channel::{unbounded, Receiver, Sender};

use rvm_core::ObjectType;
use rvm_gc::GcSweeper;
use rvm_reader::ConstantPool;

use crate::value::AnyValue;
use crate::Vm;
use crate::{Method, MethodIdentifier};

pub trait Engine: Send + Sync {
	fn create_thread(&self, runtime: Vm, config: ThreadConfig) -> ThreadHandle;

	fn compile_method(
		&self,
		runtime: &Pin<&Vm>,
		method: &Method,
		cp: &Arc<ConstantPool>,
	) -> *const c_void;
}

pub struct Thread {
	pub config: Arc<ThreadConfig>,
	pub receiver: Receiver<ThreadCommand>,
}

pub struct ThreadHandle {
	data: Arc<ThreadConfig>,
	handle: JoinHandle<eyre::Result<Option<AnyValue>>>,
	sender: Sender<ThreadCommand>,
}

impl ThreadHandle {
	pub fn new(
		config: ThreadConfig,
		func: impl FnOnce(Thread) -> eyre::Result<Option<AnyValue>> + Send + 'static,
	) -> ThreadHandle {
		let data = Arc::new(config);
		let (sender, receiver) = unbounded();
		let data2 = data.clone();

		let handle = spawn(|| {
			func(Thread {
				config: data2,
				receiver,
			})
		});

		ThreadHandle {
			data,
			handle,
			sender,
		}
	}

	pub fn join(self) -> eyre::Result<Option<AnyValue>> {
		match self.handle.join() {
			Ok(value) => value,
			Err(err) => {
				panic::resume_unwind(err);
			}
		}
	}

	pub fn name(&self) -> &str {
		self.data.name.as_str()
	}

	pub fn run(&self, ty: ObjectType, method: MethodIdentifier, parameters: Vec<AnyValue>) {
		self.sender
			.send(ThreadCommand::Run {
				ty,
				method,
				parameters,
			})
			.unwrap();
	}
}

pub enum ThreadCommand {
	Run {
		ty: ObjectType,
		method: MethodIdentifier,
		parameters: Vec<AnyValue>,
	},
	Exit,
}

pub struct ThreadConfig {
	pub name: String,
}
