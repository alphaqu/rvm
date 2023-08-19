use crate::thread::ThreadFrame;
use crate::value::StackValue;
use rvm_core::{Kind, ObjectType, PrimitiveType, Type};
use rvm_reader::{ClassConst, ConstPtr};
use rvm_runtime::{InstanceClass, Runtime};
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct ArrayCreateTask(pub PrimitiveType);
impl ArrayCreateTask {
	pub fn exec(&self, runtime: &Runtime, frame: &mut ThreadFrame) {
		let length = frame.pop().to_int();
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
		ArrayCreateRefTask(ObjectType(name.to_string()))
	}

	pub fn exec(&self, runtime: &Runtime, frame: &mut ThreadFrame) {
		let length = frame.pop().to_int();

		let id = runtime
			.class_loader
			.get_class_id(&Type::Object(self.0.clone()));
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
	pub fn exec(&self, frame: &mut ThreadFrame) {
		let reference = frame.pop().to_ref();
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
	pub fn exec(&self, frame: &mut ThreadFrame) {
		let index = frame.pop().to_int();

		let reference = frame.pop().to_ref();
		let array = reference.to_array().unwrap();
		if array.kind() != self.0 {
			panic!("Array type does not match");
		}

		let value = array.get(index).expect("Out of bounds");
		frame.push(StackValue::from_dyn(value));
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
	pub fn exec(&self, frame: &mut ThreadFrame) {
		let value = frame.pop();
		let value = value.convert(self.0).expect("unable to conver");

		let index = frame.pop().to_int();

		let reference = frame.pop().to_ref();
		let array = reference.to_array().unwrap();
		if array.kind() != self.0 {
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
