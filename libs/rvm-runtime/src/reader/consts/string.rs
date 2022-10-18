use crate::impl_constant;
use crate::reader::consts::{Constant, ConstantInfo, ConstPtr};
use crate::reader::consts::name_and_type::NameAndTypeConst;
use crate::reader::consts::utf_8::UTF8Const;

#[derive(Copy, Clone)]
pub struct StringConst {
	pub string: ConstPtr<UTF8Const>
}
impl_constant!(String StringConst);