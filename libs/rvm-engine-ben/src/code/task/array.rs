use std::fmt::{Display, Formatter};

use rvm_core::{Kind, ObjectType, PrimitiveType, Type};
use rvm_reader::{ClassConst, ConstPtr};
use rvm_runtime::{InstanceClass, Runtime};

use crate::thread::ThreadFrame;
use crate::value::StackValue;

#[derive(Debug)]
pub struct ArrayCreateTask(pub PrimitiveType);

impl ArrayCreateTask {
	#[inline(always)]
	pub fn exec(&self, runtime: &Runtime, frame: &mut ThreadFrame) {
		let length = frame.pop().to_int().unwrap();
		let array = runtime.gc.lock().allocate_array(self.0, length).unwrap();
		frame.push(StackValue::Reference(*array));
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

	#[inline(always)]
	pub fn exec(&self, runtime: &Runtime, frame: &mut ThreadFrame) {
		let length = frame.pop().to_int().unwrap();

		let id = runtime.classes.resolve(&Type::Object(self.0.clone()));
		let array = runtime.gc.lock().allocate_ref_array(id, length).unwrap();

		frame.push(StackValue::Reference(*array));
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
	pub fn exec(&self, frame: &mut ThreadFrame) {
		let reference = frame.pop().to_ref().unwrap();
		let option = reference.to_array();
		let array = option.unwrap();
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
	pub fn exec(&self, frame: &mut ThreadFrame) {
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
	pub fn exec(&self, frame: &mut ThreadFrame) {
		let value = frame.pop();
		let value = value.convert(self.0).expect("unable to conver");

		let index = frame.pop().to_int().unwrap();

		let reference = frame.pop().to_ref().unwrap();
		let array = reference.to_array().unwrap();
		if array.component_kind() != self.0 {
			panic!("Array type does not match");
		}

		array.set(index, value);
	}
}

impl Display for ArrayStoreTask {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "arraystore {}", self.0)
	}
}
