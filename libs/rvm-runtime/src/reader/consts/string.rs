use crate::impl_constant;
use crate::reader::consts::utf_8::UTF8Const;
use crate::reader::consts::ConstPtr;

#[derive(Copy, Clone, Debug)]
pub struct StringConst {
	pub string: ConstPtr<UTF8Const>,
}
impl_constant!(String StringConst);
