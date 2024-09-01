use rvm_core::{Id, PrimitiveType, StorageValue, Type};

use crate::object::array::ArrayClass;
use crate::object::instance::InstanceClass;

pub enum Class {
	Object(InstanceClass),
	Array(ArrayClass),
	Primitive(PrimitiveType),
}

impl Class {
	pub fn is_instance(&self) -> bool {
		matches!(self, Class::Object(_))
	}
	pub fn as_instance(&self) -> Option<&InstanceClass> {
		if let Self::Object(class) = self {
			return Some(class);
		}
		None
	}

	pub fn set_id(&mut self, id: Id<Class>) {
		match self {
			Class::Object(object) => {
				object.id = id;
			}
			Class::Array(_) => {}
			Class::Primitive(_) => {}
		}
	}

	pub fn cloned_ty(&self) -> Type {
		match &self {
			Class::Object(object) => Type::Object(object.ty.clone()),
			_ => todo!(),
		}
	}
}

impl StorageValue for Class {
	type Idx = u32;
}
