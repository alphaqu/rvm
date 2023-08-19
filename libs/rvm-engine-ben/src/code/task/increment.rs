use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct IncrementTask {
	pub local: u16,
	pub increment: i16,
}

impl Display for IncrementTask {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "INC {}+{}", self.local, self.increment)
	}
}
