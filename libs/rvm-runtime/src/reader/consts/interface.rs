use crate::consts::class::ClassConst;
use crate::consts::{Constant, ConstantInfo, ConstPtr};
use crate::consts::method::MethodConst;
use crate::consts::name_and_type::NameAndTypeConst;

#[derive(Copy, Clone)]
pub struct InterfaceConst {
	pub class: ConstPtr<ClassConst>,
	pub name_and_type: ConstPtr<NameAndTypeConst>
}

impl Constant for InterfaceConst {
	fn get(value: &ConstantInfo) -> &Self {
		if let ConstantInfo::Interface(v) = value {
			return v;
		}
		panic!("Wrong type")
	}
}