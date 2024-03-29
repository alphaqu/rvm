use std::ops::Deref;

use crate::impl_constant;

#[derive(Clone, Debug)]
pub struct UTF8Const(pub(crate) String);

impl Deref for UTF8Const {
	type Target = String;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl_constant!(UTF8 UTF8Const);
