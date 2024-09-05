use rvm_core::{Id, PrimitiveType, StorageValue, Type};

use crate::object::array::ArrayClass;
use crate::object::instance::InstanceClass;
use crate::Runtime;

pub enum Class {
	Instance(InstanceClass),
	Array(ArrayClass),
	Primitive(PrimitiveType),
}

impl Class {
	pub fn is_instance(&self) -> bool {
		matches!(self, Class::Instance(_))
	}

	pub fn as_array(&self) -> Option<&ArrayClass> {
		if let Self::Array(class) = self {
			return Some(class);
		}
		None
	}

	pub fn as_instance(&self) -> Option<&InstanceClass> {
		if let Self::Instance(class) = self {
			return Some(class);
		}
		None
	}

	pub fn set_id(&mut self, id: Id<Class>) {
		match self {
			Class::Instance(object) => {
				object.id = id;
			}
			Class::Array(object) => {
				object.id = id;
			}
			Class::Primitive(_) => {}
		}
	}

	pub fn id(&self) -> Id<Class> {
		match self {
			Class::Instance(class) => class.id,
			Class::Array(class) => class.id,
			Class::Primitive(_) => todo!(),
		}
	}

	pub fn cloned_ty(&self) -> Type {
		match &self {
			Class::Instance(object) => Type::Object(object.ty.clone()),
			_ => todo!(),
		}
	}
}

impl From<PrimitiveType> for Class {
	fn from(value: PrimitiveType) -> Self {
		Class::Primitive(value)
	}
}
impl StorageValue for Class {
	type Idx = u32;
}
