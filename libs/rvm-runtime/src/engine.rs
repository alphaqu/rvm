use std::ffi::c_void;
use std::pin::Pin;
use std::sync::Arc;
use std::thread;
use std::thread::{spawn, JoinHandle};

use crossbeam::channel::{unbounded, Receiver, Sender};
use crossbeam::sync::{Parker, Unparker};

use crate::gc::GcSweeper;
use crate::object::{MethodData, MethodIdentifier};
use rvm_core::ObjectType;
use rvm_reader::ConstantPool;

use crate::value::AnyValue;
use crate::Runtime;

pub trait Engine: Send + Sync {
	fn create_thread(&self, runtime: Arc<Runtime>, config: ThreadConfig) -> ThreadHandle;

	fn compile_method(
		&self,
		runtime: &Pin<&Runtime>,
		method: &MethodData,
		cp: &Arc<ConstantPool>,
	) -> *const c_void;
}

pub struct Thread {
	pub gc: GcSweeper,
	pub config: Arc<ThreadConfig>,
	pub receiver: Receiver<ThreadCommand>,
}

pub struct ThreadHandle {
	data: Arc<ThreadConfig>,
	handle: JoinHandle<Option<AnyValue>>,
	sender: Sender<ThreadCommand>,
}

impl ThreadHandle {
	pub fn new(
		runtime: Arc<Runtime>,
		config: ThreadConfig,
		func: impl FnOnce(Thread) -> Option<AnyValue> + Send + 'static,
	) -> ThreadHandle {
		let data = Arc::new(config);
		let (sender, receiver) = unbounded();
		let data2 = data.clone();

		let sweeper = runtime.gc.lock().new_sweeper();
		let handle = spawn(|| {
			func(Thread {
				gc: sweeper,
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

	pub fn join(self) -> thread::Result<Option<AnyValue>> {
		self.handle.join()
	}

	pub fn name(&self) -> &str {
		self.data.name.as_str()
	}

	pub fn run(&self, ty: ObjectType, method: MethodIdentifier, parameters: Vec<AnyValue>) {
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
		parameters: Vec<AnyValue>,
	},
	Exit,
}

pub struct ThreadConfig {
	pub name: String,
}
