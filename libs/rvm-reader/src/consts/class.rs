use crate::consts::utf_8::UTF8Const;
use crate::consts::ConstPtr;
use crate::impl_constant;

#[derive(Clone, Debug)]
pub struct ClassConst {
	pub name: ConstPtr<UTF8Const>,
}

impl_constant!(Class ClassConst);
