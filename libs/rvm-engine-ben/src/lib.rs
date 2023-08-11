#![feature(pointer_byte_offsets)]
#![feature(generic_const_exprs)]

use rvm_core::{Id, ObjectType, StackKind, Storage, Type};
use rvm_object::{ClassKind, MethodCode, MethodData, MethodIdentifier};
use rvm_reader::ConstantPool;
use rvm_runtime::engine::{Engine, ThreadConfig, ThreadHandle};
use rvm_runtime::Runtime;
use std::ffi::c_void;
use std::mem::forget;
use std::pin::Pin;
use std::sync::{Arc, RwLock};
use tracing::debug;

use crate::code::{
	CombineTask, CombineTaskOperation, CombineTaskType, ConstTask, LocalTask, LocalTaskKind,
	ReturnTask, Task,
};
use crate::method::CompiledMethod;
use crate::thread::{Thread, ThreadStack};

mod code;
mod method;
mod thread;
mod value;

pub struct BenEngine {
	// TODO fuck the clones
	methods: RwLock<Storage<(ObjectType, MethodIdentifier), Arc<CompiledMethod>>>,
}

impl BenEngine {
	pub fn get_compiled_method(
		&self,
		runtime: &Runtime,
		ty: ObjectType,
		method: MethodIdentifier,
	) -> Arc<CompiledMethod> {
		let key = (ty, method);
		let guard = self.methods.read().unwrap();
		match guard.get_keyed(&key) {
			None => {
				drop(guard);

				debug!(target: "ben", "Compiling method {key:?}");
				let id = runtime.cl.get_class_id(&Type::Object(key.0.clone()));
				let cl_guard = runtime.cl.get(id);
				let class = match &cl_guard.kind {
					ClassKind::Object(object) => object,
					_ => unreachable!(),
				};

				let raw_method = class
					.methods
					.get_keyed(&key.1)
					.expect("Could not find method");
				let method = raw_method
					.code
					.clone()
					.expect("Method does not contain code");
				let compiled_method = Arc::new(match &(*method) {
					MethodCode::Java(code) => CompiledMethod::new(code, class, raw_method),
					MethodCode::Native(_) => {
						todo!()
					}
				});

				let mut guard = self.methods.write().unwrap();
				guard.insert(key, compiled_method.clone());
				compiled_method
			}
			Some(value) => value.clone(),
		}
	}
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
		Thread::spawn(runtime, config, 1024 * 8, self.engine.clone())
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
