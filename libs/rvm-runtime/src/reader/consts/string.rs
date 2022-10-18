use crate::consts::{Constant, ConstantInfo, ConstPtr};
use crate::consts::name_and_type::NameAndTypeConst;
use crate::consts::utf_8::UTF8Const;

#[derive(Copy, Clone)]
pub struct StringConst {
	pub string: ConstPtr<UTF8Const>
}

impl Constant for StringConst {
	fn get(value: &ConstantInfo) -> &Self {
		if let ConstantInfo::String(v) = value {
			return v;
		}
		panic!("Wrong type")
	}
}