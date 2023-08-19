use rvm_core::{PrimitiveType, StorageValue, Type};

use crate::object::array::ArrayClass;
use crate::object::instance::InstanceClass;

pub enum Class {
	Object(InstanceClass),
	Array(ArrayClass),
	Primitive(PrimitiveType),
}

impl Class {
	pub fn as_instance(&self) -> Option<&InstanceClass> {
		if let Self::Object(class) = self {
			return Some(class);
		}
		None
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
