use crate::compiler::compiler::BlockCompiler;
use crate::compiler::resolver::BlockResolver;
use inkwell::values::BasicValue;
use std::fmt::{Display, Formatter};
use rvm_core::PrimitiveType;
use rvm_reader::MathInst;

/// Applies an operation on one value and spits out another
#[derive(Clone, Debug)]
pub struct ApplyTask {
	pub kind: ApplyKind,
}

impl ApplyTask {
	pub fn resolve(inst: &MathInst, resolver: &mut BlockResolver) -> ApplyTask {
		let kind = match inst {
			MathInst::Neg(PrimitiveType::Float) | MathInst::Neg(PrimitiveType::Double) => {
				ApplyKind::NEG(true)
			}
			MathInst::Neg(PrimitiveType::Int) | MathInst::Neg(PrimitiveType::Long) => {
				ApplyKind::NEG(false)
			}
			_ => {
				panic!("what")
			}
		};

		ApplyTask { kind }
	}

	pub fn compile<'b, 'a>(&self, bc: &mut BlockCompiler<'b, 'a>) {
		let output = match self.kind {
			ApplyKind::NEG(float) => {
				let x = bc.pop();
				if float {
					bc.build_float_neg(x.into_float_value(), "fneg")
						.as_basic_value_enum()
				} else {
					bc.build_int_neg(x.into_int_value(), "ineg")
						.as_basic_value_enum()
				}
			}
		};
		bc.push(output);
	}
}

impl Display for ApplyTask {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self.kind {
			ApplyKind::NEG(_) => {
				write!(f, "apply -")
			}
		}
	}
}

#[derive(Debug, Clone)]
pub enum ApplyKind {
	NEG(bool),
}
