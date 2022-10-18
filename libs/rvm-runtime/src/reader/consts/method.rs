use crate::consts::class::ClassConst;
use crate::consts::{Constant, ConstantInfo, ConstPtr};
use crate::consts::name_and_type::NameAndTypeConst;
use crate::consts::utf_8::UTF8Const;

#[derive(Copy, Clone)]
pub struct MethodConst {
	pub class: ConstPtr<ClassConst>,
	pub name_and_type: ConstPtr<NameAndTypeConst>
}

#[derive(Copy, Clone)]
pub struct MethodHandleConst {
	pub reference_kind: u8,
	pub reference_index: u16,
}

#[derive(Copy, Clone)]
pub struct MethodTypeConst {
	pub descriptor: ConstPtr<UTF8Const>
}

impl Constant for MethodConst {
	fn get(value: &ConstantInfo) -> &Self {
		if let ConstantInfo::Method(v) = value {
			return v;
		}
		panic!("Wrong type")
	}
}

impl Constant for MethodHandleConst {
	fn get(value: &ConstantInfo) -> &Self {
		if let ConstantInfo::MethodHandle(v) = value {
			return v;
		}
		panic!("Wrong type")
	}
}

impl Constant for MethodTypeConst {
	fn get(value: &ConstantInfo) -> &Self {
		if let ConstantInfo::MethodType(v) = value {
			return v;
		}
		panic!("Wrong type")
	}
}