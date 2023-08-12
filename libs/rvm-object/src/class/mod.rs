pub use array::*;
pub use object::*;
use rvm_core::{PrimitiveType, StorageValue, Type};

mod array;
mod object;

pub enum Class {
	Object(ObjectClass),
	Array(ArrayClass),
	Primitive(PrimitiveType),
}

impl Class {
	pub fn object(&self) -> Option<&ObjectClass> {
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
	///// Gets the object size excluding the header
	//pub fn obj_size(&self, object: Ref) -> usize {
	//    match self {
	//        ClassKind::Object(class) => class.size(false),
	//        ClassKind::Array(class) => class.size(object),
	//        ClassKind::Primitive(desc) => {
	//            panic!("no")
	//        }
	//    }
	//}
}

impl StorageValue for Class {
	type Idx = u32;
}
