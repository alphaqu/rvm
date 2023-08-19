use crate::object::array::ArrayClass;
use crate::object::instance::InstanceClass;
use rvm_core::{PrimitiveType, StorageValue, Type};

pub enum Class {
	Object(InstanceClass),
	Array(ArrayClass),
	Primitive(PrimitiveType),
}

impl Class {
	pub fn object(&self) -> Option<&InstanceClass> {
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
