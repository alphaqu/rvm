use std::fmt::{Display, Formatter};
use std::ops::Add;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Rem;
use std::ops::Sub;

use rvm_core::PrimitiveType;
use rvm_reader::MathInst;

use crate::thread::ThreadFrame;
use crate::value::StackValue;

#[derive(Debug)]
pub struct CombineTask {
	pub ty: CombineTaskType,
	pub op: CombineTaskOperation,
}

impl Display for CombineTask {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self.op {
			CombineTaskOperation::Add => f.write_str("ADD"),
			CombineTaskOperation::Sub => f.write_str("SUB"),
			CombineTaskOperation::Div => f.write_str("DIV"),
			CombineTaskOperation::Mul => f.write_str("MUL"),
			CombineTaskOperation::Rem => f.write_str("REM"),
			CombineTaskOperation::And => f.write_str("AND"),
			CombineTaskOperation::Or => f.write_str("OR"),
			CombineTaskOperation::Xor => f.write_str("XOR"),
			CombineTaskOperation::Shl => f.write_str("SHL"),
			CombineTaskOperation::Shr => f.write_str("SHR"),
			CombineTaskOperation::UShr => f.write_str("USHR"),
			CombineTaskOperation::FCMPG => f.write_str("FMCPG"),
			CombineTaskOperation::FCMPL => f.write_str("FCMPL"),
			CombineTaskOperation::ICMP => f.write_str("ICMP"),
		}
	}
}

impl CombineTask {
	pub fn new(inst: &MathInst) -> CombineTask {
		match inst {
			MathInst::Add(v) => CombineTask {
				ty: CombineTaskType::new(*v),
				op: CombineTaskOperation::Add,
			},
			MathInst::Sub(v) => CombineTask {
				ty: CombineTaskType::new(*v),
				op: CombineTaskOperation::Sub,
			},
			MathInst::Div(v) => CombineTask {
				ty: CombineTaskType::new(*v),
				op: CombineTaskOperation::Div,
			},
			MathInst::Mul(v) => CombineTask {
				ty: CombineTaskType::new(*v),
				op: CombineTaskOperation::Mul,
			},
			MathInst::Rem(v) => CombineTask {
				ty: CombineTaskType::new(*v),
				op: CombineTaskOperation::Rem,
			},
			MathInst::And(v) => CombineTask {
				ty: CombineTaskType::new(*v),
				op: CombineTaskOperation::And,
			},
			MathInst::Or(v) => CombineTask {
				ty: CombineTaskType::new(*v),
				op: CombineTaskOperation::Or,
			},
			MathInst::Xor(v) => CombineTask {
				ty: CombineTaskType::new(*v),
				op: CombineTaskOperation::Xor,
			},
			MathInst::Shl(v) => CombineTask {
				ty: CombineTaskType::new(*v),
				op: CombineTaskOperation::Shl,
			},
			MathInst::Shr(v) => CombineTask {
				ty: CombineTaskType::new(*v),
				op: CombineTaskOperation::Shr,
			},
			MathInst::Ushr(v) => CombineTask {
				ty: CombineTaskType::new(*v),
				op: CombineTaskOperation::UShr,
			},
			_ => unreachable!(),
		}
	}

	pub fn exec(&self, frame: &mut ThreadFrame) {
		/// According to all known laws of aviation, there is no way a bee should be able to fly.
		/// It's wings are too small to get its fat little body off the ground.
		/// The bee, of course, flies anyway, because bees don't care what humans think is impossible.
		macro_rules! exciting_macro {
			($v0:ident, $v1:ident, $TY:ident, $OP:expr) => {
				match ($v0, $v1) {
					(StackValue::$TY($v0), StackValue::$TY($v1)) => StackValue::$TY($OP),
					(_, StackValue::$TY(_)) => {
						panic!("value1 in stack is not an {}", stringify!($TY))
					}
					(StackValue::$TY(_), _) => {
						panic!("value2 in stack is not an {}", stringify!($TY))
					}
					_ => {
						panic!("both stack values is not an {}", stringify!($TY))
					}
				}
			};
		}

		macro_rules! electric_boogaloo {
			($frame:ident, $v:ident, $v0:ident, $v1:ident, $OP:expr, $FOP:expr) => {
				match $v.ty {
					CombineTaskType::Int => {
						$frame.push(exciting_macro!($v0, $v1, Int, $OP));
					}
					CombineTaskType::Long => {
						$frame.push(exciting_macro!($v0, $v1, Long, $OP));
					}
					CombineTaskType::Float => {
						$frame.push(exciting_macro!($v0, $v1, Float, $FOP));
					}
					CombineTaskType::Double => {
						$frame.push(exciting_macro!($v0, $v1, Double, $FOP));
					}
				}
			};
		}

		let v1 = frame.pop();
		let v0 = frame.pop();

		match self.op {
			CombineTaskOperation::Add => {
				electric_boogaloo!(frame, self, v0, v1, v0.overflowing_add(v1).0, v0.add(v1));
			}
			CombineTaskOperation::Sub => {
				electric_boogaloo!(frame, self, v0, v1, v0.overflowing_sub(v1).0, v0.sub(v1));
			}
			CombineTaskOperation::Div => {
				electric_boogaloo!(frame, self, v0, v1, v0.overflowing_div(v1).0, v0.div(v1));
			}
			CombineTaskOperation::Mul => {
				electric_boogaloo!(frame, self, v0, v1, v0.overflowing_mul(v1).0, v0.mul(v1));
			}
			CombineTaskOperation::Rem => {
				electric_boogaloo!(frame, self, v0, v1, v0.overflowing_rem(v1).0, v0.rem(v1));
			}
			_ => {
				todo!()
			}
		}
	}
}

#[derive(Debug)]
pub enum CombineTaskOperation {
	Add,
	Sub,
	Div,
	Mul,
	Rem,
	And,
	Or,
	Xor,
	Shl,
	Shr,
	UShr,
	// Compare
	FCMPG,
	FCMPL,
	ICMP,
}

#[derive(Debug)]
pub enum CombineTaskType {
	Int,
	Long,
	Float,
	Double,
}

impl CombineTaskType {
	pub fn new(ty: PrimitiveType) -> CombineTaskType {
		match ty {
			PrimitiveType::Int => CombineTaskType::Int,
			PrimitiveType::Long => CombineTaskType::Long,
			PrimitiveType::Float => CombineTaskType::Float,
			PrimitiveType::Double => CombineTaskType::Double,
			_ => panic!("invalid cast"),
		}
	}
}
