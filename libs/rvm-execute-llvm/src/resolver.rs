use crate::block::{Block, ResolvedBlock};
use ahash::AHashSet;
use rvm_core::Kind;
use rvm_reader::ConstantPool;


use crate::compiler::LocalId::Local;
use crate::op::jump::JumpTask;
use crate::op::variable::{Var, VarData};
use crate::op::Task;
use crate::executor::{BlocksData, Inst, Reference};

/// Psudo executes the java instructions to parse
/// them into an instruction tree which later gets
/// converter to the IR the jit consumes.
pub struct BlockResolver<'a, 'ctx> {
	tasks: Vec<Task>,
	references: AHashSet<Reference>,

	cp: &'a ConstantPool,
	data: &'a BlocksData<'a, 'ctx>,
	block: usize,
}

impl<'b, 'ctx> BlockResolver<'b, 'ctx> {
	pub fn new(blocks: &'b BlocksData<'b, 'ctx>, block: usize, cp: &'b ConstantPool) -> Self {
		Self {
			tasks: Vec::new(),
			references: Default::default(),
			cp,
			block,
			data: blocks,
		}
	}

	pub fn resolve_task(&mut self, i: usize, inst: &Inst) {
		let task = Task::resolve(i, inst, self);
		self.tasks.push(task);
	}

	pub fn inst_to_block(&self, inst: usize) -> usize {
		*self.data.inst_to_block.get(&inst).expect("out of bounds")
	}

	pub fn block(&self) -> &Block<'b, 'ctx> {
		&self.data.blocks[self.block]
	}

	pub fn get_local(&mut self, value: u16, ty: Kind) -> Var {
		Var {
			ty,
			data: VarData::Local(Local(value)),
		}
	}

	pub fn add_ref(&mut self, reference: Reference) {
		self.references.insert(reference);
	}

	pub fn build(mut self) -> (ResolvedBlock, AHashSet<Reference>) {
		// If the blocks end on a non terminating task. goto the next block else llvm will complain
		if let Some(value) = self.tasks.last() {
			match value {
				Task::Compare(_) | Task::Check(_) | Task::Jump(_) | Task::Return(_) => {}
				_ => self.tasks.push(Task::Jump(JumpTask {
					target: self.block + 1,
				})),
			}
		}
		(ResolvedBlock { tasks: self.tasks }, self.references)
	}

	pub fn cp(&self) -> &'b ConstantPool {
		self.cp
	}
}
