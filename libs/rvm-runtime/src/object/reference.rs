use crate::{AnyArray, InstanceReference};
use rvm_core::{ObjectType, Type, Typed};
use std::fmt::{Debug, Formatter};
use std::mem::size_of;
use std::ptr::null_mut;

#[derive(Copy, Clone, PartialEq)]
#[repr(transparent)]
pub struct Reference(pub *mut u8);

unsafe impl Send for Reference {}

unsafe impl Sync for Reference {}

impl Reference {
	pub const HEADER_SIZE: usize = size_of::<u8>();
	pub const NULL: Reference = Reference(null_mut());

	pub fn reference_kind(&self) -> Option<ReferenceKind> {
		if self.is_null() {
			return None;
		}

		Some(unsafe { self.reference_kind_unchecked() })
	}

	pub unsafe fn reference_kind_unchecked(&self) -> ReferenceKind {
		let i = unsafe { *self.0 };
		match i {
			1 => ReferenceKind::Instance,
			2 => ReferenceKind::Array,
			_ => panic!("Corrupted kind {i}",),
		}
	}

	pub fn is_null(self) -> bool {
		self.0.is_null()
	}

	pub fn to_class(&self) -> Option<InstanceReference> {
		InstanceReference::try_new(*self)
	}

	pub fn to_array(&self) -> Option<AnyArray> {
		AnyArray::try_new(*self)
	}

	pub fn visit_refs(&self, visitor: impl FnMut(Reference)) {
		unsafe {
			match self.reference_kind() {
				Some(ReferenceKind::Instance) => {
					InstanceReference::new_unchecked(*self).visit_refs(visitor)
				}
				Some(ReferenceKind::Array) => AnyArray::new_unchecked(*self).visit_refs(visitor),
				_ => {}
			}
		}
	}

	pub fn map_refs(&self, mapper: impl FnMut(Reference) -> Reference) {
		unsafe {
			match self.reference_kind() {
				Some(ReferenceKind::Instance) => {
					InstanceReference::new_unchecked(*self).map_refs(mapper)
				}
				Some(ReferenceKind::Array) => AnyArray::new_unchecked(*self).map_refs(mapper),
				_ => {}
			}
		}
	}
}

#[derive(Debug, Eq, PartialEq)]
pub enum ReferenceKind {
	Instance,
	Array,
}

impl Debug for Reference {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self.0)
	}
}

impl Typed for Reference {
	fn ty() -> Type {
		Type::Object(ObjectType::Object())
	}
}
