use crate::reader::consts::utf_8::UTF8Const;
use crate::reader::consts::ConstPtr;
use crate::{impl_constant, Class};
use rvm_core::Id;
use std::cell::Cell;

#[derive(Clone)]
pub struct ClassConst {
	pub name: ConstPtr<UTF8Const>,
	pub link: Cell<Option<Id<Class>>>,
}

impl_constant!(Class ClassConst);
