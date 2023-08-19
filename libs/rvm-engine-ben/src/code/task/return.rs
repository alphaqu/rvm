use std::fmt::{Display, Formatter, Write};

use rvm_core::StackKind;
use rvm_reader::ReturnInst;

#[derive(Debug)]
pub struct ReturnTask {
	pub kind: Option<StackKind>,
}

impl ReturnTask {
	pub fn new(inst: &ReturnInst) -> ReturnTask {
		ReturnTask {
			kind: inst.value.clone(),
		}
	}
}

impl Display for ReturnTask {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "RETURN ", )?;
		match self.kind {
			None => {
				write!(f, "void")
			}
			Some(value) => value.fmt(f),
		}
	}
}
