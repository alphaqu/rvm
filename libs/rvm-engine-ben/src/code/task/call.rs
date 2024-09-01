use std::fmt::{Display, Formatter};

use rvm_core::ObjectType;
use rvm_reader::{ConstPtr, InterfaceConst, InvokeInst, InvokeInstKind};
use rvm_runtime::{InstanceClass, MethodIdentifier};

#[derive(Debug, Clone)]
pub struct CallTask {
	pub method: MethodIdentifier,
	pub object: ObjectType,
	pub ty: CallType,
}

impl Display for CallTask {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"call {:?} {:?}.{:?}()",
			self.ty, self.object, self.method
		)
	}
}

impl CallTask {
	pub fn new(inst: &InvokeInst, class: &InstanceClass) -> CallTask {
		let (name_and_type, name, is_interface) = match inst.value.get(&class.cp) {
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

		CallTask {
			method: MethodIdentifier::new(name_and_type, &class.cp),
			object: ObjectType(name.to_string()),
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

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum CallType {
	Virtual,
	Static,
	Special,
	Interface,
}

impl CallType {
	pub fn is_static(&self) -> bool {
		matches!(self, CallType::Static)
	}
	pub fn is_special(&self) -> bool {
		matches!(self, CallType::Special)
	}
	pub fn is_interface(&self) -> bool {
		matches!(self, CallType::Interface)
	}
}
