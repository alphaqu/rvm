use std::fmt::{Display, Formatter};
use crate::compiler::BlockCompiler;

use crate::executor::Inst;
use crate::resolver::BlockResolver;

#[derive(Clone, Debug)]
pub struct JumpTask {
	pub target: usize,
}

impl JumpTask {
	pub fn resolve(i: usize, inst: &Inst, resolver: &mut BlockResolver) -> JumpTask {
		let offset = match inst {
			Inst::GOTO(offset) => offset.0 as i32,
			Inst::GOTO_W(offset) => offset.0,
			_ => {
				panic!("what")
			}
		};

		JumpTask {
			target: resolver.inst_to_block(i.saturating_add_signed(offset as isize)),
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
