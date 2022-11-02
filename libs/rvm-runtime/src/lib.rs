#![feature(generic_const_exprs)]
#![feature(hash_drain_filter)]
#![feature(drain_filter)]
#![feature(array_try_from_fn)]
#![feature(box_syntax)]

use crate::class::{Class, ClassKind};
use crate::gc::GarbageCollector;
use crate::object::Ref;
use crate::object::{Field, MethodCode};
use crate::object::{Method, MethodIdentifier, NativeCode};
use crate::reader::{
	BinaryName, ClassConst, ClassInfo, ConstPtr, ConstantPool, FieldConst, MethodConst,
	MethodDescriptor, StrParse, ValueDesc,
};
use ahash::HashMap;
use std::ffi::{c_char, c_void, CStr, CString};
use std::mem::{transmute, transmute_copy};
use std::ops::{Deref, DerefMut};

use inkwell::context::Context;
use inkwell::execution_engine::{JitFunction, UnsafeFunctionPointer};
use rvm_consts::MethodAccessFlags;
use std::sync::RwLock;
use tracing::{debug, info};

use crate::class_loader::ClassLoader;
use crate::compiler::{Executor, MethodReference};
use crate::error::{JError, JResult};
use rvm_core::Id;

pub mod class;
mod class_loader;
pub mod compiler;
pub mod convert;
pub mod error;
pub mod executor;
pub mod gc;
pub mod object;
pub mod reader;

//     static int ack(int m, int n) {
//         if (m == 0) {
//             return n + 1;
//         } else if (m > 0 && n == 0) {
//             return ack(m - 1, 1);
//         } else if (m > 0 && n > 0) {
//             return ack(m - 1, ack(m, n - 1));
//         } else {
//             return n + 1;
//         }
//     }

pub fn ack(m: i32, n: i32) -> i32 {
	if m == 0 {
		return n + 1;
	} else if m > 0 && n == 0 {
		return ack(m - 1, 1);
	} else if m > 0 && n > 0 {
		return ack(m - 1, ack(m, n - 1));
	} else {
		return n + 1;
	}
}

#[cfg(feature = "native")]
pub mod native;

pub extern "C" fn compile_method(
	runtime: *const Runtime,
	class: *const c_char,
	method: *const c_char,
	desc: *const c_char,
) -> *const c_void {
	let runtime = unsafe { &*runtime };
	let class = unsafe { CStr::from_ptr(class) }.to_str().unwrap();
	let method = unsafe { CStr::from_ptr(method) }.to_str().unwrap();
	let desc = unsafe { CStr::from_ptr(desc) }.to_str().unwrap();
	compile_method_rust(runtime, class, method, desc)
}

pub fn compile_method_rust(
	runtime: &Runtime,
	class_name: &str,
	method_name: &str,
	desc: &str,
) -> *const c_void {
	let string = format!("{class_name}:{method_name}:{desc}");
	info!("Resolving {class_name}:{method_name}:{desc}");
	let class_id = runtime
		.cl
		.get_class_id(&BinaryName::Object(class_name.to_string()));
	let class = runtime.cl.get_obj_class(class_id);

	let method_id = class
		.methods
		.get_id(&MethodIdentifier {
			name: method_name.to_string(),
			descriptor: desc.to_string(),
		})
		.expect("haha");

	let method = class.methods.get(method_id);

	if let Some(MethodCode::JVM(code)) = &method.code {
		let function = runtime.compiler.compile_method(
			runtime as *const _,
			&MethodReference {
				class_name: class_name.to_string(),
				method_name: method_name.to_string(),
				desc: desc.to_string()
			},
			method.flags.contains(MethodAccessFlags::STATIC),
			&**code,
			&class.cp,
		) as *const c_void;
		info!("Resolved {string}");
		return function;
	}

	panic!("native method cringe");
}

pub struct Runtime<'ctx> {
	pub cl: ClassLoader,
	pub gc: RwLock<GarbageCollector>,
	pub compiler: Executor<'ctx>,
}

