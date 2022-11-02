use crate::{compile_method, Runtime};
use crate::compiler::block::{Block, ResolvedBlock};
use crate::compiler::compiler::{BlockCompiler, FunctionCompiler};
use crate::compiler::ir_gen::IrNameGen;
use crate::compiler::op::compare::CompareTask;
use crate::compiler::op::constant::ConstTask;
use crate::compiler::op::variable::LoadVariableTask;
use crate::compiler::op::Task;
use crate::compiler::resolver::BlockResolver;
use crate::executor::Inst;
use crate::object::{Method, MethodCode, ValueType};
use crate::reader::{
	Code, ConstantInfo, ConstantPool, MethodDescriptor, MethodInfo, ReturnDescriptor, StrParse,
	ValueDesc,
};
use ahash::{AHashMap, AHashSet, HashSet};
use either::Either;
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction, UnsafeFunctionPointer};
use inkwell::module::{Linkage, Module};
use inkwell::passes::{PassBuilderOptions, PassManager, PassManagerBuilder, PassManagerSubType, PassRegistry};
use inkwell::targets::CodeModel::Kernel;
use inkwell::types::{AnyType, BasicMetadataTypeEnum, BasicType, FunctionType, PointerType};
use inkwell::values::{
	BasicMetadataValueEnum, BasicValueEnum, CallableValue, FunctionValue, GlobalValue,
	InstructionValue, PointerValue,
};
use inkwell::{AddressSpace, OptimizationLevel};
use std::ffi::c_void;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::mem::{forget, transmute};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use inkwell::targets::{CodeModel, InitializationConfig, RelocMode, Target, TargetMachine, TargetTriple};
use tracing::{debug, error, info};
use crate::compiler::op::jump::JumpTask;

mod block;
mod compiler;
mod ir_gen;
mod op;
mod resolver;

pub struct Executor<'ctx> {
	ctx: &'ctx Context,
	module: Module<'ctx>,
	exec: ExecutionEngine<'ctx>,
	fpm: PassManager<FunctionValue<'ctx>>,
	mpm: PassManager<Module<'ctx>>,

	initialized: AtomicBool,
}

