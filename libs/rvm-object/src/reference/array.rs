use crate::reference::Object;
use crate::value::{read_arr, write_arr};
use crate::{Class, DynValue, Value};
use rvm_core::{Id, Kind, PrimitiveType, Reference, StorageValue};
use std::intrinsics::transmute;
use std::marker::PhantomData;
use std::mem::size_of;
use std::ops::Deref;

#[derive(Copy, Clone)]
pub struct AnyArrayObject {
	pub(super) reference: Reference,
}

impl Deref for AnyArrayObject {
	type Target = Reference;

	fn deref(&self) -> &Self::Target {
		&self.reference
	}
}

impl AnyArrayObject {
	pub const KIND_SIZE: usize = size_of::<Kind>();
	pub const LENGTH_SIZE: usize = size_of::<i32>();
	pub const REF_ID_SIZE: usize = size_of::<<Class as StorageValue>::Idx>();

	pub fn header_size(kind: Kind) -> usize {
		let mut size = Object::HEADER_SIZE + Self::KIND_SIZE + Self::LENGTH_SIZE;
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
	) -> AnyArrayObject {
		Self::allocate(reference, kind.kind(), length);
		AnyArrayObject { reference }
	}

	pub unsafe fn allocate_ref(
		reference: Reference,
		component: Id<Class>,
		length: i32,
	) -> AnyArrayObject {
		Self::allocate(reference, Kind::Reference, length);
		write_arr(
			reference
				.0
				.add(Object::HEADER_SIZE + Self::KIND_SIZE + Self::LENGTH_SIZE),
			component.idx().to_le_bytes(),
		);
		AnyArrayObject { reference }
	}

	unsafe fn allocate(reference: Reference, kind: Kind, length: i32) {
		reference.0.write(2);
		reference.0.add(Object::HEADER_SIZE).write(kind as u8);
		write_arr(
			reference.0.add(Object::HEADER_SIZE + Self::KIND_SIZE),
			length.to_le_bytes(),
		);
	}

	pub fn kind(&self) -> Kind {
		unsafe { transmute(self.reference.0.add(Object::HEADER_SIZE).read()) }
	}

	pub fn length(&self) -> i32 {
		unsafe {
			i32::from_le_bytes(read_arr(
				self.reference.0.add(Object::HEADER_SIZE + Self::KIND_SIZE),
			))
		}
	}

	pub fn class(&self) -> Option<Id<Class>> {
		match self.kind() {
			Kind::Reference => unsafe {
				let idx = <Class as StorageValue>::Idx::from_le_bytes(read_arr(
					self.reference
						.0
						.add(Object::HEADER_SIZE + Self::KIND_SIZE + Self::LENGTH_SIZE),
				));
				Some(Id::new(idx as usize))
			},
			_ => None,
		}
	}

	pub fn get(&self, index: i32) -> Option<DynValue> {
		Some(match self.kind() {
			Kind::Reference => {
				DynValue::Reference(Array::<Reference>::new(self.clone()).get(index)?)
			}
			Kind::Boolean => DynValue::Boolean(Array::<bool>::new(self.clone()).get(index)?),
			Kind::Char => DynValue::Char(Array::<u16>::new(self.clone()).get(index)?),
			Kind::Float => DynValue::Float(Array::<f32>::new(self.clone()).get(index)?),
			Kind::Double => DynValue::Double(Array::<f64>::new(self.clone()).get(index)?),
			Kind::Byte => DynValue::Byte(Array::<i8>::new(self.clone()).get(index)?),
			Kind::Short => DynValue::Short(Array::<i16>::new(self.clone()).get(index)?),
			Kind::Int => DynValue::Int(Array::<i32>::new(self.clone()).get(index)?),
			Kind::Long => DynValue::Long(Array::<i64>::new(self.clone()).get(index)?),
		})
	}

	pub fn set(&self, index: i32, value: DynValue) {
		if value.kind() != self.kind() {
			panic!();
		}
		match value {
			DynValue::Reference(v) => Array::<Reference>::new(self.clone()).set(index, v),
			DynValue::Boolean(v) => Array::<bool>::new(self.clone()).set(index, v),
			DynValue::Char(v) => Array::<u16>::new(self.clone()).set(index, v),
			DynValue::Float(v) => Array::<f32>::new(self.clone()).set(index, v),
			DynValue::Double(v) => Array::<f64>::new(self.clone()).set(index, v),
			DynValue::Byte(v) => Array::<i8>::new(self.clone()).set(index, v),
			DynValue::Short(v) => Array::<i16>::new(self.clone()).set(index, v),
			DynValue::Int(v) => Array::<i32>::new(self.clone()).set(index, v),
			DynValue::Long(v) => Array::<i64>::new(self.clone()).set(index, v),
		};
	}

	pub fn visit_refs(&self, mut visitor: impl FnMut(Reference)) {
		if let Some(array) = Array::<Reference>::try_new(self.clone()) {
			for i in 0..array.length() {
				if let Some(reference) = array.get(i) {
					visitor(reference);
				}
			}
		}
	}

	pub fn map_refs(&self, mut mapper: impl FnMut(Reference) -> Reference) {
		if let Some(mut array) = Array::<Reference>::try_new(self.clone()) {
			for i in 0..array.length() {
				if let Some(reference) = array.get(i) {
					array.set(i, mapper(reference));
				}
			}
		}
	}
}

#[derive(Copy, Clone)]
pub struct Array<V: Value> {
	array: AnyArrayObject,
	_p: PhantomData<V>,
}

impl<V: Value> Array<V> {
	pub fn new(array: AnyArrayObject) -> Array<V> {
		Self::try_new(array).unwrap()
	}

	pub fn try_new(array: AnyArrayObject) -> Option<Array<V>> {
		if array.kind() == V::ty() {
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
		(self.array.0.add(AnyArrayObject::header_size(V::ty()))) as *const V
	}

	pub unsafe fn data_mut(&mut self) -> *mut V {
		(self.array.0.add(AnyArrayObject::header_size(V::ty()))) as *mut V
	}
}

impl<V: Value> Deref for Array<V> {
	type Target = AnyArrayObject;

	fn deref(&self) -> &Self::Target {
		&self.array
	}
}

impl TryFrom<DynValue> for AnyArrayObject {
	type Error = DynValue;

	fn try_from(value: DynValue) -> Result<Self, Self::Error> {
		let reference: Reference = DynValue::try_into(value)?;
		Ok(*Object::new(reference).as_array().ok_or(value)?)
	}
}
impl Into<DynValue> for AnyArrayObject {
	fn into(self) -> DynValue {
		DynValue::Reference(self.reference)
	}
}

impl<V: Value> TryFrom<DynValue> for Array<V> {
	type Error = DynValue;

	fn try_from(value: DynValue) -> Result<Self, Self::Error> {
		let array: AnyArrayObject = DynValue::try_into(value)?;
		Array::try_new(array).ok_or(value)
	}
}

impl<V: Value> Into<DynValue> for Array<V> {
	fn into(self) -> DynValue {
		DynValue::Reference(self.array.reference)
	}
}
