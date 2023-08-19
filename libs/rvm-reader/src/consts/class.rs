use crate::impl_constant;
use crate::consts::ConstPtr;
use crate::consts::utf_8::UTF8Const;

#[derive(Clone, Debug)]
pub struct ClassConst {
	pub name: ConstPtr<UTF8Const>,
}

impl_constant!(Class ClassConst);
