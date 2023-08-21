use rvm_core::Kind;
use rvm_runtime::Value;

pub trait Args {
	fn get_kinds() -> Vec<Kind>;
}

impl<V: Value> Args for V {
	fn get_kinds() -> Vec<Kind> {
		vec![V::ty()]
	}
}
