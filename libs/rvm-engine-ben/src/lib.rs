#![feature(pointer_byte_offsets)]
#![feature(generic_const_exprs)]

use std::ffi::c_void;
use std::pin::Pin;
use std::sync::{Arc, RwLock};

use tracing::debug;

use rvm_core::{Id, ObjectType, Storage, Type};
use rvm_reader::{Code, ConstantPool};
use rvm_runtime::engine::{Engine, ThreadConfig, ThreadHandle};
use rvm_runtime::{
	Class, InstanceClass, Method, MethodBinding, MethodCode, MethodData, MethodIdentifier, Runtime,
};

use crate::method::JavaMethod;
use crate::thread::spawn;

mod code;
mod method;
mod thread;
mod value;

pub struct BenEngine {
	// TODO fuck the clones
	pub methods: RwLock<Storage<(Id<Class>, Id<Method>), Arc<JavaMethod>>>,
}

impl BenEngine {
	pub fn resolve_method(
		&self,
		runtime: &Runtime,
		mut id: Id<Class>,
		method: MethodIdentifier,
	) -> Option<(Id<Class>, Id<Method>)> {
		let mut cl_guard = runtime.cl.get(id);
		let mut class = cl_guard.as_instance().unwrap();

		let mut method_value: Option<Id<Method>> = class.methods.get_id(&method);
		while method_value.is_none() {
			cl_guard = runtime.cl.get(id);
			class = cl_guard.as_instance().unwrap();
			method_value = class.methods.get_id(&method);
			id = class.super_id?;
		}

		Some((id, method_value?))
	}
}

pub enum MethodCalling {
	Java(Arc<JavaMethod>),
}

pub struct BenBinding {
	engine: Arc<BenEngine>,
}

impl BenBinding {
	pub fn new() -> Self {
		Self {
			engine: Arc::new(BenEngine {
				methods: RwLock::new(Storage::new()),
			}),
		}
	}
}

impl Engine for BenBinding {
	fn create_thread(&self, runtime: Arc<Runtime>, config: ThreadConfig) -> ThreadHandle {
		spawn(runtime, config, 1024 * 8, self.engine.clone())
	}

	fn compile_method(
		&self,
		runtime: &Pin<&Runtime>,
		method: &MethodData,
		cp: &Arc<ConstantPool>,
	) -> *const c_void {
		todo!()
	}
}

//pub fn main() {
// 	rvm_core::init();
// 	ThreadStack::new(1024 * 4, |ts| {
// 		let result = ts.exec(&CompiledMethod {
// 			max_locals: 2,
// 			max_stack: 2,
// 			tasks: vec![
// 				Task::Const(ConstTask::Double(1.0)),
// 				Task::Local(LocalTask {
// 					kind: LocalTaskKind::Store,
// 					ty: StackKind::Double,
// 					idx: 0,
// 				}),
// 				Task::Local(LocalTask {
// 					kind: LocalTaskKind::Load,
// 					ty: StackKind::Double,
// 					idx: 0,
// 				}),
// 				Task::Const(ConstTask::Double(2.0)),
// 				Task::Combine(CombineTask {
// 					ty: CombineTaskType::Double,
// 					op: CombineTaskOperation::Div,
// 				}),
// 				Task::Return(ReturnTask {
// 					kind: Some(StackKind::Double),
// 				}),
// 			],
// 		});
// 		println!("{:?}", result);
// 	});
// }
