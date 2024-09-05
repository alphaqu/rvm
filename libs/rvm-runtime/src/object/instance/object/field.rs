use crate::{AnyInstance, AnyValue, Field, FieldLayout, UnionValue, Value};
use rvm_core::{Id, Kind};
use std::ops::{Deref, DerefMut, Index};

#[derive(Copy, Clone)]
pub struct FieldTable<'a> {
	layout: &'a FieldLayout,
	fields: *mut u8,
}

impl<'a> FieldTable<'a> {
	/// # Safety
	/// Caller must ensure that the pointer is pointing to valid data.
	pub unsafe fn new(layout: &'a FieldLayout, fields: *mut u8) -> FieldTable {
		FieldTable { layout, fields }
	}

	pub fn by_id(&self, id: Id<Field>) -> DynField2 {
		let field = self.layout.get(id);

		DynField2 {
			ptr: unsafe { self.fields.add(field.offset as usize).cast::<UnionValue>() },
			kind: field.ty.kind(),
		}
	}
	pub fn by_id_typed<V: Value>(&self, id: Id<Field>) -> TypedField<V> {
		self.by_id(id).typed::<V>()
	}

	pub fn by_name(&self, name: &str) -> Option<DynField2> {
		let field = self.layout.get_id(name)?;
		Some(self.by_id(field))
	}

	pub fn by_name_typed<V: Value>(&self, name: &str) -> Option<TypedField<V>> {
		Some(self.by_name(name)?.typed::<V>())
	}
}

pub struct DynField2 {
	pub(super) ptr: *mut UnionValue,
	pub(super) kind: Kind,
}

impl DynField2 {
	pub fn get(&self) -> AnyValue {
		unsafe { AnyValue::read(self.ptr.read(), self.kind) }
	}

	pub fn set(&self, value: AnyValue) {
		if self.kind != value.kind() {
			panic!("Invalid type");
		}

		unsafe {
			value.write(self.ptr);
		}
	}

	pub fn typed<V: Value>(self) -> TypedField<V> {
		if self.kind != V::kind() {
			panic!("Invalid type");
		}

		unsafe {
			TypedField {
				ptr: V::cast_pointer(self.ptr),
			}
		}
	}
}

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
