use crate::compiler::compiler::BlockCompiler;
use crate::compiler::op::Task;
use crate::compiler::resolver::BlockResolver;
use crate::executor::Inst;
use crate::object::ValueType;
use inkwell::builder::Builder;
use inkwell::values::{BasicValue, BasicValueEnum};
use inkwell::{FloatPredicate, IntPredicate};
use std::fmt::{Display, Formatter};

/// Applies an operation on both values.
#[derive(Clone, Debug)]
pub struct CombineTask {
	pub kind: CombineKind,
}

impl CombineTask {
	pub fn resolve(inst: &Inst, resolver: &mut BlockResolver) -> CombineTask {
		let kind = match inst {
			Inst::DADD => CombineKind::ADD(true),
			Inst::DDIV => CombineKind::DIV(true),
			Inst::DMUL => CombineKind::MUL(true),
			Inst::DREM => CombineKind::REM(true),
			Inst::DSUB => CombineKind::SUB(true),
			Inst::FADD => CombineKind::ADD(true),
			Inst::FDIV => CombineKind::DIV(true),
			Inst::FMUL => CombineKind::MUL(true),
			Inst::FREM => CombineKind::REM(true),
			Inst::FSUB => CombineKind::SUB(true),
			Inst::IADD => CombineKind::ADD(false),
			Inst::IDIV => CombineKind::DIV(false),
			Inst::IMUL => CombineKind::MUL(false),
			Inst::IREM => CombineKind::REM(false),
			Inst::ISUB => CombineKind::SUB(false),
			Inst::LADD => CombineKind::ADD(false),
			Inst::LDIV => CombineKind::DIV(false),
			Inst::LMUL => CombineKind::MUL(false),
			Inst::LREM => CombineKind::REM(false),
			Inst::LSUB => CombineKind::SUB(false),
			Inst::IAND => CombineKind::AND,
			Inst::IOR => CombineKind::OR,
			Inst::ISHL => CombineKind::SHL,
			Inst::ISHR => CombineKind::SHR,
			Inst::IUSHR => CombineKind::USHR,
			Inst::IXOR => CombineKind::XOR,
			Inst::LAND => CombineKind::AND,
			Inst::LOR => CombineKind::OR,
			Inst::LSHL => CombineKind::SHL,
			Inst::LSHR => CombineKind::SHR,
			Inst::LUSHR => CombineKind::USHR,
			Inst::LXOR => CombineKind::XOR,
			Inst::FCMPG | Inst::DCMPG => CombineKind::FCMPG,
			Inst::LCMP => CombineKind::ICMP,
			Inst::FCMPL | Inst::DCMPL => CombineKind::FCMPL,
			_ => panic!("what"),
		};

		CombineTask {
			kind,
		}
	}

