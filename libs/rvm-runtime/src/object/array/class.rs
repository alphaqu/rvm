use rvm_core::{Id, Type};

use crate::Class;

pub struct ArrayClass {
	pub id: Id<Class>,
	pub component: Type,
	pub component_id: Option<Id<Class>>,
}

impl ArrayClass {
	pub fn new(id: Id<Class>, component: Type, component_id: Option<Id<Class>>) -> ArrayClass {
		if component.kind().is_ref() && component_id.is_none() {
			panic!("Reference array without a component id");
		}
		ArrayClass {
			id,
			component,
			component_id,
		}
	}
}
