use crate::executor::Executor;
use inkwell::context::Context;
use once_cell::sync::Lazy;
use rvm_execute::{Bindings, ExecutionEngine, Method};
use rvm_reader::ConstantPool;
use self_cell::self_cell;
use std::ffi::c_void;
use std::sync::Mutex;

mod block;
mod compiler;
mod executor;
mod ir_gen;
mod op;
mod resolver;

self_cell!(
	struct LLVMEngine {
		owner: Context,

		#[covariant]
		dependent: Executor,
	}
);

pub struct LLVMExecutionEngine {
	engine: LLVMEngine,
}

impl LLVMExecutionEngine {
	pub fn new() -> LLVMExecutionEngine {
		LLVMExecutionEngine {
			engine: LLVMEngine::new(Context::create(), |ctx| Executor::new(ctx)),
		}
	}
}

impl ExecutionEngine for LLVMExecutionEngine {
	fn compile_method(
		&self,
		bindings: &Bindings,
		method: &Method,
		cp: &ConstantPool,
	) -> *const c_void {
		self.engine
			.borrow_dependent()
			.compile_method(bindings, method, cp) as *const c_void
	}
}
