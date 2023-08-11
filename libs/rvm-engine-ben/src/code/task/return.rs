use rvm_core::StackKind;
use rvm_reader::ReturnInst;
#[derive(Debug)]

pub struct ReturnTask {
	pub kind: Option<StackKind>,
}

impl ReturnTask {
	pub fn new(inst: &ReturnInst) -> ReturnTask {
		ReturnTask {
			kind: inst.value.clone(),
		}
	}
}
