use std::fmt::{Display, Formatter};

use rvm_core::{MethodDescriptor, ObjectType};
use rvm_reader::{ConstPtr, InterfaceConst, InvokeInst, InvokeInstKind};
use rvm_runtime::{CallType, InstanceClass, MethodIdentifier};

#[derive(Debug, Clone)]
pub struct CallTask {
	pub method: MethodIdentifier,
	pub method_descriptor: MethodDescriptor,
	pub object: ObjectType,
	pub ty: CallType,
}

impl Display for CallTask {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"call {:?} {:?}.{}{}()",
			self.ty, self.object, self.method.name, self.method.descriptor
		)
	}
}

impl CallTask {
	pub fn new(inst: &InvokeInst, class: &InstanceClass) -> CallTask {
		let (name_and_type, name, _is_interface) = match inst.value.get(&class.cp) {
			Some(method) => {
				let name_and_type = method.name_and_type.get(&class.cp).unwrap();
				let target = method.class.get(&class.cp).unwrap();
				let name = target.name.get(&class.cp).unwrap();

				(name_and_type, name, false)
			}
			None => {
				let method = class
					.cp
					.get(ConstPtr::<InterfaceConst>::new(inst.value.id()))
					.unwrap();
				let name_and_type = method.name_and_type.get(&class.cp).unwrap();
				let target = method.class.get(&class.cp).unwrap();
				let name = target.name.get(&class.cp).unwrap();

				(name_and_type, name, true)
			}
		};

		let identifier = MethodIdentifier::new(name_and_type, &class.cp);
		CallTask {
			method_descriptor: MethodDescriptor::parse(&identifier.descriptor).unwrap(),
			method: identifier,
			object: ObjectType::new(name.to_string()),
			ty: match inst.kind {
				InvokeInstKind::Dynamic => todo!(),
				InvokeInstKind::Interface(_) => CallType::Interface,
				InvokeInstKind::Special => CallType::Special,
				InvokeInstKind::Static => CallType::Static,
				InvokeInstKind::Virtual => CallType::Virtual,
			},
		}
	}
}
