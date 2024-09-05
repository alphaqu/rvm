use rvm_core::{ObjectType, Type};
use rvm_reader::{FieldInst, FieldInstKind};
use rvm_runtime::{AnyInstance, Class, InstanceClass, Runtime};
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use crate::thread::ThreadFrame;
use crate::value::StackValue;

#[derive(Debug)]
pub struct FieldTask {
	pub source: ObjectType,
	pub field_name: String,
	pub instance: bool,
	pub kind: FieldInstKind,
}

impl Display for FieldTask {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match (self.kind, self.instance) {
			(FieldInstKind::Get, true) => f.write_str("GETFIELD "),
			(FieldInstKind::Get, false) => f.write_str("GETSTATIC "),
			(FieldInstKind::Put, true) => f.write_str("PUTFIELD "),
			(FieldInstKind::Put, false) => f.write_str("PUTSTATIC "),
		}?;

		write!(f, "{}.{}", self.source, self.field_name)
	}
}

impl FieldTask {
	pub fn new(inst: &FieldInst, class: &InstanceClass) -> FieldTask {
		let field = inst.value.get(&class.cp).unwrap();
		let source = field.class.get(&class.cp).unwrap();
		let source = source.name.get(&class.cp).unwrap();
		let field = field.name_and_type.get(&class.cp).unwrap();
		let field_name = field.name.get(&class.cp).unwrap();
		let _field_descriptor = field.descriptor.get(&class.cp).unwrap();
		FieldTask {
			source: ObjectType::new(source.to_string()),
			field_name: field_name.to_string(),
			instance: inst.instance,
			kind: inst.kind,
		}
	}

	#[inline(always)]
	pub fn exec(&self, runtime: &Runtime, frame: &mut ThreadFrame) -> eyre::Result<()> {
		let id = runtime.resolve_class(&Type::Object(self.source.clone()))?;
		let arc = runtime.classes.get(id);
		match &*arc {
			Class::Instance(object) => {
				if self.instance {
					let id = object.field_layout.get_id(&self.field_name).unwrap();

					match self.kind {
						FieldInstKind::Get => {
							let reference = frame.pop().to_ref()?;

							let class = reference.to_instance().unwrap();
							let instance = AnyInstance::try_new(runtime.clone(), class).unwrap();
							let fields = instance.fields();

							let value = fields.field(id).get();

							frame.push(StackValue::from_any(value));
						}
						FieldInstKind::Put => {
							let value = frame.pop();
							let reference = frame.pop().to_ref()?;
							let class = reference.to_instance().unwrap();
							let instance = AnyInstance::try_new(runtime.clone(), class).unwrap();
							let fields = instance.fields();
							fields.field(id).set(value.to_any());
						}
					}
				} else {
					let fields = object.static_fields();
					let field = fields.field_named(&self.field_name).unwrap();

					match self.kind {
						FieldInstKind::Get => {
							let value = field.get();
							frame.push(StackValue::from_any(value));
						}
						FieldInstKind::Put => {
							let value = frame.pop();
							field.set(value.to_any());
						}
					}
				}
			}
			_ => {
				panic!()
			}
		}
		Ok(())
	}
}
