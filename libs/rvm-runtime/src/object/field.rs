use rvm_consts::FieldAccessFlags;
use crate::reader::ValueDesc;
use crate::object::value::ValueType;

pub struct Field {
    pub offset: u32,
    pub flags: FieldAccessFlags,
    pub desc: ValueDesc,
    pub ty: ValueType,
}