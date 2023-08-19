use rvm_core::{Id, Type};

use crate::Class;

pub struct ArrayClass {
	component: Type,
	component_id: Option<Id<Class>>,
}

impl ArrayClass {
	pub fn new(component: Type, component_id: Option<Id<Class>>) -> ArrayClass {
		if component.kind().is_ref() && component_id.is_none() {
			panic!("Reference array without a component id");
		}
		ArrayClass {
			component,
			component_id,
		}
	}

	pub fn component(&self) -> &Type {
		&self.component
	}

	pub fn component_id(&self) -> Option<Id<Class>> {
		self.component_id
	}
}
