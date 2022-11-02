use std::fmt::{Display, Formatter};
use std::mem::transmute;
use inkwell::values::{BasicValue, BasicValueEnum};
use crate::compiler::compiler::{BlockCompiler, LocalId};
use crate::compiler::op::{Task};
use crate::compiler::op::combine::{CombineTask, CombineKind};
use crate::compiler::op::constant::ConstTask;
use crate::compiler::resolver::BlockResolver;
use crate::executor::Inst;
use crate::object::ValueType;

#[derive(Clone, Debug)]
pub struct LoadVariableTask(pub Var);

impl Into<Task> for LoadVariableTask {
	fn into(self) -> Task {
		Task::LoadVariable(self)
	}
}

impl LoadVariableTask {
	pub fn resolve(inst: &Inst, resolver: &mut BlockResolver) -> LoadVariableTask {
		let mut load = |pos: u16, ty: ValueType| {
			let variable = resolver.get_local(pos, ty);

			LoadVariableTask(variable)
		};

		match inst {
			Inst::ALOAD(v) => load(*v as u16, ValueType::Reference),
			Inst::ALOAD_W(v) => load(*v, ValueType::Reference),
			Inst::ALOAD0 => load(0, ValueType::Reference),
			Inst::ALOAD1 => load(1, ValueType::Reference),
			Inst::ALOAD2 => load(2, ValueType::Reference),
			Inst::ALOAD3 => load(3, ValueType::Reference),
			Inst::FLOAD(v) => load(*v as u16, ValueType::Float),
			Inst::FLOAD_W(v) => load(*v, ValueType::Float),
			Inst::FLOAD0 => load(0, ValueType::Float),
			Inst::FLOAD1 => load(1, ValueType::Float),
			Inst::FLOAD2 => load(2, ValueType::Float),
			Inst::FLOAD3 => load(3, ValueType::Float),
			Inst::ILOAD(v) => load(*v as u16, ValueType::Int),
			Inst::ILOAD_W(v) => load(*v, ValueType::Int),
			Inst::ILOAD0 => load(0, ValueType::Int),
			Inst::ILOAD1 => load(1, ValueType::Int),
			Inst::ILOAD2 => load(2, ValueType::Int),
			Inst::ILOAD3 => load(3, ValueType::Int),
			Inst::DLOAD(v) => load(*v as u16, ValueType::Double),
			Inst::DLOAD_W(v) => load(*v, ValueType::Double),
			Inst::DLOAD0 => load(0, ValueType::Double),
			Inst::DLOAD1 => load(1, ValueType::Double),
			Inst::DLOAD2 => load(2, ValueType::Double),
			Inst::DLOAD3 => load(3, ValueType::Double),
			Inst::LLOAD(v) => load(*v as u16, ValueType::Long),
			Inst::LLOAD_W(v) => load(*v, ValueType::Long),
			Inst::LLOAD0 => load(0, ValueType::Long),
			Inst::LLOAD1 => load(1, ValueType::Long),
			Inst::LLOAD2 => load(2, ValueType::Long),
			Inst::LLOAD3 => load(3, ValueType::Long),
			_ => {
				panic!("what")
			}
		}
	}

	pub fn get_type(&self) -> ValueType {
		self.0.ty
	}

	pub fn compile<'b, 'a>(&self, bc: &mut BlockCompiler<'b, 'a>)  {
		let ptr = match self.0.data {
			VarData::Local(id) => {
				bc.get_local(id).value
			}
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
	pub fn resolve(inst: &Inst, resolver: &mut BlockResolver) -> StoreVariableTask {
		let mut store = |pos: u16, ty: ValueType| {
			let var = resolver.get_local(pos, ty);
			StoreVariableTask {
				var,
			}
		};

		match inst {
			Inst::ASTORE(v) => store(*v as u16, ValueType::Reference),
			Inst::ASTORE_W(v) => store(*v, ValueType::Reference),
			Inst::ASTORE0 => store(0, ValueType::Reference),
			Inst::ASTORE1 => store(1, ValueType::Reference),
			Inst::ASTORE2 => store(2, ValueType::Reference),
			Inst::ASTORE3 => store(3, ValueType::Reference),
			Inst::FSTORE(v) => store(*v as u16, ValueType::Float),
			Inst::FSTORE_W(v) => store(*v, ValueType::Float),
			Inst::FSTORE0 => store(0, ValueType::Float),
			Inst::FSTORE1 => store(1, ValueType::Float),
			Inst::FSTORE2 => store(2, ValueType::Float),
			Inst::FSTORE3 => store(3, ValueType::Float),
			Inst::ISTORE(v) => store(*v as u16, ValueType::Int),
			Inst::ISTORE_W(v) => store(*v, ValueType::Int),
			Inst::ISTORE0 => store(0, ValueType::Int),
			Inst::ISTORE1 => store(1, ValueType::Int),
			Inst::ISTORE2 => store(2, ValueType::Int),
			Inst::ISTORE3 => store(3, ValueType::Int),
			Inst::DSTORE(v) => store(*v as u16, ValueType::Double),
			Inst::DSTORE_W(v) => store(*v, ValueType::Double),
			Inst::DSTORE0 => store(0, ValueType::Double),
			Inst::DSTORE1 => store(1, ValueType::Double),
			Inst::DSTORE2 => store(2, ValueType::Double),
			Inst::DSTORE3 => store(3, ValueType::Double),
			Inst::LSTORE(v) => store(*v as u16, ValueType::Long),
			Inst::LSTORE_W(v) => store(*v, ValueType::Long),
			Inst::LSTORE0 => store(0, ValueType::Long),
			Inst::LSTORE1 => store(1, ValueType::Long),
			Inst::LSTORE2 => store(2, ValueType::Long),
			Inst::LSTORE3 => store(3, ValueType::Long),
			_ => panic!("what"),
		}
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
	pub fn resolve(inst: &Inst, resolver: &mut BlockResolver) -> IncrementTask {
		let (index, amount) = match inst {
			Inst::IINC(index, amount) => {
				(*index as u16, *amount as i16)
			}
			Inst::IINC_W(index, amount) => {
				(*index, *amount)
			}
			_ => {
				panic!("what")
			}
		};

		let var = resolver.get_local(index, ValueType::Int);

		IncrementTask {
			var,
			amount
		}
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
		let amount = bc.int().const_int(unsafe {
			transmute::<i32, u32>(self.amount as i32)
		} as u64, false);

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
	pub ty: ValueType,
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
