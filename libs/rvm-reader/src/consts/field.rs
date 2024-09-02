use crate::consts::class::ClassConst;
use crate::consts::name_and_type::NameAndTypeConst;
use crate::consts::ConstPtr;
use crate::impl_constant;

#[derive(Clone, Debug)]
pub struct FieldConst {
	pub class: ConstPtr<ClassConst>,
	pub name_and_type: ConstPtr<NameAndTypeConst>,
}

impl_constant!(Field FieldConst);
