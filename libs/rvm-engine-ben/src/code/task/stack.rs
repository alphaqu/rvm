use rvm_reader::StackInst;

use crate::thread::ThreadFrame;

#[derive(Debug)]
pub enum StackTask {
	DUP,
	DUP_X1,
	DUP_X2,
	DUP2,
	DUP2_X1,
	DUP2_X2,
	POP,
	POP2,
	SWAP,
}

impl StackTask {
	pub fn new(inst: &StackInst) -> StackTask {
		match inst {
			StackInst::DUP => StackTask::DUP,
			StackInst::DUP_X1 => StackTask::DUP_X1,
			StackInst::DUP_X2 => StackTask::DUP_X2,
			StackInst::DUP2 => StackTask::DUP2,
			StackInst::DUP2_X1 => StackTask::DUP2_X1,
			StackInst::DUP2_X2 => StackTask::DUP2_X2,
			StackInst::POP => StackTask::POP,
			StackInst::POP2 => StackTask::POP2,
			StackInst::SWAP => StackTask::SWAP,
		}
	}

	pub fn exec(&self, frame: &mut ThreadFrame) {
		match self {
			StackTask::DUP => {
				let value = frame.pop();
				frame.push(value);
				frame.push(value);
			}
			StackTask::DUP_X1 => {
				let value1 = frame.pop();
				let value2 = frame.pop();
				frame.push(value1);
				frame.push(value2);
				frame.push(value1);
			}
			StackTask::DUP_X2 => {
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
			StackTask::DUP2 => {
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
			StackTask::DUP2_X1 => {
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
			StackTask::DUP2_X2 => {
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
			StackTask::POP => {
				frame.pop();
			}
			StackTask::POP2 => {
				let value = frame.pop();
				if value.category_1() {
					frame.pop();
				}
			}
			StackTask::SWAP => {
				let value1 = frame.pop();
				let value2 = frame.pop();
				frame.push(value1);
				frame.push(value2);
			}
		}
	}
}
