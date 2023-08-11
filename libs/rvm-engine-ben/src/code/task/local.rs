use rvm_core::StackKind;
use rvm_reader::LocalInst;

use crate::thread::ThreadFrame;
use crate::value::StackValue;
#[derive(Debug)]

pub struct LocalTask {
	pub kind: LocalTaskKind,
	pub ty: StackKind,
	pub idx: u16,
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
				let value = frame.load_dyn(idx, self.ty);
				frame.push(value);
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

				frame.store_dyn(idx, value);
			}
		}
	}
}
