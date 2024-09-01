use crate::{AnyInstance, AnyValue, Value};
use rvm_core::Kind;
use std::ops::{Deref, DerefMut};

pub struct DynField {
	pub(super) instance: AnyInstance,
	pub(super) offset: u32,
	pub(super) kind: Kind,
}

impl DynField {
	pub fn get(&self) -> AnyValue {
		unsafe { self.instance.raw.get_any(self.offset as usize, self.kind) }
	}

	pub fn set(&self, value: AnyValue) {
		if self.kind != value.kind() {
			panic!("Invalid type");
		}

		unsafe {
			self.instance.raw.put_any(self.offset as usize, value);
		}
	}

	pub fn typed<V: Value>(self) -> TypedField<V> {
		if self.kind != V::kind() {
			panic!("Invalid type");
		}

		let ptr = unsafe { self.instance.raw.get_mut_ptr::<V>(self.offset as usize) };

		TypedField { ptr }
	}
}

pub struct TypedField<V: Value> {
	pub(super) ptr: *mut V,
}

impl<V: Value> Clone for TypedField<V> {
	fn clone(&self) -> Self {
		*self
	}
}
impl<V: Value> Copy for TypedField<V> {}
impl<V: Value> Deref for TypedField<V> {
	type Target = V;

	fn deref(&self) -> &Self::Target {
		unsafe { &*(self.ptr as *const V) }
	}
}

impl<V: Value> DerefMut for TypedField<V> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		unsafe { &mut *self.ptr }
	}
}
