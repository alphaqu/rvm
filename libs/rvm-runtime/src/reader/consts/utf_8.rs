use std::ops::Deref;
use crate::consts::{Constant, ConstantInfo};

#[derive(Clone)]
pub struct UTF8Const(pub(crate) String);

impl Deref for UTF8Const {
	type Target = String;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl Constant for UTF8Const {
	fn get(value: &ConstantInfo) -> &Self {
		if let ConstantInfo::UTF8(v) = value {
			return v;
		}
		panic!("Wrong type")
	}
}