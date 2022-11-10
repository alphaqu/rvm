use rvm_core::{FieldAccessFlags, Type};

pub struct Field {
	pub offset: u32,
	pub flags: FieldAccessFlags,
	pub ty: Type,
}
