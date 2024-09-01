#![feature(pointer_byte_offsets)]
#![feature(generic_const_exprs)]

use std::ffi::c_void;
use std::pin::Pin;
use std::sync::{Arc, RwLock};

use tracing::{debug, info};

use rvm_core::{Id, MethodAccessFlags, MethodDescriptor, ObjectType, Storage, StorageValue, Type};
use rvm_reader::{Code, ConstantPool};
use rvm_runtime::engine::{Engine, ThreadConfig, ThreadHandle};
use rvm_runtime::{
	Class, InstanceClass, Method, MethodBinding, MethodCode, MethodIdentifier, Runtime,
};

use crate::method::JavaMethod;
use crate::thread::spawn;

mod code;
mod method;
mod thread;
mod value;

pub struct BenEngine {
	pub methods: RwLock<Storage<(Id<Class>, Id<Method>), BenMethod, Arc<BenMethod>>>,
}

impl BenEngine {
	pub fn resolve_method(
		&self,
		runtime: &Runtime,
		mut id: Id<Class>,
		method: &MethodIdentifier,
	) -> Option<(Id<Class>, Id<Method>)> {
		let mut cl_guard = runtime.cl.get(id);
		let mut class = cl_guard.as_instance().unwrap();

		let mut method_value: Option<Id<Method>> = class.methods.get_id(method);
		while method_value.is_none() {
			cl_guard = runtime.cl.get(id);
			class = cl_guard.as_instance().unwrap();
			method_value = class.methods.get_id(method);
			id = class.super_id?;
		}

		Some((id, method_value?))
	}

	pub fn compile_method(
		&self,
		runtime: &Runtime,
		id: Id<Class>,
		method_id: Id<Method>,
	) -> Arc<BenMethod> {
		let methods = self.methods.read().unwrap();
		if let Some(method) = methods.get_keyed(&(id, method_id)) {
			return method.clone();
		}
		drop(methods);

		let arc = runtime.cl.get(id);
		let instance = arc.as_instance().unwrap();
		let method = instance.methods.get(method_id);
		debug!(target: "ben", "Resolving method {}.{}{}", instance.ty, method.name, method.desc);

		let code = method.code.as_ref();
		let ben_method = Arc::new(match code {
			Some(MethodCode::Java(code)) => {
				let compiled = JavaMethod::new(code, instance, method);
				BenMethod::Java(compiled)
			}
			Some(MethodCode::Binding(binding)) => {
				let binding_guard = runtime.bindings.read();
				let binding = binding_guard.get(binding).unwrap();
				BenMethod::Binding(binding.clone())
			}
			None => {
				if method.flags.contains(MethodAccessFlags::NATIVE) {
					let binding_guard = runtime.bindings.read();
					let identifier = MethodIdentifier {
						name: method.name.clone(),
						descriptor: method.desc.to_string(),
					};
					info!("Trying to find {identifier:?}");
					if let Some(binding) = binding_guard.get(&identifier) {
						BenMethod::Binding(binding.clone())
					} else {
						BenMethod::Native(
							format!("Java_{}_{}", instance.ty.0.replace('/', "_"), method.name),
							method.desc.clone(),
						)
					}
				} else {
					panic!("Could not find method")
				}
			}
		});

		let mut guard = self.methods.write().unwrap();
		guard.insert((id, method_id), ben_method.clone());
		ben_method
	}
}

pub enum BenMethod {
	Java(JavaMethod),
	Binding(MethodBinding),
	Native(String, MethodDescriptor),
}

impl StorageValue for BenMethod {
	type Idx = u32;
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
		method: &Method,
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
