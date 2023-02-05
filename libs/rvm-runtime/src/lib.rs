#![feature(generic_const_exprs)]
#![feature(hash_drain_filter)]
#![feature(drain_filter)]
#![feature(array_try_from_fn)]
#![feature(box_syntax)]
#![feature(thread_local)]
#![feature(thread_id_value)]

use std::ffi::c_void;
use std::pin::Pin;

use either::Either;
use mmtk::vm::VMBinding;
use tracing::info;
use rvm_core::{ObjectType, Type};
use rvm_object::{ClassLoader, MethodCode, MethodIdentifier};
use crate::arena::Arena;

use crate::engine::Engine;

pub mod error;
pub mod prelude;

pub mod engine;
#[cfg(feature = "native")]
pub mod native;
pub mod arena;

/// A runtime which (almost never) conforms to [The Java Virtual Machine Specification, Java SE 19 Edition][jvms]
///
/// The runtime includes a bootstrap class source, a classloader and JITting compiler using [LLVM][llvm]
///
/// [jvms]: https://docs.oracle.com/javase/specs/jvms/se19/html/index.html
/// [llvm]: https://llvm.org/
pub struct Runtime {
	pub cl: ClassLoader,
	pub engine: Box<dyn Engine>,
	pub arena: Arena,
}

impl Runtime {
	pub fn new(heap_size: usize, engine: Box<dyn Engine>) -> Runtime {
		Runtime {
			cl: ClassLoader::new(),
			engine,
			arena: Arena::init(heap_size),
		}
	}

	/// Compiles a method with a given identifier. Uses the mapping in [`java!`]
	///
	/// ```
	/// use std::mem::transmute;
	/// use std::pin::Pin;
	/// use rvm_runtime::Runtime;
	///
	/// |runtime: &Pin<&Runtime>| {
	/// 	let pointer = runtime.compile_method("Main", "update", "(I)I");
	/// 	let function = unsafe { transmute::<_, extern "C" fn(i32) -> i32>(pointer) };
	/// 	let out = unsafe { function(3) };
	/// };
	/// ```
	pub fn compile_method(
		self: &Pin<&Self>,
		class_name: &str,
		method_name: &str,
		desc: &str,
	) -> *const c_void {
		info!("Resolving {class_name}:{method_name}:{desc}");
		let class_id = self.cl.get_class_id(&Type::Object(ObjectType {
			name: class_name.to_string(),
		}));
		let class = self.cl.get_obj_class(class_id);

		let method_id = class
			.methods
			.get_id(&MethodIdentifier {
				name: method_name.to_string(),
				descriptor: desc.to_string(),
			})
			.expect("Method not found");

		let method = class.methods.get(method_id);
		return match method.compiled.get() {
			None => {
				if let Some(code) = &method.code {
					match code.as_ref() {
						MethodCode::Java(_) => {
							let value = self.engine.compile_method(self, method, &class.cp)
								as *const c_void;
							method.compiled.set(Some(value));
							value
						}
						MethodCode::Native(either) => {
							let code = match &either {
								Either::Left(source) => {
									let code = self.cl.native_methods().get(source).unwrap();
									// todo: save in method.code as Some(MethodCode::Native(Either::Right(*code)))
									code
								}
								Either::Right(code) => code,
							};

							panic!("Native code is not supported yet");
						}
					}
				} else {
					panic!("Code is missing");
				}
			}
			Some(value) => value,
		};
	}
}
