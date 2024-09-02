use std::fmt::{Display, Formatter};

use rvm_reader::StackInst;

use crate::thread::ThreadFrame;

#[derive(Debug)]
pub enum StackTask {
	Dup,
	DupX1,
	DupX2,
	Dup2,
	Dup2X1,
	Dup2X2,
	Pop,
	Pop2,
	Swap,
}

impl Display for StackTask {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self)
	}
}

impl StackTask {
	pub fn new(inst: &StackInst) -> StackTask {
		match inst {
			StackInst::Dup => StackTask::Dup,
			StackInst::DupX1 => StackTask::DupX1,
			StackInst::DupX2 => StackTask::DupX2,
			StackInst::Dup2 => StackTask::Dup2,
			StackInst::Dup2X1 => StackTask::Dup2X1,
			StackInst::Dup2X2 => StackTask::Dup2X2,
			StackInst::Pop => StackTask::Pop,
			StackInst::Pop2 => StackTask::Pop2,
			StackInst::Swap => StackTask::Swap,
		}
	}

	pub fn exec(&self, frame: &mut ThreadFrame) {
		match self {
			StackTask::Dup => {
				let value = frame.pop();
				frame.push(value);
				frame.push(value);
			}
			StackTask::DupX1 => {
				let value1 = frame.pop();
				let value2 = frame.pop();
				frame.push(value1);
				frame.push(value2);
				frame.push(value1);
			}
			StackTask::DupX2 => {
				let value1 = frame.pop();
				let value2 = frame.pop();
				if value2.category_1() {
					let value3 = frame.pop();
					frame.push(value1);
					frame.push(value3);
					frame.push(value2);
					frame.push(value1);
				} else {
					frame.push(value1);
					frame.push(value2);
					frame.push(value1);
				}
			}
			StackTask::Dup2 => {
				let value1 = frame.pop();
				if value1.category_1() {
					let value2 = frame.pop();
					frame.push(value2);
					frame.push(value1);
					frame.push(value2);
					frame.push(value1);
				} else {
					frame.push(value1);
					frame.push(value1);
				}
			}
			StackTask::Dup2X1 => {
				let value1 = frame.pop();
				let value2 = frame.pop();
				if value1.category_1() {
					let value3 = frame.pop();
					frame.push(value2);
					frame.push(value1);
					frame.push(value3);
					frame.push(value2);
					frame.push(value1);
				} else {
					frame.push(value1);
					frame.push(value2);
					frame.push(value1);
				}
			}
			StackTask::Dup2X2 => {
				// form1 v4  v3  v2  v1
				// form3     v3* v2  v1
				// form2     v3  v2  v1*
				// form4         v2* v1*
				let value1 = frame.pop();
				let value2 = frame.pop();

				if value1.category_1() {
					let value3 = frame.pop();
					if value3.category_1() {
						// form1
						let value4 = frame.pop();
						frame.push(value2);
						frame.push(value1);
						frame.push(value4);
						frame.push(value3);
						frame.push(value2);
						frame.push(value1);
					} else {
						// form3
						frame.push(value2);
						frame.push(value1);
						frame.push(value3);
						frame.push(value2);
						frame.push(value1);
					}
				} else if value2.category_1() {
					// form2
					let value3 = frame.pop();
					frame.push(value1);
					frame.push(value3);
					frame.push(value2);
					frame.push(value1);
				} else {
					// form4
					frame.push(value1);
					frame.push(value2);
					frame.push(value1);
				}
			}
			StackTask::Pop => {
				frame.pop();
			}
			StackTask::Pop2 => {
				let value = frame.pop();
				if value.category_1() {
					frame.pop();
				}
			}
			StackTask::Swap => {
				let value1 = frame.pop();
				let value2 = frame.pop();
				frame.push(value1);
				frame.push(value2);
			}
		}
	}
}
