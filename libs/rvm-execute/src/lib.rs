use rvm_consts::MethodAccessFlags;
use rvm_core::{StorageValue, Type};
use rvm_reader::{Code, ConstantPool};
use std::ffi::{c_char, c_void};

pub trait ExecutionEngine {
	fn compile_method(&self, bindings: &Bindings, method: &Method, cp: &ConstantPool) -> *const c_void;
}

pub struct Bindings {
	pub runtime_global: *const c_void,
	pub resolve_method: extern "C" fn(
		runtime: *const c_void,
		class: *const c_char,
		method: *const c_char,
		desc: *const c_char,
	),
}

pub struct Method {
	pub name: String,
	pub desc: MethodDesc,
	pub flags: MethodAccessFlags,
	pub code: Code,
}

pub struct MethodDesc {
	parameters: Vec<Type>,
	ret: Option<Type>,
}

impl StorageValue for Method {
	type Idx = u16;
}
