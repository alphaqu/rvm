mod array;
mod field;
mod method;
mod object;

use rvm_core::{PrimitiveType, StorageValue};

use std::ptr::{read, write};

use crate::Ref;
pub use array::Array;
pub use array::ArrayClass;
pub use field::ClassFieldManager;
pub use method::ClassMethodManager;
pub use object::Object;
pub use object::ObjectClass;

pub struct Class {
	pub name: String,
	pub kind: ClassKind,
}

impl Class {}

pub enum ClassKind {
	Object(ObjectClass),
	Array(ArrayClass),
	Primitive(PrimitiveType),
}

impl ClassKind {
	/// Gets the object size excluding the header
	pub fn obj_size(&self, object: Ref) -> usize {
		match self {
			ClassKind::Object(class) => class.size(false),
			ClassKind::Array(class) => class.size(object),
			ClassKind::Primitive(desc) => {
				panic!("no")
			}
		}
	}
}

impl StorageValue for Class {
	type Idx = u32;
}

#[inline(always)]
unsafe fn read_arr<const C: usize>(ptr: *mut u8) -> [u8; C] {
	let mut out = [0; C];
	for i in 0..C {
		*out.get_unchecked_mut(i) = read(ptr.add(i));
	}
	out
}

#[inline(always)]
unsafe fn write_arr<const C: usize>(ptr: *mut u8, value: [u8; C]) {
	for i in 0..C {
		write(ptr.add(i), *value.get_unchecked(i));
	}
}
