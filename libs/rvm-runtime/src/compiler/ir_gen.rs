#[derive(Default)]
pub struct IrNameGen {
	i: u128,
}

impl IrNameGen {
	pub fn next(&mut self) -> String {
		let value = self.i;
		self.i += 1;
		format!("v{value}")
	}
}
