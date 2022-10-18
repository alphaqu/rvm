use std::ops::Index;
use ahash::AHashMap;
use rvm_reader::{ConstantPool, FieldDescriptor, FieldInfo, FieldType};
use crate::object::Object;

pub struct Field {
	pub(crate) offset: u32,
	pub(crate) ty: FieldType,
}

#[repr(transparent)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct FieldId(pub(crate) u16);

#[derive(Debug)]
pub enum FieldValue {
	Boolean(bool),
	Byte(i8),
	Short(i16),
	Int(i32),
	Long(i64),
	Char(u16),
	Float(f32),
	Double(f64),
	Object(Object),
}

impl FieldValue {
	//pub fn ty(&self) -> FieldType {
	//	match self {
	//		FieldValue::Boolean(_) => FieldType::Boolean,
	//		FieldValue::Byte(_) => FieldType::Byte,
	//		FieldValue::Short(_) => FieldType::Short,
	//		FieldValue::Int(_) => FieldType::Int,
	//		FieldValue::Long(_) => FieldType::Long,
	//		FieldValue::Char(_) => FieldType::Char,
	//		FieldValue::Float(_) => FieldType::Float,
	//		FieldValue::Double(_) => FieldType::Double,
	//		FieldValue::Object(_) => FieldType::Object,
	//	}
	//}
}