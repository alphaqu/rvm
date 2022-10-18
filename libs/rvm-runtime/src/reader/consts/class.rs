use crate::consts::{Constant, ConstantInfo, ConstPtr};
use crate::consts::utf_8::UTF8Const;

#[derive(Copy, Clone)]
pub struct ClassConst {
	pub name: ConstPtr<UTF8Const>,
}

impl Constant for ClassConst {
	fn get(value: &ConstantInfo) -> &Self {
		if let ConstantInfo::Class(v) = value {
			return v;
		}
		panic!("Wrong type")
	}
}