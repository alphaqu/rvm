use crate::consts::{Constant, ConstantInfo, ConstPtr};
use crate::consts::utf_8::UTF8Const;

#[derive(Copy, Clone)]
pub struct NameAndTypeConst {
	pub name: ConstPtr<UTF8Const>,
	pub descriptor: ConstPtr<UTF8Const>,
}

impl Constant for NameAndTypeConst {
	fn get(value: &ConstantInfo) -> &Self {
		if let ConstantInfo::NameAndType(v) = value {
			return v;
		}
		panic!("Wrong type")
	}
}