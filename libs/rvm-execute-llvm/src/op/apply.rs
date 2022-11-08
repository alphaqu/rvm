use crate::compiler::BlockCompiler;
use crate::resolver::BlockResolver;
use crate::executor::Inst;
use inkwell::values::BasicValue;
use std::fmt::{Display, Formatter};

/// Applies an operation on one value and spits out another
#[derive(Clone, Debug)]
pub struct ApplyTask {
	pub kind: ApplyKind,
}

impl ApplyTask {
	pub fn resolve(inst: &Inst, resolver: &mut BlockResolver) -> ApplyTask {
		let kind = match inst {
			Inst::FNEG | Inst::DNEG => ApplyKind::NEG(true),
			Inst::INEG | Inst::LNEG => ApplyKind::NEG(false),
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
