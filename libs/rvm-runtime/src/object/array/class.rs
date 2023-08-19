use rvm_core::{Type};

pub struct ArrayClass {
	component: Type,
}

impl ArrayClass {
	pub fn new(component: Type) -> ArrayClass {
		ArrayClass { component }
	}

	pub fn component(&self) -> &Type {
		&self.component
	}
}
