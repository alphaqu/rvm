use std::fmt::{Display, Formatter};
use rvm_reader::JumpKind;
use crate::compiler::compiler::BlockCompiler;

use crate::compiler::resolver::BlockResolver;

#[derive(Clone, Debug)]
pub struct JumpTask {
	pub target: usize,
}

impl JumpTask {
	pub fn resolve(target: usize, resolver: &mut BlockResolver) -> JumpTask {
		JumpTask {
			target,
		}
	}

	pub fn compile(&self, bc: &mut BlockCompiler) {
		let mut target = bc.get_block(self.target);
		bc.build_unconditional_branch(target);
	}
}

impl Display for JumpTask {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "goto block{}", self.target)
	}
}
