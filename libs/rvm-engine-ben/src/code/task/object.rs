use std::fmt::{Display, Formatter};

use rvm_core::ObjectType;
use rvm_reader::NewInst;
use rvm_runtime::InstanceClass;

#[derive(Debug)]
pub struct NewTask {
	pub class_name: ObjectType,
}

impl Display for NewTask {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "NEW {}", self.class_name)
	}
}

impl NewTask {
	pub fn new(inst: &NewInst, class: &InstanceClass) -> NewTask {
		let class_data = inst.class.get(&class.cp).unwrap();
		let name = class_data.name.get(&class.cp).unwrap();
		NewTask {
			class_name: ObjectType::new(name.to_string()),
		}
	}
}
