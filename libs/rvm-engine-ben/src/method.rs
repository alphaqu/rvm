use rvm_core::{Kind, MethodAccessFlags, StorageValue, Type};
use rvm_reader::Code;
use rvm_runtime::{InstanceClass, Method};

use crate::code::Task;

pub struct JavaMethod {
	pub max_locals: u16,
	pub max_stack: u16,
	pub flags: MethodAccessFlags,
	pub tasks: Vec<Task>,
	pub parameters: Vec<Type>,
	pub returns: Option<Kind>,
}

impl JavaMethod {
	pub fn new(code: &Code, class: &InstanceClass, method: &Method) -> JavaMethod {
		JavaMethod {
			max_locals: code.max_locals,
			max_stack: code.max_stack,
			flags: method.flags,
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

impl StorageValue for JavaMethod {
	type Idx = u32;
}
