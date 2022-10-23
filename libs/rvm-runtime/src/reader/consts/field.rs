use crate::reader::consts::class::ClassConst;
use crate::reader::consts::name_and_type::NameAndTypeConst;
use crate::reader::consts::ConstPtr;
use crate::{impl_constant, Field};
use rvm_core::Id;
use std::cell::Cell;

#[derive(Clone)]
pub struct FieldConst {
	pub class: ConstPtr<ClassConst>,
	pub name_and_type: ConstPtr<NameAndTypeConst>,
	pub link: Cell<Option<Id<Field>>>,
}

impl_constant!(Field FieldConst);