impl<'ctx> Runtime<'ctx> {
	pub fn new(ctx: &'ctx Context) -> Runtime<'ctx> {
		Runtime {
			cl: ClassLoader::new(),
			gc: RwLock::new(GarbageCollector::new()),
			compiler: Executor::new(ctx),
		}
	}

	//pub fn compile_method<V: UnsafeFunctionPointer>(
	//	&self,
	//	class_id: Id<Class>,
	//	method_id: Id<Method>,
	//) -> JitFunction<V> {
	//	let class = self.cl.get_obj_class(class_id);
	//	let method = class.methods.get(method_id);
	//
	//	if let Some(MethodCode::JVM(code)) = &method.code {
	//		let name = format!("{class}:{name}:{desc}");
	//
	//		let string = format!("CLASS{}_METHOD{}", class_id.idx(), method.name);
	//		return unsafe {
	//			transmute_copy(&self.compiler.compile_method(
	//				&string,
	//				method.flags.contains(MethodAccessFlags::STATIC),
	//				&method.desc.clone(),
	//				code.as_ref(),
	//				&class.cp,
	//			))
	//		};
	//	}
	//
	//	panic!("native method bruh")
	//}

	//pub fn compile_method_raw(&self, class: CString, method: CString, desc: CString) -> usize {
	//	let class = class.to_str().unwrap();
	//	let method = method.to_str().unwrap();
	//	let desc = desc.to_str().unwrap();
	//
	//	let class_id = self.cl.get_class_id(&BinaryName::Object(class.to_string()));
	//
	//	let class = self.cl.get_obj_class(class_id);
	//	let method_id = class
	//		.methods
	//		.get_id(&MethodIdentifier {
	//			name: method.to_string(),
	//			descriptor: desc.to_string(),
	//		})
	//		.unwrap();
	//
	//	let method = class.methods.get(method_id);
	//
	//	if let Some(MethodCode::JVM(code)) = &method.code {
	//		let string = format!("CLASS{}_METHOD{}", class_id.idx(), method.name);
	//		return self.compiler.compile_method(
	//			&string,
	//			method.flags.contains(MethodAccessFlags::STATIC),
	//			&method.desc.clone(),
	//			code.as_ref(),
	//			&class.cp,
	//		);
	//	}
	//
	//	panic!("native stuff")
	//}

	pub fn resolve_class(
		&self,
		from: Id<Class>,
		class_ptr: ConstPtr<ClassConst>,
	) -> JResult<Id<Class>> {
		let desc = {
			// very important to free the class lock if its gonna get resolved
			let class = self.cl.get_obj_class(from);
			let class_const = class_ptr.get(&class.cp);
			if let Some(value) = class_const.link.get() {
				// symbolic link fast af
				return Ok(value);
			}

			let desc1 = class_const.name.get(&class.cp).as_str().replace('/', ".");
			BinaryName::parse(&desc1)
		};

		debug!(target: "resolve", "Resolving class \"{:?}\"", desc);
		let id = self.cl.get_class_id(&desc);

		// Link the value
		let class = self.cl.get_obj_class(from);
		class_ptr.get(&class.cp).link.replace(Some(id));
		Ok(id)
	}

	pub fn resolve_field(
		&self,
		from: Id<Class>,
		field_ptr: ConstPtr<FieldConst>,
	) -> JResult<(Id<Class>, Id<Field>)> {
		let from_class = self.cl.get_obj_class(from);
		let field_const = field_ptr.get(&from_class.cp);
		if let Some(value) = field_const.link.get() {
			let class_id = field_const
				.class
				.get(&from_class.cp)
				.link
				.get()
				.expect("Field linked to a non linked class");
			return Ok((class_id, value));
		}

		let name_and_type = field_const.name_and_type.get(&from_class.cp);
		let name = name_and_type.name.get(&from_class.cp).to_string();
		let class_ptr = field_const.class;
		//let descriptor = name_and_type.descriptor.get(&class.cp).as_str();

		debug!(target: "resolve", "Resolving field \"{}\"", name);
		// to allow for loading incase it gets defined
		drop(from_class);
		let class_id = self.resolve_class(from, class_ptr)?;

		let class = self.cl.get_obj_class(class_id);
		if let Some(id) = class.fields.get_id(&name) {
			let from_class = self.cl.get_obj_class(from);
			field_ptr.get(&from_class.cp).link.replace(Some(id));
			return Ok((class_id, id));
		}
		//let class_id = self.class.get(cp).get_id(cp, runtime)?;
		//         let name_and_type = self.name_and_type.get(cp);
		//         let name = name_and_type.name.get(cp).as_str();
		//
		//         let id = runtime.get_field(class_id, name)?;
		//         self.link.replace(Some(id));
		//         Ok(id)
		//
		//         debug!(target: "resolve", "Resolving field \"{}\"", field);
		//         let class = self.class_loader.get(from);
		//         match &class.kind {
		//             ClassKind::Object(object) => {
		//                 if let Some(value) = object.fields.get_id(field) {
		//                     return Ok(value)
		//                 }
		//             }
		//             _ => {
		//                 panic!("Expected object but found other")
		//             }
		//         }
		panic!("Failed to resolve field. SUPER not yet supported")
	}

	pub fn resolve_method(
		&self,
		from: Id<Class>,
		method_ptr: ConstPtr<MethodConst>,
	) -> JResult<(Id<Class>, Id<Method>)> {
		let from_class = self.cl.get_obj_class(from);
		let method_const = method_ptr.get(&from_class.cp);
		if let Some(value) = method_const.link.get() {
			let class_id = method_const
				.class
				.get(&from_class.cp)
				.link
				.get()
				.expect("Method linked to a non linked class");
			return Ok((class_id, value));
		}

		let name_and_type = method_const.name_and_type.get(&from_class.cp);
		let name = MethodIdentifier::new(name_and_type, &from_class.cp);

		let class_ptr = method_const.class;
		//let descriptor = name_and_type.descriptor.get(&class.cp).as_str();

		debug!(target: "resolve", "Resolving method \"{:?}\"", name);

		// to allow for loading incase it gets defined
		drop(from_class);
		let class_id = self.resolve_class(from, class_ptr)?;

		let class = self.cl.get_obj_class(class_id);
		if let Some(id) = class.methods.get_id(&name) {
			let from_class = self.cl.get_obj_class(from);
			method_ptr.get(&from_class.cp).link.replace(Some(id));
			return Ok((class_id, id));
		}
		//      if let Some(value) = self.link.get() {
		// 			return Ok(value);
		// 		}
		//
		// 		let class_id = self.class.get(cp).get_id(cp, runtime)?;
		// 		let name_and_type = self.name_and_type.get(cp);
		// 		debug!(target: "resolve", "Resolving method \"{}\"", name_and_type.name.get(cp).as_str());
		// 		let identifier = MethodIdentifier::new(name_and_type, cp);
		//
		// 		let id = runtime.get_method(class_id, &identifier)?;
		// 		self.link.replace(Some(id));
		// 		Ok(id)
		// debug!(target: "resolve", "Resolving method \"{method:?}\"");
		//
		//         let class = self.class_loader.get(class_id);
		//         match &class.kind {
		//             ClassKind::Object(object) => {
		//                 if let Some(value) = object.methods.get_id(method) {
		//                     return Ok(value);
		//                 }
		//             }
		//             _ => {
		//                 panic!("Expected object but found other")
		//             }
		//         }

		panic!("Failed to resolve method. SUPER not yet supported")
	}
}

pub struct CringeContext(pub Context);

unsafe impl Sync for CringeContext {}

unsafe impl<'a> Sync for Runtime<'a> {}

unsafe impl<'a> Send for Runtime<'a> {}

impl Deref for CringeContext {
	type Target = Context;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for CringeContext {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}
