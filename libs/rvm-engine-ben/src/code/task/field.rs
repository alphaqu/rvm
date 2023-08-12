use rvm_core::{ObjectType, Type};
use rvm_object::{Class, ObjectClass};
use rvm_reader::{FieldInst, FieldInstKind};
use rvm_runtime::arena::object::Object;
use rvm_runtime::Runtime;

use crate::thread::ThreadFrame;
use crate::value::StackValue;

#[derive(Debug)]
pub struct FieldTask {
	pub source: ObjectType,
	pub field_name: String,
	pub instance: bool,
	pub kind: FieldInstKind,
}

impl FieldTask {
	pub fn new(inst: &FieldInst, class: &ObjectClass) -> FieldTask {
		let field = inst.value.get(&class.cp);
		let source = field.class.get(&class.cp);
		let source = source.name.get(&class.cp);
		let field = field.name_and_type.get(&class.cp);
		let field_name = field.name.get(&class.cp);
		let field_descriptor = field.descriptor.get(&class.cp);
		FieldTask {
			source: ObjectType(source.to_string()),
			field_name: field_name.to_string(),
			instance: inst.instance,
			kind: inst.kind,
		}
	}

	pub fn exec(&self, runtime: &Runtime, frame: &mut ThreadFrame) {
		let id = runtime
			.class_loader
			.get_class_id(&Type::Object(self.source.clone()));
		let arc = runtime.class_loader.get(id);
		match &arc.kind {
			Class::Object(object) => {
				let field = object
					.fields
					.get_keyed(&self.field_name)
					.expect("Could not find field");
				if field.is_static() {
					if self.instance {
						panic!("Found static field on INSTANCE field op");
					}
					todo!()
				} else {
					if !self.instance {
						panic!("Found instance field on STATIC field op");
					}

					match self.kind {
						FieldInstKind::Get => {
							let reference = frame.pop().to_ref();

							let object = Object::wrap(reference, &runtime.class_loader);
							let value = object.get_dyn_field(&self.field_name);
							frame.push(StackValue::from_dyn(value));
						}
						FieldInstKind::Put => {
							let value = frame.pop();
							let reference = frame.pop().to_ref();

							let object = Object::wrap(reference, &runtime.class_loader);
							object.set_dyn_field(&self.field_name, value.to_dyn());
						}
					}
				}
			}
			_ => {
				panic!()
			}
		}
	}
}
