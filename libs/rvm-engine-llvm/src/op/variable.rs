use std::fmt::{Display, Formatter};
use std::mem::transmute;

use inkwell::values::BasicValue;
use rvm_core::Kind;
use crate::compiler::{BlockCompiler, LocalId};
use crate::op::Task;
use crate::resolver::BlockResolver;

#[derive(Clone, Debug)]
pub struct LoadVariableTask(pub Var);

impl From<LoadVariableTask> for Task {
	fn from(val: LoadVariableTask) -> Self {
		Task::LoadVariable(val)
	}
}

impl LoadVariableTask {
	pub fn resolve(kind: Kind, var: u16, resolver: &mut BlockResolver) -> LoadVariableTask {
		let variable = resolver.get_local(var, kind);
		LoadVariableTask(variable)
	}

	pub fn get_type(&self) -> Kind {
		self.0.ty
	}

	pub fn compile<'b, 'a>(&self, bc: &mut BlockCompiler<'b, 'a>) {
		let ptr = match self.0.data {
			VarData::Local(id) => bc.get_local(id).value,
		};

		let name = bc.gen.next();
		bc.push(bc.build_load(ptr, &name).as_basic_value_enum());
	}
}

impl Display for LoadVariableTask {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "load {}", self.0)
	}
}

#[derive(Clone, Debug)]
pub struct StoreVariableTask {
	pub var: Var,
}

impl StoreVariableTask {
	pub fn resolve(kind: Kind, var: u16, resolver: &mut BlockResolver) -> StoreVariableTask {
		let var = resolver.get_local(var, kind);
		StoreVariableTask { var }
	}

	pub fn compile(&self, bc: &mut BlockCompiler) {
		let ptr = match self.var.data {
			VarData::Local(id) => {
				bc.define_variable(id, self.var.ty);
				bc.get_local(id).value
			}
		};

		let value = bc.pop();
		bc.build_store(ptr, value);
	}
}

impl Display for StoreVariableTask {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "store {}", self.var)
	}
}

#[derive(Clone, Debug)]
pub struct IncrementTask {
	var: Var,
	amount: i16,
}

impl IncrementTask {
	pub fn resolve(var: u16, amount: i16, resolver: &mut BlockResolver) -> IncrementTask {
		let var = resolver.get_local(var, Kind::Int);
		IncrementTask { var, amount }
	}

	pub fn compile(&self, bc: &mut BlockCompiler) {
		let ptr = match self.var.data {
			VarData::Local(id) => {
				bc.define_variable(id, self.var.ty);
				bc.get_local(id).value
			}
		};

		let name = bc.gen.next();
		let value = bc.build_load(ptr, &name).into_int_value();
		let amount = bc.int().const_int(
			unsafe { transmute::<i32, u32>(self.amount as i32) } as u64,
			false,
		);

		let name = bc.gen.next();
		let output = bc.build_int_add(value, amount, &name);
		bc.build_store(ptr, output);
	}
}

impl Display for IncrementTask {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "inc {} {}", self.var, self.amount)
	}
}
#[derive(Copy, Clone, Debug)]
pub struct Var {
	pub ty: Kind,
	pub data: VarData,
}

impl Display for Var {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self.data {
			VarData::Local(id) => write!(f, "{id:?}_{}", self.ty),
		}
	}
}

#[derive(Debug, Copy, Clone)]
pub enum VarData {
	/// A local variable
	Local(LocalId),
}