	pub fn compile<'b, 'a>(&self, bc: &mut BlockCompiler<'b, 'a>) {
		let right = bc.pop();
		let left = bc.pop();
		let name = bc.gen.next();

		let output = match self.kind {
			CombineKind::ADD(float) => {
				if float {
					let left = left.into_float_value();
					let right = right.into_float_value();
					bc.build_float_add(left, right, &name).as_basic_value_enum()
				} else {
					let left = left.into_int_value();
					let right = right.into_int_value();
					bc.build_int_add(left, right, &name).as_basic_value_enum()
				}
			}
			CombineKind::DIV(float) => {
				if float {
					let left = left.into_float_value();
					let right = right.into_float_value();
					bc.build_float_div(left, right, &name).as_basic_value_enum()
				} else {
					let left = left.into_int_value();
					let right = right.into_int_value();
					bc.build_int_signed_div(left, right, &name)
						.as_basic_value_enum()
				}
			}
			CombineKind::MUL(float) => {
				if float {
					let left = left.into_float_value();
					let right = right.into_float_value();
					bc.build_float_mul(left, right, &name).as_basic_value_enum()
				} else {
					let left = left.into_int_value();
					let right = right.into_int_value();
					bc.build_int_mul(left, right, &name).as_basic_value_enum()
				}
			}
			CombineKind::REM(float) => {
				if float {
					let left = left.into_float_value();
					let right = right.into_float_value();
					bc.build_float_rem(left, right, &name).as_basic_value_enum()
				} else {
					let left = left.into_int_value();
					let right = right.into_int_value();
					bc.build_int_signed_rem(left, right, &name)
						.as_basic_value_enum()
				}
			}
			CombineKind::SUB(float) => {
				if float {
					let left = left.into_float_value();
					let right = right.into_float_value();
					bc.build_float_sub(left, right, &name).as_basic_value_enum()
				} else {
					let left = left.into_int_value();
					let right = right.into_int_value();
					bc.build_int_sub(left, right, &name).as_basic_value_enum()
				}
			}
			CombineKind::AND => {
				let left = left.into_int_value();
				let right = right.into_int_value();
				bc.build_and(left, right, &name).as_basic_value_enum()
			}
			CombineKind::OR => {
				let left = left.into_int_value();
				let right = right.into_int_value();
				bc.build_or(left, right, &name).as_basic_value_enum()
			}
			CombineKind::SHL => {
				let left = left.into_int_value();
				let right = right.into_int_value();
				bc.build_left_shift(left, right, &name)
					.as_basic_value_enum()
			}
			CombineKind::SHR => {
				let left = left.into_int_value();
				let right = right.into_int_value();
				bc.build_right_shift(left, right, true, &name)
					.as_basic_value_enum()
			}
			CombineKind::USHR => {
				let left = left.into_int_value();
				let right = right.into_int_value();
				bc.build_right_shift(left, right, false, &name)
					.as_basic_value_enum()
			}
			CombineKind::XOR => {
				let left = left.into_int_value();
				let right = right.into_int_value();
				bc.build_xor(left, right, &name).as_basic_value_enum()
			}
			CombineKind::FCMPL | CombineKind::FCMPG => {
				let left = left.into_float_value();
				let right = right.into_float_value();

				// fucking cursed
				let greater = bc.build_float_compare(FloatPredicate::OGT, left, right, &name);
				let less = bc.build_float_compare(FloatPredicate::OLT, left, right, &name);
				let less = bc.build_int_neg(less, "fcmp");
				let unordered = bc.build_float_compare(FloatPredicate::UNO, left, right, &name);

				let y = if matches!(self.kind, CombineKind::FCMPG) {
					bc.build_int_neg(unordered, &name)
				} else {
					unordered
				};

				let value = bc.build_int_add(greater, less, &name);
				bc.build_int_add(value, y, &name).as_basic_value_enum()
			}
			CombineKind::ICMP => {
				let left = left.into_int_value();
				let right = right.into_int_value();

				// fucking cursed

				let v1 = bc.gen.next();
				let v2 = bc.gen.next();
				let v3 = bc.gen.next();
				let greater = bc.build_int_compare(IntPredicate::SGT, left, right, &name);
				let less = bc.build_int_compare(IntPredicate::SLT, left, right, &v1);
				let less = bc.build_int_neg(less, &v2);
				bc.build_int_add(greater, less, &v3)
					.as_basic_value_enum()
			}
		};

		bc.push(output);
	}
}

impl Display for CombineTask {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let op = match self.kind {
			CombineKind::ADD(_) => "+",
			CombineKind::DIV(_) => "/",
			CombineKind::MUL(_) => "*",
			CombineKind::REM(_) => "%",
			CombineKind::SUB(_) => "-",
			CombineKind::AND => "&",
			CombineKind::OR => "|",
			CombineKind::SHL => "<<",
			CombineKind::SHR => ">>",
			CombineKind::USHR => ">>>",
			CombineKind::XOR => "^",
			CombineKind::FCMPG => "cmpg",
			CombineKind::FCMPL => "cmpl",
			CombineKind::ICMP => "icmp",
		};

		write!(f, "combine {}",op)
	}
}


#[derive(Debug, Clone)]
pub enum CombineKind {
	// Math
	ADD(bool),
	DIV(bool),
	MUL(bool),
	REM(bool),
	SUB(bool),
	AND,
	OR,
	SHL,
	SHR,
	USHR,
	XOR,
	// Compare
	FCMPG,
	FCMPL,
	ICMP,
}
