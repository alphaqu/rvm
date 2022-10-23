use crate::object::value::ValueType;
use crate::reader::ValueDesc;
use rvm_consts::FieldAccessFlags;

pub struct Field {
	pub offset: u32,
	pub flags: FieldAccessFlags,
	pub desc: ValueDesc,
	pub ty: ValueType,
}
