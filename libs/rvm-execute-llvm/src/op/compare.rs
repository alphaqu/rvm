use crate::compiler::compiler::BlockCompiler;
use inkwell::IntPredicate;
use std::fmt::{Display, Formatter};

use crate::compiler::resolver::BlockResolver;
use crate::executor::Inst;

/// Compares two values against eachother
#[derive(Clone, Debug)]
pub struct CompareTask {
	pub target: usize,
	pub kind: CompareKind,
}

impl CompareTask {
	pub fn resolve(i: usize, inst: &Inst, resolver: &mut BlockResolver) -> CompareTask {
		let (kind, target) = match inst {
			Inst::IF_ACMPEQ(target) => (CompareKind::Equals, target),
			Inst::IF_ACMPNE(target) => (CompareKind::NotEquals, target),
			Inst::IF_ICMPEQ(target) => (CompareKind::Equals, target),
			Inst::IF_ICMPNE(target) => (CompareKind::NotEquals, target),
			Inst::IF_ICMPLT(target) => (CompareKind::LessThan, target),
			Inst::IF_ICMPLE(target) => (CompareKind::LessOrEquals, target),
			Inst::IF_ICMPGT(target) => (CompareKind::GreaterThan, target),
			Inst::IF_ICMPGE(target) => (CompareKind::GreaterOrEquals, target),
			_ => {
				panic!("invalid input, inputs needs to be matched")
			}
		};

		CompareTask {
			target: resolver.inst_to_block(i.saturating_add_signed(target.0 as isize)),
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
