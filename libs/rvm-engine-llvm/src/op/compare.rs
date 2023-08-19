use std::fmt::{Display, Formatter};

use inkwell::IntPredicate;

use rvm_reader::JumpKind;

use crate::compiler::BlockCompiler;
use crate::resolver::BlockResolver;

/// Compares two values against eachother
#[derive(Clone, Debug)]
pub struct CompareTask {
	pub target: usize,
	pub kind: CompareKind,
}

impl CompareTask {
	pub fn resolve(target: usize, kind: &JumpKind, resolver: &mut BlockResolver) -> CompareTask {
		let kind = match kind {
			JumpKind::IF_ACMPEQ => CompareKind::Equals,
			JumpKind::IF_ACMPNE => CompareKind::NotEquals,
			JumpKind::IF_ICMPEQ => CompareKind::Equals,
			JumpKind::IF_ICMPNE => CompareKind::NotEquals,
			JumpKind::IF_ICMPLT => CompareKind::LessThan,
			JumpKind::IF_ICMPLE => CompareKind::LessOrEquals,
			JumpKind::IF_ICMPGT => CompareKind::GreaterThan,
			JumpKind::IF_ICMPGE => CompareKind::GreaterOrEquals,
			_ => {
				panic!("invalid input, inputs needs to be matched")
			}
		};

		CompareTask {
			target,
			kind,
		}
	}

	pub fn compile(&self, bc: &mut BlockCompiler) {
		let lhs = bc.pop().into_int_value();
		let rhs = bc.pop().into_int_value();
		let then_block = bc.get_block(self.target);
		let else_block = bc.next_block();

		let op = match self.kind {
			CompareKind::Equals => IntPredicate::EQ,
			CompareKind::NotEquals => IntPredicate::NE,
			CompareKind::LessThan => IntPredicate::SLT,
			CompareKind::LessOrEquals => IntPredicate::SLE,
			CompareKind::GreaterThan => IntPredicate::SGT,
			CompareKind::GreaterOrEquals => IntPredicate::SGE,
		};

		let name = bc.gen.next();
		let comparison = bc.build_int_compare(op, lhs, rhs, &name);
		bc.build_conditional_branch(comparison, then_block, else_block);
	}
}

impl Display for CompareTask {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let op = match self.kind {
			CompareKind::Equals => "==",
			CompareKind::NotEquals => "!=",
			CompareKind::LessThan => "<",
			CompareKind::LessOrEquals => "<=",
			CompareKind::GreaterThan => ">",
			CompareKind::GreaterOrEquals => ">=",
		};
		write!(f, "if {op} then block{}", self.target)
	}
}

#[derive(Clone, Debug)]
pub enum CompareKind {
	Equals,
	NotEquals,
	LessThan,
	LessOrEquals,
	GreaterThan,
	GreaterOrEquals,
}
