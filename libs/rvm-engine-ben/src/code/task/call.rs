use rvm_core::ObjectType;
use rvm_object::{MethodIdentifier, ObjectClass};
use rvm_reader::InvokeInst;
use std::fmt::{Display, Formatter};

#[derive(Debug)]

pub struct CallTask {
	pub method: MethodIdentifier,
	pub object: ObjectType,
}

impl Display for CallTask {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "call {:?}.{:?}()", self.object, self.method)
	}
}
impl CallTask {
	pub fn new(inst: &InvokeInst, class: &ObjectClass) -> CallTask {
		let method = inst.value.get(&class.cp);
		let name_and_type = method.name_and_type.get(&class.cp);
		let target = method.class.get(&class.cp);
		let name = target.name.get(&class.cp);

		CallTask {
			method: MethodIdentifier::new(&name_and_type, &class.cp),
			object: ObjectType(name.to_string()),
		}
	}
}
