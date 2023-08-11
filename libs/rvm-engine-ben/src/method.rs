use crate::code::Task;
use rvm_core::{Kind, StorageValue, Type};
use rvm_object::{Method, ObjectClass};
use rvm_reader::Code;

pub struct CompiledMethod {
	pub max_locals: u16,
	pub max_stack: u16,
	pub tasks: Vec<Task>,
	pub parameters: Vec<Type>,
	pub returns: Option<Kind>,
}

impl CompiledMethod {
	pub fn new(code: &Code, class: &ObjectClass, method: &Method) -> CompiledMethod {
		CompiledMethod {
			max_locals: code.max_locals as u16,
			max_stack: code.max_stack as u16,
			tasks: code
				.instructions
				.iter()
				.map(|v| Task::new(v, class))
				.collect(),
			parameters: method.desc.parameters.to_vec(),
			returns: method.desc.ret.as_ref().map(|v| v.kind()),
		}
	}
}

impl StorageValue for CompiledMethod {
	type Idx = u32;
}
