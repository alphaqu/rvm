#![feature(pointer_byte_offsets)]
#![feature(generic_const_exprs)]

use std::ffi::c_void;
use std::pin::Pin;
use std::sync::{Arc, RwLock};

use tracing::debug;

use rvm_core::{ObjectType, Storage, Type};
use rvm_object::{Method, MethodCode, MethodData, MethodIdentifier};
use rvm_reader::ConstantPool;
use rvm_runtime::engine::{Engine, ThreadConfig, ThreadHandle};
use rvm_runtime::Runtime;

use crate::method::CompiledMethod;
use crate::thread::spawn;

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
		let mut id = runtime.class_loader.get_class_id(&Type::Object(ty.clone()));
		let mut cl_guard = runtime.class_loader.get(id);
		let mut class = cl_guard.object().unwrap();

		let mut method_value: Option<&Method> = class.methods.get_keyed(&method);
		while method_value.is_none() {
			cl_guard = runtime.class_loader.get(id);
			class = cl_guard.object().unwrap();
			method_value = class.methods.get_keyed(&method);
			id = class.super_id.expect("Could not resolve method");
		}

		let method_value = method_value.unwrap();
		let method_code = method_value
			.code
			.clone()
			.expect("Method does not contain code");

		let key = (ty, method);
		let guard = self.methods.read().unwrap();
		match guard.get_keyed(&key) {
			None => {
				drop(guard);
				debug!(target: "ben", "Compiling method {key:?}");
				let compiled_method = Arc::new(match &(*method_code) {
					MethodCode::Java(code) => CompiledMethod::new(code, class, method_value),
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
