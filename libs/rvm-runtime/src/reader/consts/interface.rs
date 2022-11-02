use crate::impl_constant;
use crate::reader::consts::class::ClassConst;
use crate::reader::consts::name_and_type::NameAndTypeConst;
use crate::reader::consts::ConstPtr;

#[derive(Copy, Clone, Debug)]
pub struct InterfaceConst {
	pub class: ConstPtr<ClassConst>,
	pub name_and_type: ConstPtr<NameAndTypeConst>,
}

impl_constant!(Interface InterfaceConst);
