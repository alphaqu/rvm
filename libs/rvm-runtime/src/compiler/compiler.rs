use crate::compiler::block::{Block, BlockVariable, CompiledBlock, CompilingBlock};
use crate::executor::{StackValue, StackValueType};
use crate::object::ValueType;
use ahash::{AHashMap, AHashSet};
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::types::{BasicMetadataTypeEnum, BasicType, FloatType, IntType};
use inkwell::values::{BasicValue, BasicValueEnum, FunctionValue};
use std::collections::hash_map::Entry;
use std::ops::{Deref, DerefMut};
use inkwell::module::{Linkage, Module};
use inkwell::passes::PassManager;
use tracing::{info, warn};
use crate::compiler::ir_gen::IrNameGen;
use crate::compiler::MethodReference;
use crate::compiler::op::Task;
use crate::reader::{MethodDescriptor, ParameterDescriptor, ReturnDescriptor};


pub struct FunctionCompiler<'a, 'ctx> {
	ctx: &'ctx Context,
	module: &'a Module<'ctx>,
	fpm: &'a PassManager<FunctionValue<'ctx>>,
	gen: IrNameGen,
	builder: Builder<'ctx>,
	blocks: Vec<Block<'a, 'ctx>>,

	returns: ReturnDescriptor,
	pub func: FunctionValue<'ctx>,
}

impl<'a, 'ctx> FunctionCompiler<'a, 'ctx> {
	pub fn new(
		ctx: &'ctx Context,
		module: &'a Module<'ctx>,
		fpm: &'a PassManager<FunctionValue<'ctx>>,
		name: &MethodReference,
		is_static: bool,
		mut blocks: Vec<Block<'a, 'ctx>>,

	) -> FunctionCompiler<'a, 'ctx> {
		let mut gen = IrNameGen::default();
		let desc = name.desc();

		// Create the signature
		let mut parameter_types = Vec::new();
		let param = desc.parameters.clone();

		if !is_static {
			panic!("not static method")
		}

		for parameter in &param {
			let ty = parameter.0.ty().ir(ctx);
			parameter_types.push(ty);
		}

		let param_types: Vec<BasicMetadataTypeEnum> = parameter_types.iter().map(|v| BasicMetadataTypeEnum::from(*v)).collect();
		let ty = match &desc.ret {
			ReturnDescriptor::Field(ty) => {
				ty.ty().ir(ctx).fn_type(&param_types, false)
			}
			ReturnDescriptor::Void => {
				ctx.void_type().fn_type(&param_types, false)
			}
		};

		let id = name.def_name();

		let func = module.add_function(&id, ty, Some(Linkage::External));

		let mut first_block = ctx.append_basic_block(func, "entry");
		let builder = ctx.create_builder();
		// for definition of inputs
		builder.position_at_end(first_block);

		// Define parameters
		let mut parameters = AHashMap::new();
		for (i, (ty, desc)) in parameter_types.iter().zip(param.iter()).enumerate() {
			let pointer_value = builder.build_alloca(*ty, &gen.next());
			builder.build_store(pointer_value, func.get_nth_param(i as u32).unwrap());
			parameters.insert(LocalId::Local(i as u16), BlockVariable {

				value: pointer_value,
				ty: desc.0.ty(),
			});
		}

		// Resolve blocks
		let mut to_resolve = Vec::new();
		to_resolve.push((0usize, first_block));
		let mut visited = AHashSet::new();
		while let Some((id, basic_block)) = to_resolve.pop() {
			let block = &blocks[id];
			visited.insert(id);

			// Add targets

			// Resolve targets
			for target in &block.targets {
				if !visited.contains(target) {
					to_resolve.push((*target, ctx.insert_basic_block_after(basic_block, &format!("block{target}"))));
				}
			}

			// Compile inputs values by allocating inputs
			// 			let res = block.resolved.as_ref().expect("unresolved");
			//
			// 			let mut outputs = Vec::new();
			// 			if !res.outputs.is_empty() {
			// 				// Check if outputs already exist
			// 				'check: for target in &block.targets {
			// 					let target_sources = &blocks[*target].sources;
			// 					for target_source in target_sources {
			// 						if let Some(input) = blocks[*target_source].compiling.as_ref() {
			// 							outputs = input.outputs.clone();
			// 							break 'check;
			// 						}
			// 					}
			// 				}
			//
			// 				// Create stack outputs and pray llvm optimizes things
			// 				if outputs.is_empty() {
			// 					for op in res.outputs.iter() {
			// 						let ty = op.get_type();
			// 						let value = ty.ir(ctx);
			// 						let value = builder.build_alloca(value, &gen.next());
			// 						outputs.push(BlockVariable { value, ty })
			// 					}
			// 				}
			// 			}
			//
			// 			let mut inputs = Vec::new();
			// 			for source in &block.sources {
			// 				if let Some(compiling) = &blocks[*source].compiling {
			// 					inputs = compiling.outputs.clone();
			// 					break;
			// 				}
			// 			}

			blocks[id].compiling = Some(CompilingBlock {
				variables: parameters.clone(),
				basic_block,
			});
		}

		FunctionCompiler {
			ctx,
			module,
			fpm,
			gen,
			builder,
			blocks,
			returns: desc.ret,
			func,
		}
	}

	pub fn compile(& mut self, order: &[usize]) {
		for block in order {
			self.compile_block(*block);
		}

		if self.fpm.run_on(&self.func) {
			info!("Optimized")
		};
	}

