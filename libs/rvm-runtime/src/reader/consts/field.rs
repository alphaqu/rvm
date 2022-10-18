use std::cell::Cell;
use rvm_core::Id;
use crate::{ConstantPool, Field, impl_constant, JResult, Runtime};
use crate::reader::consts::class::ClassConst;
use crate::reader::consts::{ConstPtr};
use crate::reader::consts::name_and_type::NameAndTypeConst;

#[derive(Clone)]
pub struct FieldConst {
	pub class: ConstPtr<ClassConst>,
	pub name_and_type: ConstPtr<NameAndTypeConst>,
	pub link: Cell<Option<Id<Field>>>
}

impl_constant!(Field FieldConst);
