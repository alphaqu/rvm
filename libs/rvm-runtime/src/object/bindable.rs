use crate::{Value, ValueCell, Vm};
use rvm_core::Type;

pub trait Bindable {
	type Cell;
	type Value: Value;
	fn ty() -> Type;
	fn bind(vm: &Vm, value: ValueCell<Self::Value>) -> Self::Cell;
}
