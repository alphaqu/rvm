use crate::consts::utf_8::UTF8Const;
use crate::consts::ConstPtr;
use crate::{impl_constant};
use rvm_core::Id;
use std::cell::Cell;

#[derive(Clone, Debug)]
pub struct ClassConst {
	pub name: ConstPtr<UTF8Const>,
}

impl_constant!(Class ClassConst);
