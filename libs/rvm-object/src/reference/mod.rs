use crate::value::Value;
use crate::{Class, DynValue, Field, ObjectClass, ObjectFieldLayout};
use rvm_core::{Reference, ReferenceKind, StorageValue};
use std::mem::size_of;
use std::ops::Deref;

mod array;
mod class;

pub use array::*;
pub use class::*;
pub enum Object {
	Class(AnyClassObject),
	Array(AnyArrayObject),
}

impl Object {
	pub const HEADER_SIZE: usize = size_of::<u8>();
	pub fn new(reference: Reference) -> Object {
		match reference.kind() {
			ReferenceKind::Class => Object::Class(AnyClassObject { reference }),
			ReferenceKind::Array => Object::Array(AnyArrayObject { reference }),
		}
	}

	pub fn as_class(&self) -> Option<&AnyClassObject> {
		match self {
			Object::Class(class) => Some(class),
			Object::Array(_) => None,
		}
	}

	pub fn as_array(&self) -> Option<&AnyArrayObject> {
		match self {
			Object::Array(array) => Some(array),
			Object::Class(_) => None,
		}
	}

	pub fn visit_refs(&self, mut visitor: impl FnMut(Reference)) {
		match self {
			Object::Class(raw) => raw.visit_refs(visitor),
			Object::Array(raw) => raw.visit_refs(visitor),
		}
	}

	pub fn map_refs(&self, mut mapper: impl FnMut(Reference) -> Reference) {
		match self {
			Object::Class(raw) => raw.map_refs(mapper),
			Object::Array(raw) => raw.map_refs(mapper),
		}
	}
}
