use rvm_core::StackKind;
use rvm_reader::LocalInst;

use crate::thread::ThreadFrame;
use crate::value::StackValue;

pub struct LocalTask {
	pub kind: LocalTaskKind,
	pub ty: StackKind,
	pub idx: u16,
}
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
				let value = match self.ty {
					StackKind::Int => StackValue::Int(frame.load::<i32>(idx)),
					StackKind::Long => StackValue::Long(frame.load::<i64>(idx)),
					StackKind::Float => StackValue::Float(frame.load::<f32>(idx)),
					StackKind::Double => StackValue::Double(frame.load::<f64>(idx)),
					StackKind::Reference => {
						todo!()
					}
				};
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

				match value {
					StackValue::Int(val) => frame.store(idx, val),
					StackValue::Float(val) => frame.store(idx, val),
					StackValue::Long(val) => frame.store(idx, val),
					StackValue::Double(val) => frame.store(idx, val),
					StackValue::Reference(v) => todo!(),
				}
			}
		}
	}
}
