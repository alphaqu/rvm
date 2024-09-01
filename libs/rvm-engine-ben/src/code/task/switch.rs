use crate::thread::ThreadFrame;
use rvm_reader::TableSwitchInst;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct SwitchTableTask {
	pub low: i32,
	pub high: i32,
	pub default_jump: i32,
	pub jumps: Vec<i32>,
}

impl SwitchTableTask {
	pub fn new(inst: &TableSwitchInst) -> SwitchTableTask {
		SwitchTableTask {
			low: inst.low,
			high: inst.high,
			default_jump: inst.default_offset,
			jumps: inst.offsets.clone(),
		}
	}

	pub fn exec(&self, frame: &mut ThreadFrame) -> i32 {
		let value = frame.pop().to_int();
		let idx = if value < self.low || value > self.high {
			return self.default_jump;
		} else {
			value - self.low
		};

		self.jumps[idx as usize]
	}
}

impl Display for SwitchTableTask {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"tableswitch range:{}-{} def:{} targets:{:?}",
			self.low, self.high, self.default_jump, self.jumps
		)
	}
}
