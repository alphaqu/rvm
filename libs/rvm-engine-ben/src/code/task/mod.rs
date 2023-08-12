use std::fmt::{Display, Formatter};

pub use combine::{CombineTask, CombineTaskOperation, CombineTaskType};
pub use local::{LocalTask, LocalTaskKind};
pub use r#const::ConstTask;
pub use r#return::ReturnTask;
use rvm_object::ObjectClass;
use rvm_reader::{Inst, JumpInst, LocalInst, MathInst};

use crate::code::task::call::CallTask;
use crate::code::task::field::FieldTask;
use crate::code::task::object::NewTask;
use crate::code::task::stack::StackTask;

mod call;
mod combine;
mod r#const;
mod field;
mod jump;
mod local;
mod object;
mod r#return;
mod stack;

#[derive(Debug)]
pub enum Task {
	Nop,
	Const(ConstTask),
	Combine(CombineTask),
	Local(LocalTask),
	Return(ReturnTask),
	Jump(JumpInst),
	Call(CallTask),
	Stack(StackTask),
	New(NewTask),
	Field(FieldTask),
}

impl Display for Task {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Task::Nop => write!(f, "NOP"),
			Task::Const(v) => v.fmt(f),
			Task::Combine(v) => v.fmt(f),
			Task::Local(v) => v.fmt(f),
			Task::Return(v) => v.fmt(f),
			Task::Jump(v) => v.fmt(f),
			Task::Call(v) => v.fmt(f),
			Task::Stack(v) => v.fmt(f),
			Task::New(v) => v.fmt(f),
			Task::Field(v) => v.fmt(f),
		}
	}
}
impl Task {
	pub fn new(inst: &Inst, class: &ObjectClass) -> Task {
		match inst {
			Inst::Nop => Task::Nop,
			Inst::Const(inst) => Task::Const(ConstTask::new(inst, class)),
			Inst::Math(
				math @ (MathInst::Add(_)
				| MathInst::Sub(_)
				| MathInst::Div(_)
				| MathInst::Mul(_)
				| MathInst::Rem(_)
				| MathInst::And(_)
				| MathInst::Or(_)
				| MathInst::Xor(_)
				| MathInst::Shl(_)
				| MathInst::Shr(_)
				| MathInst::Ushr(_)),
			) => Task::Combine(CombineTask::new(math)),
			Inst::Local(local @ (LocalInst::Load(_, _) | LocalInst::Store(_, _))) => {
				Task::Local(LocalTask::new(local))
			}
			Inst::Invoke(inst) => Task::Call(CallTask::new(inst, class)),
			Inst::Return(ret) => Task::Return(ReturnTask::new(ret)),
			Inst::Jump(inst) => Task::Jump(*inst),
			Inst::Stack(inst) => Task::Stack(StackTask::new(inst)),
			Inst::New(inst) => Task::New(NewTask::new(inst, class)),
			Inst::Field(inst) => Task::Field(FieldTask::new(inst, class)),
			_ => todo!("{inst:?}"),
		}
	}
}
