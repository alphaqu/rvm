use ahash::AHashMap;
use inkwell::basic_block::BasicBlock;
use inkwell::values::{BasicValueEnum, PointerValue};
use crate::compiler::compiler::LocalId;
use crate::compiler::op::{Task};
use crate::executor::Inst;
use crate::object::ValueType;
use crate::reader::MethodDescriptor;

pub struct Block<'a, 'ctx> {
    pub inst_start: usize,
    pub instructions: &'a [Inst],
    pub sources: Vec<usize>,
    pub targets: Vec<usize>,
    pub resolved: Option<ResolvedBlock>,
    pub compiling: Option<CompilingBlock<'ctx>>,
    pub compiled: Option<CompiledBlock<'ctx>>,
}

impl<'a, 'ctx>Block<'a, 'ctx> {
    pub fn get_start_idx(&self) -> usize{
        self.inst_start
    }
    pub fn get_end_idx(&self) -> usize {
        self.inst_start + (self.instructions.len() - 1)
    }

    pub fn compile(&mut self) {

    }
}

pub struct ResolvedBlock {
    pub tasks: Vec<Task>,
}

pub struct CompilingBlock<'ctx> {
    pub variables: AHashMap<LocalId, BlockVariable<'ctx>>,
    pub basic_block: BasicBlock<'ctx>,
}
pub struct CompiledBlock<'ctx> {
    pub outputs: Vec<PointerValue<'ctx>>
}

#[derive(Clone, Copy)]
pub struct BlockVariable<'ctx> {
    pub value: PointerValue<'ctx>,
    pub ty: ValueType
}
