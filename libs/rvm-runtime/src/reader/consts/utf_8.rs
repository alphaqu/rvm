use std::ops::Deref;
use crate::impl_constant;
use crate::reader::consts::{Constant, ConstantInfo};

#[derive(Clone)]
pub struct UTF8Const(pub(crate) String);

impl Deref for UTF8Const {
	type Target = String;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl_constant!(UTF8 UTF8Const);