use rvm_core::ObjectType;
use rvm_object::ObjectClass;
use rvm_reader::NewInst;

#[derive(Debug)]
pub struct NewTask {
	pub class_name: ObjectType,
}

impl NewTask {
	pub fn new(inst: &NewInst, class: &ObjectClass) -> NewTask {
		let class_data = inst.class.get(&class.cp);
		let name = class_data.name.get(&class.cp);
		NewTask {
			class_name: ObjectType(name.to_string()),
		}
	}
}
