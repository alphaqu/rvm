use crate::Runtime;
use crossbeam::channel::{unbounded, Receiver, Sender};
use rvm_core::ObjectType;
use rvm_object::{DynValue, MethodData, MethodIdentifier};
use rvm_reader::ConstantPool;
use std::any::Any;
use std::ffi::c_void;
use std::pin::Pin;
use std::sync::Arc;
use std::thread;
use std::thread::{scope, spawn, JoinHandle};

pub trait Engine: Send + Sync {
	fn create_thread(&self, runtime: Arc<Runtime>, config: ThreadConfig) -> ThreadHandle;

	fn compile_method(
		&self,
		runtime: &Pin<&Runtime>,
		method: &MethodData,
		cp: &Arc<ConstantPool>,
	) -> *const c_void;
}

pub struct ThreadHandle {
	data: Arc<ThreadConfig>,
	handle: JoinHandle<Option<DynValue>>,
	sender: Sender<ThreadCommand>,
}

impl ThreadHandle {
	pub fn new(
		config: ThreadConfig,
		func: impl FnOnce(Arc<ThreadConfig>, Receiver<ThreadCommand>) -> Option<DynValue>
			+ Send
			+ 'static,
	) -> ThreadHandle {
		let data = Arc::new(config);
		let (sender, receiver) = unbounded();
		let data2 = data.clone();

		let handle = spawn(|| func(data2, receiver));

		ThreadHandle {
			data,
			handle,
			sender,
		}
	}

	pub fn join(self) -> thread::Result<Option<DynValue>> {
		self.handle.join()
	}

	pub fn name(&self) -> &str {
		self.data.name.as_str()
	}

	pub fn run(&self, ty: ObjectType, method: MethodIdentifier, parameters: Vec<DynValue>) {
		self.sender
			.send(ThreadCommand::Run {
				ty: ty,
				method: method,
				parameters: parameters,
			})
			.unwrap();
	}
}

pub enum ThreadCommand {
	Run {
		ty: ObjectType,
		method: MethodIdentifier,
		parameters: Vec<DynValue>,
	},
	Exit,
}

pub struct ThreadConfig {
	pub name: String,
}
