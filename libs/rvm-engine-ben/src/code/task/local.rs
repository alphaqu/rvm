use std::fmt::{Debug, Display, Formatter, Write};

use rvm_core::StackKind;
use rvm_reader::LocalInst;

use crate::thread::ThreadFrame;

#[derive(Debug)]
pub struct LocalTask {
	pub kind: LocalTaskKind,
	pub ty: StackKind,
	pub idx: u16,
}

impl Display for LocalTask {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self.kind {
			LocalTaskKind::Load => f.write_str("LOAD "),
			LocalTaskKind::Store => f.write_str("STORE "),
		}?;

		write!(f, "{}", self.idx)?;
		f.write_str(": ")?;
		write!(f, "{}", self.ty)
	}
}

#[derive(Debug)]
pub enum LocalTaskKind {
	Load,
	Store,
}

impl LocalTask {
	pub fn new(inst: &LocalInst) -> LocalTask {
		match inst {
			LocalInst::Load(ty, idx) => LocalTask {
				kind: LocalTaskKind::Load,
				ty: *ty,
				idx: *idx,
			},
			LocalInst::Store(ty, idx) => LocalTask {
				kind: LocalTaskKind::Store,
				ty: *ty,
				idx: *idx,
			},
			LocalInst::Increment(_, _) => unreachable!(),
		}
	}

	pub fn exec(&self, frame: &mut ThreadFrame) {
		let idx = self.idx;

		match self.kind {
			LocalTaskKind::Load => {
				let stack_value = frame.load(idx);
				if stack_value.kind() != self.ty {
					panic!(
						"Expected stack value {:?} but got {:?}",
						self.ty,
						stack_value.kind()
					)
				}
				frame.push(stack_value);
			}
			LocalTaskKind::Store => {
				let value = frame.pop();
				if value.kind() != self.ty {
					panic!(
						"Expected stack value {:?} but got {:?}",
						self.ty,
						value.kind()
					)
				}

				frame.store(idx, value);
			}
		}
	}
}
