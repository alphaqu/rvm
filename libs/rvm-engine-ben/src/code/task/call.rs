use rvm_core::ObjectType;
use rvm_object::{MethodIdentifier, ObjectClass};
use rvm_reader::InvokeInst;

#[derive(Debug)]

pub struct CallTask {
	pub method: MethodIdentifier,
	pub object: ObjectType,
}

impl CallTask {
	pub fn new(inst: &InvokeInst, class: &ObjectClass) -> CallTask {
		let method = inst.value.get(&class.cp);
		let name_and_type = method.name_and_type.get(&class.cp);
		let target = method.class.get(&class.cp);
		let name = target.name.get(&class.cp);

		CallTask {
			method: MethodIdentifier::new(&name_and_type, &class.cp),
			object: ObjectType {
				name: name.to_string(),
			},
		}
	}
}
