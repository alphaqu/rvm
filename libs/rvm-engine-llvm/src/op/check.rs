use crate::compiler::BlockCompiler;
use inkwell::IntPredicate;
use std::fmt::{Display, Formatter};
use rvm_reader::JumpKind;

use crate::resolver::BlockResolver;

/// Checks a single value against a constant value
#[derive(Clone, Debug)]
pub struct CheckTask {
	pub target: usize,
	pub kind: CheckKind,
}

impl CheckTask {
	pub fn resolve(target: usize, kind: &JumpKind, resolver: &mut BlockResolver) -> CheckTask {
		let kind = match kind {
			JumpKind::IFEQ => CheckKind::EqualZero,
			JumpKind::IFNE => CheckKind::NotEqualZero,
			JumpKind::IFLT => CheckKind::LessThanZero,
			JumpKind::IFLE => CheckKind::LessOrEqualZero,
			JumpKind::IFGT => CheckKind::GreaterThanZero,
			JumpKind::IFGE => CheckKind::GreaterOrEqualZero,
			JumpKind::IFNONNULL => CheckKind::NotNull,
			JumpKind::IFNULL => CheckKind::Null,
			_ => {
				panic!("invalid input, inputs needs to be matched")
			}
		};

		CheckTask {
			target,
			kind,
		}
	}

	pub fn compile(&self, bc: &mut BlockCompiler) {
		let lhs = bc.pop().into_int_value();
		let zero = lhs.get_type().const_int(0, false);
		let then_block = bc.get_block(self.target);
		let else_block = bc.next_block();

		let op = match self.kind {
			CheckKind::EqualZero => IntPredicate::EQ,
			CheckKind::NotEqualZero => IntPredicate::NE,
			CheckKind::LessThanZero => IntPredicate::SLT,
			CheckKind::LessOrEqualZero => IntPredicate::SLE,
			CheckKind::GreaterThanZero => IntPredicate::SGT,
			CheckKind::GreaterOrEqualZero => IntPredicate::SGE,
			CheckKind::NotNull => IntPredicate::NE,
			CheckKind::Null => IntPredicate::EQ,
		};

		let name = bc.gen.next();
		let comparison = bc.build_int_compare(op, lhs, zero, &name);
		bc.build_conditional_branch(comparison, then_block, else_block);
	}
}

impl Display for CheckTask {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let op = match self.kind {
			CheckKind::EqualZero => "== 0",
			CheckKind::NotEqualZero => "!= 0",
			CheckKind::LessThanZero => "< 0",
			CheckKind::LessOrEqualZero => "<= 0",
			CheckKind::GreaterThanZero => "> 0",
			CheckKind::GreaterOrEqualZero => ">= 0",
			CheckKind::NotNull => "!= null",
			CheckKind::Null => "== null",
		};
		write!(f, "if (v0 {op}) then block{}", self.target)
	}
}

#[derive(Clone, Debug)]
pub enum CheckKind {
	EqualZero,
	NotEqualZero,
	LessThanZero,
	LessOrEqualZero,
	GreaterThanZero,
	GreaterOrEqualZero,
	NotNull,
	Null,
}
