use crate::consts::utf_8::UTF8Const;
use crate::consts::ConstPtr;
use crate::impl_constant;

#[derive(Copy, Clone, Debug)]
pub struct StringConst {
	pub string: ConstPtr<UTF8Const>,
}
impl_constant!(String StringConst);
