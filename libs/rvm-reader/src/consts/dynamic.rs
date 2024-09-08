use crate::consts::class::ClassConst;
use crate::consts::name_and_type::NameAndTypeConst;
use crate::consts::utf_8::UTF8Const;
use crate::consts::ConstPtr;
use crate::impl_constant;

#[derive(Clone, Debug)]
pub struct InvokeDynamicConst {
	pub bootstrap_method_attr_index: u16,
	pub name_and_type: ConstPtr<NameAndTypeConst>,
}

#[derive(Copy, Clone, Debug)]
pub struct DynamicConst {
	pub bootstrap_method_attr_index: u16,
	pub name_and_type: ConstPtr<NameAndTypeConst>,
}

impl_constant!(InvokeDynamic InvokeDynamicConst);
impl_constant!(Dynamic DynamicConst);
