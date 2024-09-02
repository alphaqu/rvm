use rvm_core::{ArrayType, Id, Kind, PrimitiveType, StorageValue, Type, Typed};
use std::intrinsics::transmute;
use std::marker::PhantomData;
use std::mem::size_of;
use std::ops::Deref;
use std::sync::Arc;

use crate::{
	read_arr, write_arr, AnyValue, Castable, Class, Reference, ReferenceKind, Returnable, Runtime,
	Value,
};

#[derive(Copy, Clone)]
pub struct AnyArray {
	reference: Reference,
}

impl Deref for AnyArray {
	type Target = Reference;

	fn deref(&self) -> &Self::Target {
		&self.reference
	}
}

impl AnyArray {
	pub const KIND_SIZE: usize = size_of::<Kind>();
	pub const LENGTH_SIZE: usize = size_of::<i32>();
	pub const REF_ID_SIZE: usize = size_of::<<Class as StorageValue>::Idx>();

	pub fn new(reference: Reference) -> AnyArray {
		Self::try_new(reference).unwrap()
	}

	pub fn try_new(reference: Reference) -> Option<AnyArray> {
		if reference.reference_kind() != Some(ReferenceKind::Array) {
			return None;
		}

		Some(unsafe { Self::new_unchecked(reference) })
	}

	/// # Safety
	/// The caller must ensure that the reference is not null, and that its kind is Array.
	pub unsafe fn new_unchecked(reference: Reference) -> AnyArray {
		AnyArray { reference }
	}

	pub fn header_size(kind: Kind) -> usize {
		let mut size = Reference::HEADER_SIZE + Self::KIND_SIZE + Self::LENGTH_SIZE;
		if matches!(kind, Kind::Reference) {
			size += Self::REF_ID_SIZE;
		}
		size
	}
	pub fn size(kind: Kind, length: i32) -> usize {
		Self::header_size(kind) + (kind.size() * length as usize)
	}

	/// Allocates a new array
	pub unsafe fn allocate_primitive(
		reference: Reference,
		kind: PrimitiveType,
		length: i32,
	) -> AnyArray {
		Self::allocate(reference, kind.kind(), length);
		AnyArray::new_unchecked(reference)
	}

	pub unsafe fn allocate_ref(
		reference: Reference,
		component: Id<Class>,
		length: i32,
	) -> AnyArray {
		Self::allocate(reference, Kind::Reference, length);
		write_arr(
			reference
				.0
				.add(Reference::HEADER_SIZE + Self::KIND_SIZE + Self::LENGTH_SIZE),
			component.idx().to_le_bytes(),
		);
		AnyArray::new_unchecked(reference)
	}

	unsafe fn allocate(reference: Reference, kind: Kind, length: i32) {
		reference.0.write(2);
		reference.0.add(Reference::HEADER_SIZE).write(kind as u8);
		write_arr(
			reference.0.add(Reference::HEADER_SIZE + Self::KIND_SIZE),
			length.to_le_bytes(),
		);
	}

	pub fn kind(&self) -> Kind {
		unsafe { transmute(self.reference.0.add(Reference::HEADER_SIZE).read()) }
	}

	pub fn length(&self) -> i32 {
		unsafe {
			i32::from_le_bytes(read_arr(
				self.reference
					.0
					.add(Reference::HEADER_SIZE + Self::KIND_SIZE),
			))
		}
	}

	pub fn class(&self) -> Option<Id<Class>> {
		match self.kind() {
			Kind::Reference => unsafe {
				let idx = <Class as StorageValue>::Idx::from_le_bytes(read_arr(
					self.reference
						.0
						.add(Reference::HEADER_SIZE + Self::KIND_SIZE + Self::LENGTH_SIZE),
				));
				Some(Id::new(idx as usize))
			},
			_ => None,
		}
	}

	pub fn get(&self, index: i32) -> Option<AnyValue> {
		Some(match self.kind() {
			Kind::Reference => AnyValue::Reference(Array::<Reference>::new(*self).get(index)?),
			Kind::Boolean => AnyValue::Boolean(Array::<bool>::new(*self).get(index)?),
			Kind::Char => AnyValue::Char(Array::<u16>::new(*self).get(index)?),
			Kind::Float => AnyValue::Float(Array::<f32>::new(*self).get(index)?),
			Kind::Double => AnyValue::Double(Array::<f64>::new(*self).get(index)?),
			Kind::Byte => AnyValue::Byte(Array::<i8>::new(*self).get(index)?),
			Kind::Short => AnyValue::Short(Array::<i16>::new(*self).get(index)?),
			Kind::Int => AnyValue::Int(Array::<i32>::new(*self).get(index)?),
			Kind::Long => AnyValue::Long(Array::<i64>::new(*self).get(index)?),
		})
	}