	fn compile_block(&mut self, id: usize) {
		let block = &self.blocks[id];
		let resolved = block.resolved.as_ref().expect("unresolved");
		let compiling = block.compiling.as_ref().expect("unresolved");
		let variables = compiling.variables.clone();

		let mut compiler = BlockCompiler {
			ctx: self.ctx,
			gen: &mut self.gen,
			builder: &self.builder,
			block: id,
			blocks: &self.blocks,
			variables,
			module: self.module,
			stack: Vec::new(),
			returns: self.returns.clone(),
		};

		let basic_block = compiler.current_block();
		compiler.position_at_end(basic_block);

		// Insert inputs
		for source in &block.sources {
			let source = &self.blocks[*source];
			if let Some(compiled) = &source.compiled {
				for value in &compiled.outputs {
					let value_enum = compiler.build_load(*value, "input");
					compiler.stack.push(value_enum);
				}
				break;
			}
		}

		// Compile values
		for task in &resolved.tasks {
			task.compile(&mut compiler);
		}

		let stack = compiler.stack.clone();
		let mut outputs = Vec::new();
		if !stack.is_empty() {
			'check: for target in &block.targets {
				let target_sources = &self.blocks[*target].sources;
				for target_source in target_sources {
					if let Some(input) = self.blocks[*target_source].compiled.as_ref() {

						outputs = input.outputs.clone();
						break 'check;
					}
				}
			}

			if outputs.is_empty() {
				// Position at beggining
				let first_block = compiler.get_block(0);
				if let Some(first) = first_block.get_first_instruction() {
					compiler.position_before(&first);
				} else {
					compiler.position_at_end(first_block);
				}

				for value in &stack {
					let output = self.builder.build_alloca(value.get_type(), "output");
					outputs.push(output);
				}
			}

			if outputs.is_empty() {
				panic!("outputs were never processed. very concern")
			}

			// Process outputs, set the location before the terminator for obvious reasons
			if let Some(value) = basic_block.get_terminator() {
				compiler.position_before(&value);
			}

			for (i, value) in stack.into_iter().enumerate() {
				self.builder.build_store(outputs[i], value);
			}
		}

		let variables = compiler.variables;
		let block = &mut self.blocks[id];
		block.compiled = Some(CompiledBlock {
			outputs
		});

		let compiling = block.compiling.as_mut().expect("unresolved");
		// this is technically not needed because it will never get compiled again so this info is useless
		for (id, var) in &variables {
			compiling.variables.insert(*id, *var);
		}

		// set target variables
		for targets in block.targets.clone() {
			let block = &mut self.blocks[targets];
			let compiling = block.compiling.as_mut().expect("unresolved");
			for (id, var) in &variables {
				compiling.variables.insert(*id, *var);
			}
		}
	}
}

pub struct BlockCompiler<'b, 'ctx> {
	pub gen: &'b mut IrNameGen,
	ctx: &'ctx Context,
	module: &'b Module<'ctx>,
	builder: &'b Builder<'ctx>,
	block: usize,
	blocks: &'b Vec<Block<'b, 'ctx>>,
	variables: AHashMap<LocalId, BlockVariable<'ctx>>,

	returns: ReturnDescriptor,
	stack: Vec<BasicValueEnum<'ctx>>,
}

impl<'b, 'a> BlockCompiler<'b, 'a> {
	pub fn define_variable(&mut self, id: LocalId, ty: ValueType) {
		match self.variables.entry(id) {
			Entry::Occupied(mut occupied) => {
				if occupied.get().ty != ty {
					warn!("Overwriting variable {id:?}");
					occupied.get_mut().ty = ty;
				}
			}
			Entry::Vacant(vacant) => {
				info!("Defining local {ty} {id:?}");
				let basic_ty = ty.ir(self.ctx);
				let value = self.builder.build_alloca(basic_ty, &self.gen.next());

				vacant.insert(BlockVariable { value, ty });
			}
		}
	}

	pub fn get_local(&self, id: LocalId) -> BlockVariable<'a> {
		*self.variables.get(&id).ok_or_else(|| {
			format!("Could not find local {id:?}")
		}).unwrap()
	}

	pub fn get_block(&self, id: usize) -> BasicBlock<'a> {
		self.blocks[id].compiling.as_ref().expect("dead").basic_block
	}

	pub fn next_block(&self) -> BasicBlock<'a> {
		self.get_block(self.block + 1)
	}

	pub fn current_block(&self) -> BasicBlock<'a> {
		self.get_block(self.block)
	}

	pub fn boolean(&self) -> IntType<'a> {
		self.ctx.bool_type()
	}
	pub fn i8(&self) -> IntType<'a> {
		self.ctx.i8_type()
	}
	pub fn short(&self) -> IntType<'a> {
		self.ctx.i16_type()
	}
	pub fn int(&self) -> IntType<'a> {
		self.ctx.i32_type()
	}
	pub fn long(&self) -> IntType<'a> {
		self.ctx.i64_type()
	}
	pub fn char(&self) -> IntType<'a> {
		self.ctx.i16_type()
	}
	pub fn float(&self) -> FloatType<'a> {
		self.ctx.f32_type()
	}
	pub fn double(&self) -> FloatType<'a> {
		self.ctx.f64_type()
	}


	pub fn module(&self) -> &'b Module<'a> {
		self.module
	}

	pub fn push(&mut self, value: BasicValueEnum<'a>) {
		self.stack.push(value);
	}

	pub fn pop(&mut self) -> BasicValueEnum<'a> {
		self.stack.pop().unwrap()
	}


	pub fn returns(&self) -> &ReturnDescriptor {
		&self.returns
	}
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub enum LocalId {
	Temporary(u32),
	Local(u16),
}

impl<'b, 'a> Deref for BlockCompiler<'b, 'a> {
	type Target = Builder<'a>;

	fn deref(&self) -> &Self::Target {
		&self.builder
	}
}


