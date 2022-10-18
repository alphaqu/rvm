use std::sync::{Arc, Weak};

#[derive(Clone, Debug)]
pub struct RootHandle {
	pub rc: Arc<()>
}

impl RootHandle {
	pub fn new() -> RootHandle {
		RootHandle {
			rc: Arc::new(())
		}
	}

	pub fn weak(&self) -> Weak<()> {
		Arc::downgrade(&self.rc)
	}
}