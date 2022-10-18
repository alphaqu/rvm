use std::cell::{Cell, RefCell};
use std::mem::forget;
use tracing::debug;
use rvm_core::Id;
use crate::{Class, ConstantPool, impl_constant, JResult, Runtime, ValueDesc};
use crate::reader::consts::{ConstPtr};
use crate::reader::consts::utf_8::UTF8Const;

#[derive(Clone)]
pub struct ClassConst {
	pub name: ConstPtr<UTF8Const>,
	pub link: Cell<Option<Id<Class>>>
}

impl_constant!(Class ClassConst);
