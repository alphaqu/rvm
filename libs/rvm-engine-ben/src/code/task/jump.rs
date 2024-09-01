use crate::code::{JavaScope, ReturnTask};
use crate::thread::ThreadFrame;
use rvm_reader::{JumpInst, JumpKind};
use rvm_runtime::Reference;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct JumpTask {
	offset: i32,
	kind: JumpKind,
}

impl JumpTask {
	pub fn new(jump_inst: &JumpInst) -> JumpTask {
		JumpTask {
			offset: jump_inst.offset,
			kind: jump_inst.kind,
		}
	}
	pub fn exec(&self, scope: &mut JavaScope) {
		let frame = &mut scope.frame;
		let condition = match self.kind {
			JumpKind::IF_ICMPEQ | JumpKind::IF_ACMPEQ => {
				let value2 = frame.pop();
				let value1 = frame.pop();
				value1 == value2
			}
			JumpKind::IF_ICMPNE | JumpKind::IF_ACMPNE => {
				let value2 = frame.pop();
				let value1 = frame.pop();
				value1 != value2
			}
			JumpKind::IF_ICMPLT => {
				let value2 = frame.pop().to_int().unwrap();
				let value1 = frame.pop().to_int().unwrap();
				value1 < value2
			}
			JumpKind::IF_ICMPGE => {
				let value2 = frame.pop().to_int().unwrap();
				let value1 = frame.pop().to_int().unwrap();
				value1 >= value2
			}
			JumpKind::IF_ICMPGT => {
				let value2 = frame.pop().to_int().unwrap();
				let value1 = frame.pop().to_int().unwrap();
				value1 > value2
			}
			JumpKind::IF_ICMPLE => {
				let value2 = frame.pop().to_int().unwrap();
				let value1 = frame.pop().to_int().unwrap();
				value1 <= value2
			}
			JumpKind::IFEQ => {
				let value = frame.pop().to_int().unwrap();
				value == 0
			}
			JumpKind::IFNE => {
				let value = frame.pop().to_int().unwrap();
				value != 0
			}
			JumpKind::IFLT => {
				let value = frame.pop().to_int().unwrap();
				value < 0
			}
			JumpKind::IFGE => {
				let value = frame.pop().to_int().unwrap();
				value >= 0
			}
			JumpKind::IFGT => {
				let value = frame.pop().to_int().unwrap();
				value > 0
			}
			JumpKind::IFLE => {
				let value = frame.pop().to_int().unwrap();
				value <= 0
			}
			JumpKind::IFNONNULL => {
				let value = frame.pop().to_ref().unwrap();
				value != Reference::NULL
			}
			JumpKind::IFNULL => {
				let value = frame.pop().to_ref().unwrap();
				value == Reference::NULL
			}
			JumpKind::GOTO => true,
		};

		if condition {
			scope.cursor = scope
				.cursor
				.checked_add_signed(self.offset as isize)
				.unwrap();
		} else {
			scope.cursor += 1;
		}
	}
}

impl Display for JumpTask {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "JUMP {:?} -> {}", self.kind, self.offset)
	}
}
