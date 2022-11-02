use crate::reader::consts::class::ClassConst;
use crate::reader::consts::name_and_type::NameAndTypeConst;
use crate::reader::consts::utf_8::UTF8Const;
use crate::reader::consts::ConstPtr;
use crate::{impl_constant, Method};
use rvm_core::Id;
use std::cell::Cell;

#[derive(Clone, Debug)]
pub struct MethodConst {
	pub class: ConstPtr<ClassConst>,
	pub name_and_type: ConstPtr<NameAndTypeConst>,
	pub link: Cell<Option<Id<Method>>>,
}

#[derive(Copy, Clone, Debug)]
pub struct MethodHandleConst {
	pub reference_kind: u8,
	pub reference_index: u16,
}

#[derive(Copy, Clone, Debug)]
pub struct MethodTypeConst {
	pub descriptor: ConstPtr<UTF8Const>,
}

impl_constant!(Method MethodConst);
impl_constant!(MethodHandle MethodHandleConst);
impl_constant!(MethodType MethodTypeConst);
