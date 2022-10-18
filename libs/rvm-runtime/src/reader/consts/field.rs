use crate::consts::class::ClassConst;
use crate::consts::{Constant, ConstantInfo, ConstPtr};
use crate::consts::method::MethodConst;
use crate::consts::name_and_type::NameAndTypeConst;

#[derive(Copy, Clone)]
pub struct FieldConst {
	pub class: ConstPtr<ClassConst>,
	pub name_and_type: ConstPtr<NameAndTypeConst>
}

impl Constant for FieldConst {
	fn get(value: &ConstantInfo) -> &Self {
		if let ConstantInfo::Field(v) = value {
			return v;
		}
		panic!("Wrong type")
	}
}