use crate::consts::class::ClassConst;
use crate::consts::ConstPtr;
use crate::consts::name_and_type::NameAndTypeConst;
use crate::impl_constant;

#[derive(Copy, Clone, Debug)]
pub struct InterfaceConst {
	pub class: ConstPtr<ClassConst>,
	pub name_and_type: ConstPtr<NameAndTypeConst>,
}

impl_constant!(Interface InterfaceConst);
