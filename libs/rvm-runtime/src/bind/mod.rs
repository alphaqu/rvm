use std::collections::HashMap;
use crate::{MethodBinding, MethodIdentifier};

pub struct Binder {
	pub bindings: HashMap<MethodIdentifier, MethodBinding>,
}