	pub fn set(&self, index: i32, value: AnyValue) {
		if value.kind() != self.kind() {
			panic!();
		}
		match value {
			AnyValue::Reference(v) => Array::<Reference>::new(*self).set(index, v),
			AnyValue::Boolean(v) => Array::<bool>::new(*self).set(index, v),
			AnyValue::Char(v) => Array::<u16>::new(*self).set(index, v),
			AnyValue::Float(v) => Array::<f32>::new(*self).set(index, v),
			AnyValue::Double(v) => Array::<f64>::new(*self).set(index, v),
			AnyValue::Byte(v) => Array::<i8>::new(*self).set(index, v),
			AnyValue::Short(v) => Array::<i16>::new(*self).set(index, v),
			AnyValue::Int(v) => Array::<i32>::new(*self).set(index, v),
			AnyValue::Long(v) => Array::<i64>::new(*self).set(index, v),
		};
	}

	pub fn visit_refs(&self, mut visitor: impl FnMut(Reference)) {
		if let Some(array) = Array::<Reference>::try_new(*self) {
			for i in 0..array.length() {
				if let Some(reference) = array.get(i) {
					visitor(reference);
				}
			}
		}
	}

	pub fn map_refs(&self, mut mapper: impl FnMut(Reference) -> Reference) {
		if let Some(mut array) = Array::<Reference>::try_new(*self) {
			for i in 0..array.length() {
				if let Some(reference) = array.get(i) {
					array.set(i, mapper(reference));
				}
			}
		}
	}

	pub fn typed<V: Value>(self) -> Array<V> {
		Array::new(self)
	}
}

#[derive(Copy, Clone)]
pub struct Array<V: Value> {
	array: AnyArray,
	_p: PhantomData<V>,
}

impl<V: Value> Array<V> {
	pub fn new(array: AnyArray) -> Array<V> {
		Self::try_new(array).unwrap()
	}

	pub fn try_new(array: AnyArray) -> Option<Array<V>> {
		if array.kind() == V::kind() {
			Some(Array {
				array,
				_p: Default::default(),
			})
		} else {
			None
		}
	}

	pub fn get(&self, index: i32) -> Option<V> {
		if index >= 0 && index < self.array.length() {
			unsafe { Some(self.data().add(index as usize).read()) }
		} else {
			None
		}
	}

	pub fn set(&mut self, index: i32, value: V) {
		if index >= 0 && index < self.array.length() {
			unsafe {
				self.data_mut().add(index as usize).write(value);
			}
		}
	}

	pub fn length(&self) -> i32 {
		self.array.length()
	}

	pub unsafe fn data(&self) -> *const V {
		(self.array.0.add(AnyArray::header_size(V::kind()))) as *const V
	}

	pub unsafe fn data_mut(&mut self) -> *mut V {
		(self.array.0.add(AnyArray::header_size(V::kind()))) as *mut V
	}
}

impl<V: Value> Castable for Array<V> {
	fn cast_from(runtime: &Arc<Runtime>, value: AnyValue) -> Self {
		let reference = Reference::cast_from(runtime, value);
		Array::new(reference.to_array().unwrap())
	}
}

impl<V: Value> Deref for Array<V> {
	type Target = AnyArray;

	fn deref(&self) -> &Self::Target {
		&self.array
	}
}

impl<V: Value + Typed> Typed for Array<V> {
	fn ty() -> Type {
		Type::Array(ArrayType::from_component(V::ty()))
	}
}

impl TryFrom<AnyValue> for AnyArray {
	type Error = AnyValue;

	fn try_from(value: AnyValue) -> Result<Self, Self::Error> {
		let reference: Reference = AnyValue::try_into(value)?;
		Ok(AnyArray::try_new(reference).ok_or(value)?)
	}
}

impl Into<AnyValue> for AnyArray {
	fn into(self) -> AnyValue {
		AnyValue::Reference(self.reference)
	}
}

impl<V: Value> TryFrom<AnyValue> for Array<V> {
	type Error = AnyValue;

	fn try_from(value: AnyValue) -> Result<Self, Self::Error> {
		let array: AnyArray = AnyValue::try_into(value)?;
		Array::try_new(array).ok_or(value)
	}
}

impl<V: Value> Into<AnyValue> for Array<V> {
	fn into(self) -> AnyValue {
		AnyValue::Reference(self.array.reference)
	}
}
