use std::fmt::{Display, Formatter};

use combine::CombineTask;
use rvm_reader::{Inst, JumpInst, LocalInst, MathInst};

use crate::compiler::BlockCompiler;
use crate::op::apply::ApplyTask;
use crate::op::check::CheckTask;
use crate::op::compare::CompareTask;
use crate::op::constant::ConstTask;
use crate::op::conversion::ConversionTask;
use crate::op::invoke::InvokeTask;
use crate::op::jump::JumpTask;
use crate::op::ret::ReturnTask;
use crate::op::stack::StackTask;
use crate::op::variable::{IncrementTask, LoadVariableTask, StoreVariableTask};
use crate::resolver::BlockResolver;

pub mod apply;
pub mod check;
pub mod combine;
pub mod compare;
pub mod constant;
pub mod conversion;
pub mod invoke;
pub mod jump;
pub mod ret;
pub mod stack;
pub mod variable;

#[derive(Clone, Debug)]
pub enum Task {
	Nop,
	Apply(ApplyTask),
	Combine(CombineTask),
	Const(ConstTask),
	Stack(StackTask),
	Conversion(ConversionTask),
	Compare(CompareTask),
	Check(CheckTask),
	Jump(JumpTask),
	LoadVariable(LoadVariableTask),
	StoreVariable(StoreVariableTask),
	Increase(IncrementTask),
	Return(ReturnTask),
	Invoke(InvokeTask),
}

impl Task {
	pub fn resolve(i: usize, inst: &Inst, resolver: &mut BlockResolver) -> Task {
		match inst {
			Inst::Nop => Task::Nop,
			Inst::Math(inst @ MathInst::Neg(_)) => {
				Task::Apply(ApplyTask::resolve(inst, resolver))
			}
			Inst::Math(inst) => {
				Task::Combine(CombineTask::resolve(inst, resolver))
			}
			Inst::Const(inst) => {
				Task::Const(ConstTask::resolve(inst, resolver))
			}
			Inst::Stack(inst) => {
				Task::Stack(StackTask::resolve(inst, resolver))
			}
			Inst::Array(_) => {
				todo!("array")
			}
			Inst::Conversion(inst) => {
				Task::Conversion(ConversionTask::resolve(inst, resolver))
			}
			Inst::Jump(JumpInst {
						   offset,
						   kind
					   }) => {
				let target = resolver.inst_to_block(i.saturating_add_signed(*offset as isize));
				match kind.args() {
					2 => {
						Task::Compare(CompareTask::resolve(target, kind, resolver))
					}
					1 => {
						Task::Check(CheckTask::resolve(target, kind, resolver))
					}
					_ => {
						Task::Jump(JumpTask::resolve(target, resolver))
					}
				}
			}
			Inst::Local(inst) => {
				match inst {
					LocalInst::Load(kind, var) => {
						Task::LoadVariable(LoadVariableTask::resolve(kind.kind(), *var, resolver))
					}
					LocalInst::Store(kind, var) => {
						Task::StoreVariable(StoreVariableTask::resolve(kind.kind(), *var, resolver))
					}
					LocalInst::Increment(amount, var) => {
						Task::Increase(IncrementTask::resolve(*var, *amount, resolver))
					}
				}
			}
			Inst::Return(inst) => {
				Task::Return(ReturnTask::resolve(inst, resolver))
			}
			Inst::Invoke(inst) => {
				Task::Invoke(InvokeTask::resolve(inst, resolver))
			}
			// grandpa shit
			Inst::JSR(_) => todo!("grandpa shit"),
			Inst::JSR_W(_) => todo!("grandpa shit"),
			Inst::RET(_) => todo!("grandpa shit"),
			Inst::Throw(_) => {
				todo!("throw")
			}
			Inst::Comparison(_) => {
				todo!("comparison")
			}
			Inst::CheckCast(_) => {
				todo!("checkcast")
			}
			Inst::InstanceOf(_) => {
				todo!("instanceof")
			}
			Inst::New(_) => {
				todo!("new")
			}
			Inst::Field(_) => {
				todo!("field")
			}
			// alpha reading challange any%
			Inst::LOOKUPSWITCH => todo!("read"),
			Inst::TABLESWITCH => todo!("read"),
			Inst::MONITORENTER => todo!("read"),
			Inst::MONITOREXIT => todo!("read"),
		}
	}

	pub fn compile<'b, 'a>(&self, bc: &mut BlockCompiler<'b, 'a>) {
		match self {
			Task::Nop => {
				// pray
				panic!("nop instruction is intended to be temporary. shit blew up")
			}
			Task::Apply(v) => v.compile(bc),
			Task::Combine(v) => v.compile(bc),
			Task::Const(v) => v.compile(bc),
			Task::Conversion(v) => v.compile(bc),
			Task::LoadVariable(v) => v.compile(bc),
			Task::Invoke(v) => v.compile(bc),
			Task::Stack(v) => v.compile(bc),
			Task::Compare(v) => v.compile(bc),
			Task::Check(v) => v.compile(bc),
			Task::Jump(v) => v.compile(bc),
			Task::StoreVariable(v) => v.compile(bc),
			Task::Increase(v) => v.compile(bc),
			Task::Return(v) => v.compile(bc),
		}
	}
}

impl Display for Task {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Task::Nop => {
				write!(f, "nop")
			}
			Task::Apply(v) => v.fmt(f),
			Task::Combine(v) => v.fmt(f),
			Task::Const(v) => v.fmt(f),
			Task::Stack(v) => v.fmt(f),
			Task::Conversion(v) => v.fmt(f),
			Task::Compare(v) => v.fmt(f),
			Task::Check(v) => v.fmt(f),
			Task::Jump(v) => v.fmt(f),
			Task::LoadVariable(v) => v.fmt(f),
			Task::StoreVariable(v) => v.fmt(f),
			Task::Increase(v) => v.fmt(f),
			Task::Return(v) => v.fmt(f),
			Task::Invoke(v) => v.fmt(f),
		}
	}
}
