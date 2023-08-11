use crate::thread::ThreadFrame;
use crate::value::StackValue;
use rvm_object::ObjectClass;
use rvm_reader::{ConstInst, ConstantInfo};
#[derive(Debug)]

pub enum ConstTask {
	Null,
	Int(i32),
	Long(i64),
	Float(f32),
	Double(f64),
}

impl ConstTask {
	pub fn new(inst: &ConstInst, class: &ObjectClass) -> ConstTask {
		match inst {
			ConstInst::Null => ConstTask::Null,
			ConstInst::Int(v) => ConstTask::Int(*v),
			ConstInst::Long(v) => ConstTask::Long(*v),
			ConstInst::Float(v) => ConstTask::Float(*v),
			ConstInst::Double(v) => ConstTask::Double(*v),
			ConstInst::Ldc { id, cat2 } => {
				let info = class.cp.get_raw(*id).unwrap();
				match info {
					ConstantInfo::Integer(value) => ConstTask::Int(value.bytes),
					ConstantInfo::Float(value) => ConstTask::Float(value.bytes),
					ConstantInfo::Long(value) => ConstTask::Long(value.bytes),
					ConstantInfo::Double(value) => ConstTask::Double(value.bytes),
					_ => {
						panic!();
					}
				}
			}
		}
	}

	pub fn exec(&self, frame: &mut ThreadFrame) {
		match self {
			ConstTask::Null => {
				frame.push(StackValue::Reference(0));
			}
			ConstTask::Int(v) => frame.push(StackValue::Int(*v)),
			ConstTask::Long(v) => frame.push(StackValue::Long(*v)),
			ConstTask::Float(v) => frame.push(StackValue::Float(*v)),
			ConstTask::Double(v) => frame.push(StackValue::Double(*v)),
		}
	}
}
