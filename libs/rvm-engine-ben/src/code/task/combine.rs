use num_traits::{Bounded, PrimInt, Signed};
use std::fmt::{Display, Formatter};
use std::ops::Div;
use std::ops::Mul;
use std::ops::Rem;
use std::ops::Sub;
use std::ops::{Add, Shl, Shr};

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
		let task = CombineTaskType::new(inst.ty());
		let operation = match inst {
			MathInst::Add(_) => CombineTaskOperation::Add,
			MathInst::Sub(_) => CombineTaskOperation::Sub,
			MathInst::Div(_) => CombineTaskOperation::Div,
			MathInst::Mul(_) => CombineTaskOperation::Mul,
			MathInst::Rem(_) => CombineTaskOperation::Rem,
			MathInst::And(_) => CombineTaskOperation::And,
			MathInst::Or(_) => CombineTaskOperation::Or,
			MathInst::Xor(_) => CombineTaskOperation::Xor,
			MathInst::Shl(_) => CombineTaskOperation::Shl,
			MathInst::Shr(_) => CombineTaskOperation::Shr,
			_ => unreachable!(),
		};

		CombineTask {
			ty: task,
			op: operation,
		}
	}

	pub fn exec(&self, frame: &mut ThreadFrame) {
		macro_rules! impl_for_type {
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

		macro_rules! impl_for_types {
			($frame:ident, $task:ident, $v0:ident, $v1:ident, $OP:expr, $FOP:expr) => {
				match $task.ty {
					CombineTaskType::Int => {
						$frame.push(impl_for_type!($v0, $v1, Int, $OP));
					}
					CombineTaskType::Long => {
						$frame.push(impl_for_type!($v0, $v1, Long, $OP));
					}
					CombineTaskType::Float => {
						$frame.push(impl_for_type!($v0, $v1, Float, $FOP));
					}
					CombineTaskType::Double => {
						$frame.push(impl_for_type!($v0, $v1, Double, $FOP));
					}
				}
			};
		}

		let v1 = frame.pop();
		let v0 = frame.pop();

		fn java_div<V: Signed + Bounded>(v0: V, v1: V) -> V {
			if v1 == V::zero() {
				panic!("Division by 0");
			}
			if v1 == V::one().neg() && v0 == V::min_value() {
				return v0;
			}

			V::div(v0, v1)
		}

		match &self.op {
			CombineTaskOperation::Add => {
				impl_for_types!(frame, self, v0, v1, v0.wrapping_add(v1), v0.add(v1));
			}
			CombineTaskOperation::Sub => {
				impl_for_types!(frame, self, v0, v1, v0.wrapping_sub(v1), v0.sub(v1));
			}
			CombineTaskOperation::Div => {
				impl_for_types!(frame, self, v0, v1, java_div(v0, v1), v0.div(v1));
			}
			CombineTaskOperation::Mul => {
				impl_for_types!(frame, self, v0, v1, v0.wrapping_mul(v1), v0.mul(v1));
			}
			CombineTaskOperation::Rem => {
				impl_for_types!(frame, self, v0, v1, v0.wrapping_rem(v1), v0.rem(v1));
			}

			task => {
				todo!("{task:?}")
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
