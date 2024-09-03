use std::fmt::{Display, Formatter};

use rvm_reader::{ConstInst, ConstantInfo};
use rvm_runtime::{InstanceClass, Reference};

use crate::thread::ThreadFrame;
use crate::value::StackValue;

#[derive(Debug)]
pub enum ConstTask {
	Null,
	Int(i32),
	Long(i64),
	Float(f32),
	Double(f64),
}

impl Display for ConstTask {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"CONST {}",
			match self {
				ConstTask::Null => "null".to_string(),
				ConstTask::Int(v) => v.to_string(),
				ConstTask::Long(v) => v.to_string(),
				ConstTask::Float(v) => v.to_string(),
				ConstTask::Double(v) => v.to_string(),
			}
		)
	}
}

impl ConstTask {
	pub fn new(inst: &ConstInst, class: &InstanceClass) -> ConstTask {
		match inst {
			ConstInst::Null => ConstTask::Null,
			ConstInst::Int(v) => ConstTask::Int(*v),
			ConstInst::Long(v) => ConstTask::Long(*v),
			ConstInst::Float(v) => ConstTask::Float(*v),
			ConstInst::Double(v) => ConstTask::Double(*v),
			ConstInst::Ldc { id, cat2: _ } => {
				let info = class.cp.raw_get(*id).unwrap();
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

	#[inline(always)]
	pub fn exec(&self, frame: &mut ThreadFrame) {
		match self {
			ConstTask::Null => {
				frame.push(StackValue::Reference(Reference::NULL));
			}
			ConstTask::Int(v) => frame.push(StackValue::Int(*v)),
			ConstTask::Long(v) => frame.push(StackValue::Long(*v)),
			ConstTask::Float(v) => frame.push(StackValue::Float(*v)),
			ConstTask::Double(v) => frame.push(StackValue::Double(*v)),
		}
	}
}