impl<'ctx> Executor<'ctx> {
	pub fn new(context: &'ctx Context) -> Executor<'ctx> {
		let module = context.create_module("module");
		Target::initialize_x86(&InitializationConfig::default());
		let triple = TargetTriple::create("x86_64-pc-linux-gnu");
		let target = Target::from_triple(&triple).unwrap();
		let machine = target.create_target_machine(
			&triple,
			TargetMachine::get_host_cpu_name().to_str().unwrap(),
			TargetMachine::get_host_cpu_features().to_str().unwrap(),
			OptimizationLevel::Aggressive,
			RelocMode::Default,
			CodeModel::JITDefault
		).unwrap();

		let target_triple = machine.get_triple();
		let target_data = machine.get_target_data();
		module.set_triple(&target_triple);
		module.set_data_layout(&target_data.get_data_layout());

		let builder = PassManagerBuilder::create();
		builder.set_optimization_level(OptimizationLevel::Aggressive);

		let registry = PassRegistry::get_global();
		registry.initialize_analysis();
		let fpm = PassManager::create(&module);
		fpm.add_instruction_combining_pass();
		fpm.add_reassociate_pass();
		fpm.add_gvn_pass();
		fpm.add_cfg_simplification_pass();
		fpm.add_basic_alias_analysis_pass();
		fpm.add_promote_memory_to_register_pass();
		fpm.add_instruction_combining_pass();
		fpm.add_reassociate_pass();
		fpm.initialize();

		// let target = Target::from_triple(&triple).unwrap();
		//
		// 		let machine = target.create_target_machine(
		// 			&triple,
		// 			TargetMachine::get_host_cpu_name().to_str().unwrap(),
		// 			TargetMachine::get_host_cpu_features().to_str().unwrap(),
		// 			OptimizationLevel::Aggressive,
		// 			RelocMode::Default,
		// 			CodeModel::JITDefault
		// 		).unwrap();
		//
		// 		let stromg = "tti,targetlibinfo,targetpassconfig,machinemoduleinfo,tbaa,scopednoaliasaa,assumptioncachetracker,profilesummaryinfo,collectormetadata,machinebranchprob,regallocevict,preiselintrinsiclowering,atomicexpand,loweramxintrinsics,loweramxtype,verify,domtree,basicaa,loops,loopsimplify,scalarevolution,canonfreeze,ivusers,loopreduce,basicaa,aa,mergeicmps,loops,lazybranchprob,lazyblockfreq,expandmemcmp,gclowering,shadowstackgclowering,lowerconstantintrinsics,unreachableblockelim,loops,postdomtree,branchprob,blockfreq,consthoist,replacewithveclib,partiallyinlinelibcalls,expandvp,scalarizemaskedmemintrin,expandreductions,loops,tlshoist,interleavedaccess,x86partialreduction,indirectbrexpand,loops,codegenprepare,domtree,dwarfehprepare,safestack,stackprotector,verify,basicaa,aa,loops,postdomtree,branchprob,lazybranchprob,lazyblockfreq,machinedomtree,finalizeisel,x86domainreassignment,lazymachineblockfreq,earlytailduplication,optphis,slotindexes,stackcoloring,localstackalloc,deadmielimination,machinedomtree,machineloops,machinetracemetrics,earlyifcvt,lazymachineblockfreq,machinecombiner,x86cmovconversion,machinedomtree,machineloops,machineblockfreq,earlymachinelicm,machinedomtree,machineblockfreq,machinecse,machinepostdomtree,machinecycles,machinesink,peepholeopt,deadmielimination,lrshrink,x86fixupsetcc,lazymachineblockfreq,x86optimizeLEAs,x86cfopt,x86avoidSFB,x86slh,machinedomtree,x86flagscopylowering,machinedomtree,machineloops,tilepreconfig,detectdeadlanes,processimpdefs,unreachablembbelimination,livevars,phinodeelimination,twoaddressinstruction,slotindexes,liveintervals,simpleregistercoalescing,renameindependentsubregs,machinescheduler,machineblockfreq,livedebugvars,livestacks,virtregmap,liveregmatrix,edgebundles,spillcodeplacement,lazymachineblockfreq,machineoptremarkemitter,greedy,tileconfig,greedy,virtregrewriter,regallocscoringpass,stackslotcoloring,machinecp,machinelicm,lowertilecopy,edgebundles,x86codegen,machinedomtree,machinedomfrontier,x86lviload,removeredundantdebugvalues,fixupstatepointcallersaved,postramachinesink,machineblockfreq,machinepostdomtree,lazymachineblockfreq,machineoptremarkemitter,shrinkwrap,prologepilog,branchfolder,lazymachineblockfreq,tailduplication,machinecp,postrapseudos,x86pseudo,machinedomtree,machineloops,postRAsched,gcanalysis,machineblockfreq,machinepostdomtree,blockplacement,fentryinsert,xrayinstrumentation,patchablefunction,reachingdepsanalysis,x86executiondomainfix,breakfalsedeps,machinedomtree,machineloops,lazymachineblockfreq,x86fixupbwinsts,lazymachineblockfreq,x86fixupLEAs,x86evextovexcompress,funcletlayout,stackmapliveness,livedebugvalues,x86seses,x86returnthunks,cfiinstrinserter,x86lviret,pseudoprobeinserter,lazymachineblockfreq,machineoptremarkemitter";
		//
		// 		for pass in stromg.split(",") {
		// 			if module.run_passes(
		// 				stromg,
		// 				&machine, PassBuilderOptions::create()).is_ok() {
		// 				println!("{pass}");
		// 			}
		// 		}
		// -tti -targetlibinfo -targetpassconfig -machinemoduleinfo -tbaa -scoped-noalias-aa -assumption-cache-tracker -profile-summary-info -collector-metadata -machine-branch-prob -regalloc-evict -pre-isel-intrinsic-lowering -atomic-expand -lower-amx-intrinsics -lower-amx-type -verify -domtree -basic-aa -loops -loop-simplify -scalar-evolution -canon-freeze -iv-users -loop-reduce -basic-aa -aa -mergeicmps -loops -lazy-branch-prob -lazy-block-freq -expandmemcmp -gc-lowering -shadow-stack-gc-lowering -lower-constant-intrinsics -unreachableblockelim -loops -postdomtree -branch-prob -block-freq -consthoist -replace-with-veclib -partially-inline-libcalls -expandvp -scalarize-masked-mem-intrin -expand-reductions -loops -tlshoist -interleaved-access -x86-partial-reduction -indirectbr-expand -loops -codegenprepare -domtree -dwarfehprepare -safe-stack -stack-protector -verify -basic-aa -aa -loops -postdomtree -branch-prob -lazy-branch-prob -lazy-block-freq -machinedomtree -finalize-isel -x86-domain-reassignment -lazy-machine-block-freq -early-tailduplication -opt-phis -slotindexes -stack-coloring -localstackalloc -dead-mi-elimination -machinedomtree -machine-loops -machine-trace-metrics -early-ifcvt -lazy-machine-block-freq -machine-combiner -x86-cmov-conversion -machinedomtree -machine-loops -machine-block-freq -early-machinelicm -machinedomtree -machine-block-freq -machine-cse -machinepostdomtree -machine-cycles -machine-sink -peephole-opt -dead-mi-elimination -lrshrink -x86-fixup-setcc -lazy-machine-block-freq -x86-optimize-LEAs -x86-cf-opt -x86-avoid-SFB -x86-slh -machinedomtree -x86-flags-copy-lowering -machinedomtree -machine-loops -tilepreconfig -detect-dead-lanes -processimpdefs -unreachable-mbb-elimination -livevars -phi-node-elimination -twoaddressinstruction -slotindexes -liveintervals -simple-register-coalescing -rename-independent-subregs -machine-scheduler -machine-block-freq -livedebugvars -livestacks -virtregmap -liveregmatrix -edge-bundles -spill-code-placement -lazy-machine-block-freq -machine-opt-remark-emitter -greedy -tileconfig -greedy -virtregrewriter -regallocscoringpass -stack-slot-coloring -machine-cp -machinelicm -lowertilecopy -edge-bundles -x86-codegen -machinedomtree -machine-domfrontier -x86-lvi-load -removeredundantdebugvalues -fixup-statepoint-caller-saved -postra-machine-sink -machine-block-freq -machinepostdomtree -lazy-machine-block-freq -machine-opt-remark-emitter -shrink-wrap -prologepilog -branch-folder -lazy-machine-block-freq -tailduplication -machine-cp -postrapseudos -x86-pseudo -machinedomtree -machine-loops -post-RA-sched -gc-analysis -machine-block-freq -machinepostdomtree -block-placement -fentry-insert -xray-instrumentation -patchable-function -reaching-deps-analysis -x86-execution-domain-fix -break-false-deps -machinedomtree -machine-loops -lazy-machine-block-freq -x86-fixup-bw-insts -lazy-machine-block-freq -x86-fixup-LEAs -x86-evex-to-vex-compress -funclet-layout -stackmap-liveness -livedebugvalues -x86-seses -x86-return-thunks -cfi-instr-inserter -x86-lvi-ret -pseudo-probe-inserter -lazy-machine-block-freq -machine-opt-remark-emitter
		let mpm = PassManager::create(());
		mpm.add_type_based_alias_analysis_pass();
		mpm.add_sccp_pass();
		mpm.add_prune_eh_pass();
		mpm.add_dead_arg_elimination_pass();
		mpm.add_lower_expect_intrinsic_pass();
		mpm.add_scalar_repl_aggregates_pass();
		mpm.add_instruction_combining_pass();
		mpm.add_jump_threading_pass();
		mpm.add_correlated_value_propagation_pass();
		mpm.add_cfg_simplification_pass();
		mpm.add_reassociate_pass();
		mpm.add_loop_rotate_pass();
		mpm.add_loop_unswitch_pass();
		mpm.add_ind_var_simplify_pass();
		mpm.add_licm_pass();
		mpm.add_loop_vectorize_pass();
		mpm.add_instruction_combining_pass();
		mpm.add_sccp_pass();
		mpm.add_reassociate_pass();
		mpm.add_cfg_simplification_pass();
		mpm.add_gvn_pass();
		mpm.add_memcpy_optimize_pass();
		mpm.add_dead_store_elimination_pass();
		mpm.add_bit_tracking_dce_pass();
		mpm.add_instruction_combining_pass();
		mpm.add_reassociate_pass();
		mpm.add_cfg_simplification_pass();
		mpm.add_slp_vectorize_pass();
		mpm.add_early_cse_pass();

		//	builder.populate_function_pass_manager(&fpm);
		//builder.populate_lto_pass_manager(&mpm, false, false);

		let exec = module
			.create_jit_execution_engine(OptimizationLevel::Aggressive)
			.unwrap();

		Executor {
			ctx: context,
			module,
			exec,
			fpm,
			mpm,
			initialized: AtomicBool::new(false),
		}
	}

	pub fn prepare(&self, runtime: *const Runtime, reference: &Reference) {
		debug!("Preparing {reference:?}");

		match reference {
			Reference::Method(method) => {
				let fn_name = method.call_name();
				debug!("Checking relay existance {fn_name}");
				if self.exec.get_function_address(&fn_name).is_err() {
					self.compile_relay(runtime, method);
				}
			}
		}
	}

	fn compile_relay(&self, runtime: *const Runtime, reference: &MethodReference) {
		let mut gen = IrNameGen::default();

		let descriptor = reference.desc();
		let fn_type = descriptor.func(self.ctx);

		let name = reference.call_name();
		debug!("Defining relay {name}");
		let function = self.module.add_function(&name, fn_type, Some(Linkage::External));
		let block = self.ctx.append_basic_block(function, &gen.next());

		// Write relay
		let builder = self.ctx.create_builder();
		builder.position_at_end(block);

		// Create string globals
		let class_name =
			builder.build_global_string_ptr(&reference.class_name, "class_name");
		let method_name =
			builder.build_global_string_ptr(&reference.method_name, "method_name");
		let desc = builder.build_global_string_ptr(&reference.desc, "desc");

		// Call the resolve_method function
		let resolve = self.module.get_function("resolve_method").unwrap();
		let resolved_ptr = builder
			.build_call(
				resolve,
				&[
					builder.build_int_to_ptr(self.ctx.i64_type().const_int(runtime as u64, false), self.ctx.i8_type().ptr_type(AddressSpace::Generic), "runtime").into(),
					class_name.as_pointer_value().into(),
					method_name.as_pointer_value().into(),
					desc.as_pointer_value().into(),
				],
				&gen.next(),
			)
			.try_as_basic_value()
			.unwrap_left()
			.into_pointer_value();

		// Invoke the resolved function. Next time this method will be this value
		let function_pointer = builder.build_pointer_cast(
			resolved_ptr,
			fn_type.ptr_type(AddressSpace::Generic),
			&gen.next(),
		);
		let args: Vec<BasicMetadataValueEnum> = function
			.get_params()
			.into_iter()
			.map(|v| BasicMetadataValueEnum::from(v))
			.collect();
		let function: CallableValue = function_pointer.try_into().unwrap();
		let ret = builder.build_call(function, &args, &gen.next());
		match ret.try_as_basic_value() {
			Either::Left(v) => {
				builder.build_return(Some(&v));
			}
			Either::Right(v) => {
				builder.build_return(None);
			}
		}

		//self.exec
		//	.get_function_address(&name)
		//	.expect("Could not find relay function we just defined")
	}

	pub fn compile_method(
		&self,
		runtime: *const Runtime,
		name: &MethodReference,
		is_static: bool,
		code: &Code,
		cp: &ConstantPool,
	) -> usize {
		if !self.initialized.load(Ordering::Relaxed) {
			self.initialized.store(true, Ordering::Relaxed);
			let runtime = self.ctx.i8_type().ptr_type(AddressSpace::Generic);
			let string = self.ctx.i8_type().ptr_type(AddressSpace::Generic);
			let function_type = self.ctx.i8_type().ptr_type(AddressSpace::Generic).fn_type(
				&[
					BasicMetadataTypeEnum::PointerType(runtime),
					BasicMetadataTypeEnum::PointerType(string),
					BasicMetadataTypeEnum::PointerType(string),
					BasicMetadataTypeEnum::PointerType(string),
				],
				false,
			);
			let value = self
				.module
				.add_function("resolve_method", function_type, Some(Linkage::External));
			self.exec
				.add_global_mapping(&value, compile_method as usize);
		}

		debug!("Computing {name}");
		// STAGE 1: Computation of blocks
		// This stage decompiles the bytecode to a block form where jump instructions go to a certain block.
		// Blocks are way more useful for compilation purposes.
		let mut data = self.compute_blocks(&code.instructions);

		debug!("Resolving {name}");
		// STAGE 2: Resolution of blocks.
		// This resolves things like stack values and makes the code more IR convertible
		// by partly decompiling the code and creating a concept with variables and temporaries
		self.resolve_blocks(runtime, &mut data, cp);

		debug!("Compiling {name}");
		//Self::print_blocks(&data.blocks);
		// STAGE 3: Compilation
		// This takes the resolution result and makes it into IR where LLVM can optimize away!
		self.compile_blocks(name, is_static, data);

		let fn_name = name.def_name();

		self.module.write_bitcode_to_path(&Path::new(&format!("./{name}.bc")));
		let function = self
			.exec
			.get_function_address(&fn_name)
			.expect("Could not find function");

		function
	}

	fn compile_blocks(&self, name: &MethodReference, is_static: bool, data: BlocksData<'_, 'ctx>) {
		let mut block_compiler = FunctionCompiler::new(
			self.ctx,
			&self.module,
			&self.fpm,
			name,
			is_static,
			data.blocks,
		);
		block_compiler.compile(&data.compile_order);

		info!("Redefining module");

		//self.module.print_to_stderr();
		if let Some(caller) = self.module.get_function(&name.call_name()) {
			caller.replace_all_uses_with(block_compiler.func);
		}

	//	self.module.print_to_stderr();
		if self.mpm.run_on(&self.module) {
			error!("optimized module");
			self.module.print_to_stderr();
		}

		self.exec.remove_module(&self.module).unwrap();
		self.exec.add_module(&self.module).unwrap();
	}

	fn resolve_blocks(&self, runtime: *const Runtime, data: &mut BlocksData, cp: &ConstantPool) {
		let mut references = AHashSet::new();
		for i in &data.compile_order {
			info!(target: "resolve", "Resolving block {i}");

			// resolve instructions
			let mut resolver = BlockResolver::new(data, *i, cp);

			let block = resolver.block();
			let start = block.inst_start;
			for (i, inst) in block.instructions.iter().enumerate() {
				resolver.resolve_task(start + i, inst);
			}

			let (compiled_block, refs) = resolver.build();
			for refs in refs {
				references.insert(refs);
			}
			data.blocks[*i].resolved = Some(compiled_block);
		}

		// define the references
		for reference in references {
			self.prepare(runtime, &reference);
		}
	}

	fn compute_blocks<'a>(&'a self, code: &'a [Inst]) -> BlocksData<'a, 'ctx> {
		// Compute splits
		let mut splits = Vec::new();
		for (i, inst) in code.iter().enumerate() {
			match inst {
				Inst::IF_ACMPEQ(value)
				| Inst::IF_ACMPNE(value)
				| Inst::IF_ICMPEQ(value)
				| Inst::IF_ICMPNE(value)
				| Inst::IF_ICMPLT(value)
				| Inst::IF_ICMPGE(value)
				| Inst::IF_ICMPGT(value)
				| Inst::IF_ICMPLE(value)
				| Inst::IFEQ(value)
				| Inst::IFNE(value)
				| Inst::IFLT(value)
				| Inst::IFGE(value)
				| Inst::IFGT(value)
				| Inst::IFLE(value)
				| Inst::IFNONNULL(value)
				| Inst::IFNULL(value) => {
					splits.push(i + 1);
					splits.push((value.0 as isize + i as isize) as usize);
				}
				Inst::GOTO(value) => {
					splits.push(i + 1);
					splits.push((value.0 as isize + i as isize) as usize);
				}
				Inst::GOTO_W(value) => {
					splits.push(i + 1);
					splits.push((value.0 as isize + i as isize) as usize);
				}
				Inst::RETURN
				| Inst::ARETURN
				// TODO implement throw correctly
				| Inst::ATHROW
				| Inst::DRETURN
				| Inst::FRETURN
				| Inst::IRETURN
				| Inst::LRETURN => {
					splits.push(i + 1);
				}
				_ => {}
			}
		}

		// remove duplicates
		splits.sort_unstable();
		splits.dedup();

		// create inst_to_block lookup
		let mut blocks = Vec::new();
		let mut inst_to_block = AHashMap::new();
		let mut old_pos = 0;
		for (i, pos) in splits.into_iter().enumerate() {
			let instructions = &code[old_pos..pos];
			//println!("{i}: {instructions:?}");
			if !instructions.is_empty() {
				inst_to_block.insert(old_pos, i);
				blocks.push(Block {
					inst_start: old_pos,
					instructions,
					sources: vec![],
					targets: vec![],
					resolved: None,
					compiling: None,
					compiled: None,
				});
				old_pos = pos;
			}
		}

		// Compute targets and sources.
		let values: Vec<(usize, Option<Inst>)> = blocks
			.iter()
			.enumerate()
			.map(|(i, v)| (i, v.instructions.last().cloned()))
			.collect();
		for (i, last) in values {
			if let Some(last) = last {
				match last {
					Inst::IF_ACMPEQ(value)
					| Inst::IF_ACMPNE(value)
					| Inst::IF_ICMPEQ(value)
					| Inst::IF_ICMPNE(value)
					| Inst::IF_ICMPLT(value)
					| Inst::IF_ICMPGE(value)
					| Inst::IF_ICMPGT(value)
					| Inst::IF_ICMPLE(value)
					| Inst::IFEQ(value)
					| Inst::IFNE(value)
					| Inst::IFLT(value)
					| Inst::IFGE(value)
					| Inst::IFGT(value)
					| Inst::IFLE(value)
					| Inst::IFNONNULL(value)
					| Inst::IFNULL(value) => {
						let this = &mut blocks[i];
						let position = (value.0 as isize + this.get_end_idx() as isize) as usize;
						let target = *inst_to_block.get(&position).expect("Could not find target");
						this.targets = vec![i + 1, target];
						blocks[target].sources.push(i);
						blocks[i + 1].sources.push(i);
					}
					Inst::GOTO(value) => {
						let this = &mut blocks[i];
						let position = (value.0 as isize + this.get_end_idx() as isize) as usize;
						let target = *inst_to_block.get(&position).expect("Could not find target");
						this.targets = vec![target];
						blocks[target].sources.push(i);
					}
					Inst::GOTO_W(value) => {
						let this = &mut blocks[i];
						let position = (value.0 as isize + this.get_end_idx() as isize) as usize;
						let target = *inst_to_block.get(&position).expect("Could not find target");
						this.targets = vec![target];
						blocks[target].sources.push(i);
					}
					Inst::RETURN
					// TODO implement throw correctly
					| Inst::ATHROW
					| Inst::ARETURN
					| Inst::DRETURN
					| Inst::FRETURN
					| Inst::IRETURN
					| Inst::LRETURN => {
						let this = &mut blocks[i];
						this.targets = vec![];
					}
					_ => {
						let this = &mut blocks[i];
						this.targets = vec![i + 1];
						blocks[i + 1].sources.push(i);
					}
				}
			}
		}

		// Create compilation order
		let mut visit = vec![0usize];
		let mut visited = AHashSet::new();
		let mut compile_order = Vec::new();

		while let Some(value) = visit.pop() {
			if !visited.contains(&value) {
				visited.insert(value);
				compile_order.push(value);

				for target in &blocks[value].targets {
					visit.push(*target);
				}
			}
		}

		BlocksData {
			inst_to_block,
			compile_order,
			blocks,
		}
	}

