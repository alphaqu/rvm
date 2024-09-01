use std::fmt::{Display, Formatter};

pub use combine::{CombineTask, CombineTaskOperation, CombineTaskType};
pub use local::{LocalTask, LocalTaskKind};
pub use r#const::ConstTask;
pub use r#return::ReturnTask;
use rvm_reader::{ArrayInst, Inst, JumpInst, LocalInst, MathInst};
use rvm_runtime::InstanceClass;

use crate::code::task::array::{
	ArrayCreateRefTask, ArrayCreateTask, ArrayLengthTask, ArrayLoadTask, ArrayStoreTask,
};
pub use crate::code::task::call::*;
use crate::code::task::field::FieldTask;
use crate::code::task::increment::IncrementTask;
use crate::code::task::jump::JumpTask;
use crate::code::task::object::NewTask;
use crate::code::task::stack::StackTask;
use crate::code::task::switch::SwitchTableTask;

mod array;
mod call;
mod combine;
mod r#const;
mod field;
mod increment;
mod jump;
mod local;
mod object;
mod r#return;
mod stack;
mod switch;

#[derive(Debug)]
pub enum Task {
	Nop,
	Const(ConstTask),
	Combine(CombineTask),
	Local(LocalTask),
	Increment(IncrementTask),
	Return(ReturnTask),
	Jump(JumpTask),
	Call(CallTask),
	Stack(StackTask),
	New(NewTask),
	Field(FieldTask),

	ArrayLength(ArrayLengthTask),
	ArrayCreate(ArrayCreateTask),
	ArrayCreateRef(ArrayCreateRefTask),
	ArrayLoad(ArrayLoadTask),
	ArrayStore(ArrayStoreTask),
	SwitchTable(SwitchTableTask),
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
			Task::Increment(v) => v.fmt(f),
			Task::ArrayLength(v) => v.fmt(f),
			Task::ArrayLoad(v) => v.fmt(f),
			Task::ArrayStore(v) => v.fmt(f),
			Task::ArrayCreate(v) => v.fmt(f),
			Task::ArrayCreateRef(v) => v.fmt(f),
			Task::SwitchTable(v) => v.fmt(f),
		}
	}
}

impl Task {
	pub fn new(inst: &Inst, class: &InstanceClass) -> Task {
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
			Inst::Local(LocalInst::Increment(amount, local)) => Task::Increment(IncrementTask {
				local: *local,
				increment: *amount,
			}),
			Inst::Invoke(inst) => Task::Call(CallTask::new(inst, class)),
			Inst::Return(ret) => Task::Return(ReturnTask::new(ret)),
			Inst::Jump(inst) => Task::Jump(JumpTask::new(inst)),
			Inst::Stack(inst) => Task::Stack(StackTask::new(inst)),
			Inst::New(inst) => Task::New(NewTask::new(inst, class)),
			Inst::Field(inst) => Task::Field(FieldTask::new(inst, class)),
			Inst::Array(ArrayInst::Length) => Task::ArrayLength(ArrayLengthTask),
			Inst::Array(ArrayInst::Load(kind)) => Task::ArrayLoad(ArrayLoadTask(*kind)),
			Inst::Array(ArrayInst::Store(kind)) => Task::ArrayStore(ArrayStoreTask(*kind)),
			Inst::Array(ArrayInst::NewPrim(ty)) => Task::ArrayCreate(ArrayCreateTask(*ty)),
			Inst::Array(ArrayInst::NewRef(ptr)) => {
				Task::ArrayCreateRef(ArrayCreateRefTask::new(ptr, class))
			}
			Inst::TableSwitch(inst) => Task::SwitchTable(SwitchTableTask::new(inst)),
			_ => todo!("{inst:?}"),
		}
	}
}
