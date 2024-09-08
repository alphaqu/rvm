use std::fmt::{Display, Formatter};

use crate::code::Executor;
use crate::thread::{BenFrameMut, ThreadFrame};
use crate::value::StackValue;
use rvm_core::{ArrayType, Kind, ObjectType, PrimitiveType, Type};
use rvm_reader::{ClassConst, ConstPtr};
use rvm_runtime::{Class, InstanceClass, Vm};

#[derive(Debug)]
pub struct ArrayCreateTask(pub PrimitiveType);

impl ArrayCreateTask {
	#[inline(always)]
	pub fn exec(&self, executor: &mut Executor) -> eyre::Result<()> {
		let mut frame = executor.current_frame();
		let length = frame.pop().to_int()?;

		let array = executor
			.runtime()
			.alloc_array(&Class::Primitive(self.0), length as u32)?;

		let mut frame = executor.current_frame();
		frame.push(StackValue::Reference(*array));
		Ok(())
	}
}

impl Display for ArrayCreateTask {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "newarray {}", self.0)
	}
}

#[derive(Debug)]
pub struct ArrayCreateRefTask(ObjectType);

impl ArrayCreateRefTask {
	pub fn new(ptr: &ConstPtr<ClassConst>, obj: &InstanceClass) -> ArrayCreateRefTask {
		let class = ptr.get(&obj.cp).unwrap();
		let name = class.name.get(&obj.cp).unwrap();
		ArrayCreateRefTask(ObjectType::new(name.to_string()))
	}

	pub fn exec(&self, executor: &mut Executor) -> eyre::Result<()> {
		let mut frame = executor.current_frame();
		let length = frame.pop().to_int()?;

		let mut ctx = executor.runtime();
		let component_id = ctx.resolve_class(&Type::Object(self.0.clone()))?;
		let component_class = ctx.vm.classes.get(component_id);
		let array = ctx.alloc_array(&component_class, length as u32)?;

		let mut frame = executor.current_frame();
		frame.push(StackValue::Reference(*array));

		Ok(())
	}
}

impl Display for ArrayCreateRefTask {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "anewarray {}", self.0)
	}
}

#[derive(Debug)]
pub struct ArrayLengthTask;

impl ArrayLengthTask {
	#[inline(always)]
	pub fn exec(&self, frame: &mut BenFrameMut) {
		let reference = frame.pop().to_ref().unwrap();
		let option = reference.to_array();
		let array = option.expect("Expected array");
		let length = array.length();
		frame.push(StackValue::Int(length));
	}
}

impl Display for ArrayLengthTask {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "arraylength")
	}
}

#[derive(Debug)]
pub struct ArrayLoadTask(pub Kind);

impl ArrayLoadTask {
	#[inline(always)]
	pub fn exec(&self, frame: &mut BenFrameMut) {
		let index = frame.pop().to_int().unwrap();

		let reference = frame.pop().to_ref().unwrap();
		let array = reference.to_array().unwrap();
		if array.component_kind() != self.0 {
			panic!("Array type does not match");
		}

		let value = array.get(index).expect("Out of bounds");
		frame.push(StackValue::from_any(value));
	}
}

impl Display for ArrayLoadTask {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "arrayload {}", self.0)
	}
}

#[derive(Debug)]
pub struct ArrayStoreTask(pub Kind);

impl ArrayStoreTask {
	#[inline(always)]
	pub fn exec(&self, frame: &mut BenFrameMut) -> eyre::Result<()> {
		let value = frame.pop();
		let value = value.convert(self.0)?;

		let index = frame.pop().to_int()?;

		let reference = frame.pop().to_ref()?;
		let array = reference.to_array().unwrap();
		if array.component_kind() != self.0 {
			panic!("Array type does not match");
		}

		array.set(index, value);
		Ok(())
	}
}

impl Display for ArrayStoreTask {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "arraystore {}", self.0)
	}
}
