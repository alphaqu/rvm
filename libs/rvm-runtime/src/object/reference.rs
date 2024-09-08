use crate::conversion::JavaTyped;
use crate::gc::{JavaHeader, JavaUser};
use crate::{ArrayRef, InstanceRef, Vm};
use rvm_core::{ArrayType, ObjectType, Type, Typed};
use rvm_gc::GcRef;
use std::fmt::{Debug, Formatter};
use std::mem::size_of;
use std::ops::Deref;
use std::ptr::null_mut;
use std::sync::Arc;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Reference(GcRef<JavaUser>);

unsafe impl Send for Reference {}

unsafe impl Sync for Reference {}

impl Reference {
	pub const HEADER_SIZE: usize = size_of::<u8>();
	pub const NULL: Reference = Reference(GcRef::NULL);

	pub fn new(gc: GcRef<JavaUser>) -> Reference {
		Reference(gc)
	}

	pub fn reference_kind(&self) -> Option<ReferenceKind> {
		if self.is_null() {
			return None;
		}

		Some(unsafe { self.reference_kind_unchecked() })
	}

	/// # Safety
	/// Dont be null
	pub unsafe fn reference_kind_unchecked(&self) -> ReferenceKind {
		self.0.header().kind()
	}

	pub fn is_null(self) -> bool {
		self.0.is_null()
	}

	pub fn to_instance(&self) -> Option<InstanceRef> {
		InstanceRef::try_new(*self)
	}

	pub fn to_array(&self) -> Option<ArrayRef> {
		ArrayRef::try_new(*self)
	}

	pub fn visit_refs(&self, visitor: impl FnMut(Reference)) {
		let header = &**self.0.header();
		unsafe {
			match header.kind() {
				ReferenceKind::Instance => InstanceRef::new_unchecked(*self).visit_refs(visitor),
				ReferenceKind::Array => ArrayRef::new_unchecked(*self).visit_refs(visitor),
			}
		}
	}

	pub fn map_refs(&self, mapper: impl FnMut(Reference) -> Reference) {
		let header = &**self.0.header();
		unsafe {
			match header.kind() {
				ReferenceKind::Instance => InstanceRef::new_unchecked(*self).map_refs(mapper),
				ReferenceKind::Array => ArrayRef::new_unchecked(*self).map_refs(mapper),
			}
		}
	}

	pub fn ty(&self, runtime: &Vm) -> Type {
		match self.reference_kind() {
			None => {
				// When its null, its just a regular untyped object
				Type::Object(ObjectType::Object())
			}
			Some(ReferenceKind::Instance) => {
				let instance = self.to_instance().unwrap();
				instance.ty(runtime)
			}
			Some(ReferenceKind::Array) => {
				let array = self.to_array().unwrap();
				array.ty(runtime)
			}
		}
	}
}

impl Deref for Reference {
	type Target = GcRef<JavaUser>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

#[derive(Debug, Eq, PartialEq)]
pub enum ReferenceKind {
	Instance,
	Array,
}

impl Debug for Reference {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let option = self.reference_kind();
		match option {
			None => write!(f, "null"),
			Some(ReferenceKind::Array) => write!(f, "arr{:?}", self.0),
			Some(ReferenceKind::Instance) => write!(f, "obj{:?}", self.0),
		}
	}
}

impl JavaTyped for Reference {
	fn java_type() -> Type {
		Type::Object(ObjectType::Object())
	}
}

impl Typed for Reference {
	fn ty() -> Type {
		Type::Object(ObjectType::Object())
	}
}