	pub fn print_blocks(blocks: &[Block]) -> fmt::Result {
		use std::fmt::Write;

		for (b, block) in blocks.iter().enumerate() {
			println!("Block {b}");
			if let Some(value) = block.resolved.as_ref() {
				for (i, task) in value.tasks.iter().enumerate() {
					println!("\t{i}: {task}");
				}
			}
		}

		let mut output = String::new();
		writeln!(&mut output, "digraph G {{")?;

		// Create the nodes
		for (i, block) in blocks.iter().enumerate() {
			writeln!(&mut output, "subgraph cluster_{i} {{")?;
			writeln!(&mut output, "label = \"Block {i}\";")?;
			let compiled = block.resolved.as_ref().unwrap();
			for (j, task) in compiled.tasks.iter().enumerate() {
				writeln!(
					&mut output,
					"b{i}i{j} [label=\"{}\"]",
					format!("{task}").replace("\"", "\\\"")
				)?;
			}

			writeln!(&mut output, "}}")?;
		}

		// Link nodes
		for (i, block) in blocks.iter().enumerate() {
			let compiled = block.resolved.as_ref().unwrap();
			for (j, _) in compiled.tasks.iter().enumerate() {
				if j != 0 {
					// link previous instruction
					writeln!(&mut output, "b{i}i{} -> b{i}i{j}", j - 1)?;
				}
			}
			// link outputs
			if !compiled.tasks.is_empty() {
				let last = compiled.tasks.len() - 1;
				for (v, value) in block.targets.iter().enumerate() {
					writeln!(
						&mut output,
						"b{i}i{last} -> b{}i0 [color=\"{}\"]",
						*value,
						if v == 0 { "green" } else { "red" }
					)?;
				}
			}
		}
		writeln!(&mut output, "}}")?;

		println!("{output}");
		Ok(())
	}
}

pub struct BlocksData<'a, 'ctx> {
	inst_to_block: AHashMap<usize, usize>,
	// compilation order
	compile_order: Vec<usize>,
	blocks: Vec<Block<'a, 'ctx>>,
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum Reference {
	Method(MethodReference),
}

impl Display for Reference {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Reference::Method(method) => {
				write!(f, "METHOD:{method}")
			}
		}
	}
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct MethodReference {
	pub class_name: String,
	pub method_name: String,
	pub desc: String,
}

impl MethodReference {
	pub fn call_name(&self) -> String {
		format!("CALL{}", self)
	}

	pub fn def_name(&self) -> String {
		format!("DEF{}", self)
	}

	pub fn desc(&self) -> MethodDescriptor {
		// valid because checked on creation
		MethodDescriptor::parse(&self.desc).unwrap()
	}
}

impl Display for MethodReference {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}.{}{}", self.class_name, self.method_name, self.desc)
	}
}
