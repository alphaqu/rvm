use crate::impl_constant;
use crate::reader::consts::{ConstPtr};
use crate::reader::consts::utf_8::UTF8Const;

#[derive(Copy, Clone)]
pub struct NameAndTypeConst {
	pub name: ConstPtr<UTF8Const>,
	pub descriptor: ConstPtr<UTF8Const>,
}


impl_constant!(NameAndType NameAndTypeConst);