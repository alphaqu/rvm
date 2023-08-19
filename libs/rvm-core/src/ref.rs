use std::fmt::{Debug, Formatter};
use std::ptr::null_mut;

#[derive(Copy, Clone, PartialEq)]
#[repr(transparent)]
pub struct Reference(pub *mut u8);

unsafe impl Send for Reference {}

unsafe impl Sync for Reference {}
impl Reference {
	pub const NULL: Reference = Reference(null_mut());

	pub fn kind(&self) -> ReferenceKind {
		let i = unsafe {
			let i1 = *self.0;
			i1
		};
		match i {
			1 => ReferenceKind::Class,
			2 => ReferenceKind::Array,
			_ => panic!("Corrupted kind {i}",),
		}
	}

	pub fn is_null(self) -> bool {
		self.0.is_null()
	}
}

#[derive(Debug)]
pub enum ReferenceKind {
	Class,
	Array,
}

impl Debug for Reference {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self.0)
	}
}
