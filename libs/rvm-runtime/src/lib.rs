#![feature(hash_drain_filter)]
#![feature(drain_filter)]
#![feature(array_try_from_fn)]
#![feature(thread_local)]
#![feature(thread_id_value)]

use std::sync::Arc;
use std::thread::spawn;

use parking_lot::Mutex;

pub use object::*;
pub use value::*;

use crate::engine::Engine;
use crate::gc::GarbageCollector;

pub mod engine;
pub mod error;
pub mod gc;
#[cfg(feature = "native")]
pub mod native;
mod object;
pub mod prelude;
mod value;

/// A runtime which (almost never) conforms to [The Java Virtual Machine Specification, Java SE 19 Edition][jvms]
///
/// The runtime includes a bootstrap class source, a classloader and JITting compiler using [LLVM][llvm]
///
/// [jvms]: https://docs.oracle.com/javase/specs/jvms/se19/html/index.html
/// [llvm]: https://llvm.org/
pub struct Runtime {
	pub cl: ClassLoader,
	pub engine: Box<dyn Engine>,
	pub gc: Mutex<GarbageCollector>,
}

impl Runtime {
	pub fn new(heap_size: usize, engine: Box<dyn Engine>) -> Runtime {
		Runtime {
			cl: ClassLoader::new(),
			engine,
			gc: Mutex::new(GarbageCollector::new(heap_size)),
		}
	}

	pub fn gc(runtime: Arc<Runtime>) {
		spawn(move || {
			let mut gc = runtime.gc.lock();
			gc.gc();
		});
	}

	// /// Compiles a method with a given identifier. Uses the mapping in [`java!`]
	// 	///
	// 	/// ```
	// 	/// use std::mem::transmute;
	// 	/// use std::pin::Pin;
	// 	/// use rvm_runtime::Runtime;
	// 	///
	// 	/// |runtime: &Pin<&Runtime>| {
	// 	/// 	let pointer = runtime.compile_method("Main", "update", "(I)I");
	// 	/// 	let function = unsafe { transmute::<_, extern "C" fn(i32) -> i32>(pointer) };
	// 	/// 	let out = unsafe { function(3) };
	// 	/// };
	// 	/// ```
	// 	pub fn compile_method(
	// 		self: &Pin<&Self>,
	// 		class_name: &str,
	// 		method_name: &str,
	// 		desc: &str,
	// 	) -> *const c_void {
	// 		info!("Resolving {class_name}:{method_name}:{desc}");
	// 		let class_id = self.cl.get_class_id(&Type::Object(ObjectType {
	// 			name: class_name.to_string(),
	// 		}));
	//
	// 		let class = self.cl.get(class_id);
	// 		let class = match &class.kind {
	// 			ClassKind::Object(obj) => obj,
	// 			_ => {
	// 				panic!("Invalid type")
	// 			}
	// 		};
	//
	// 		let method_id = class
	// 			.methods
	// 			.get_id(&MethodIdentifier {
	// 				name: method_name.to_string(),
	// 				descriptor: desc.to_string(),
	// 			})
	// 			.expect("Method not found");
	//
	// 		let method = class.methods.get(method_id);
	// 		return match method.compiled.get() {
	// 			None => {
	// 				if let Some(code) = &method.code {
	// 					match code.as_ref() {
	// 						MethodCode::Java(_) => {
	// 							let value = self.engine.compile_method(self, method, &class.cp)
	// 								as *const c_void;
	// 							method.compiled.set(Some(value));
	// 							value
	// 						}
	// 						MethodCode::Native(either) => {
	// 							let code = match &either {
	// 								Either::Left(source) => {
	// 									let code = self.cl.native_methods().get(source).unwrap();
	// 									// todo: save in method.code as Some(MethodCode::Native(Either::Right(*code)))
	// 									code
	// 								}
	// 								Either::Right(code) => code,
	// 							};
	//
	// 							panic!("Native code is not supported yet");
	// 						}
	// 					}
	// 				} else {
	// 					panic!("Code is missing");
	// 				}
	// 			}
	// 			Some(value) => value,
	// 		};
	// 	}
}
