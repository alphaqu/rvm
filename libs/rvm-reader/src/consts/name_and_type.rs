use crate::impl_constant;
use crate::consts::utf_8::UTF8Const;
use crate::consts::ConstPtr;

#[derive(Copy, Clone, Debug)]
pub struct NameAndTypeConst {
	pub name: ConstPtr<UTF8Const>,
	pub descriptor: ConstPtr<UTF8Const>,
}

impl_constant!(NameAndType NameAndTypeConst);
