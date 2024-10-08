use crate::conversion::{FromJava, JavaTyped, ToJava};
use crate::gc::{ArrayHeader, JavaHeader};
use crate::object::bindable::Bindable;
use crate::{
	read_arr, write_arr, AnyValue, Castable, Class, JavaKind, Reference, ReferenceKind, UnionValue,
	Value, ValueCell, Vm,
};
use eyre::ContextCompat;
use rvm_core::{
	ArrayType, CastTypeError, Id, Kind, ObjectType, PrimitiveType, StorageValue, Type, Typed,
};
use std::intrinsics::transmute;
use std::marker::PhantomData;
use std::mem::size_of;
use std::ops::Deref;
use std::sync::Arc;

#[derive(Copy, Clone)]
pub struct ArrayRef {
	reference: Reference,
}

impl Deref for ArrayRef {
	type Target = Reference;

	fn deref(&self) -> &Self::Target {
		&self.reference
	}
}

impl ArrayRef {
	//pub const KIND_SIZE: usize = size_of::<Kind>();
	//pub const LENGTH_SIZE: usize = size_of::<i32>();
	//pub const REF_ID_SIZE: usize = size_of::<<Class as StorageValue>::Idx>();

	pub fn new(reference: Reference) -> ArrayRef {
		Self::try_new(reference).unwrap()
	}

	pub fn try_new(reference: Reference) -> Option<ArrayRef> {
		if reference.reference_kind() != Some(ReferenceKind::Array) {
			return None;
		}

		Some(unsafe { Self::new_unchecked(reference) })
	}

	/// # Safety
	/// The caller must ensure that the reference is not null, and that its kind is Array.
	pub unsafe fn new_unchecked(reference: Reference) -> ArrayRef {
		ArrayRef { reference }
	}

	pub fn header(&self) -> &ArrayHeader {
		let JavaHeader::Array(header) = self.reference.header().user() else {
			panic!("Wrong header type");
		};
		header
	}

	pub fn component_kind(&self) -> Kind {
		self.header().kind
	}

	pub fn component_class(&self) -> Option<Id<Class>> {
		self.header().component_id
	}

	pub fn length(&self) -> i32 {
		self.header().length as i32
	}

	pub fn get(&self, index: i32) -> Option<AnyValue> {
		Some(match self.component_kind() {
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
		if value.kind() != self.component_kind() {
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

	pub fn ty(&self, runtime: &Vm) -> Type {
		let component_ty = match self.component_kind() {
			Kind::Reference => {
				let id = self.component_class().unwrap();
				runtime.classes.get(id).cloned_ty()
			}
			Kind::Boolean => Type::Primitive(PrimitiveType::Boolean),
			Kind::Char => Type::Primitive(PrimitiveType::Char),
			Kind::Float => Type::Primitive(PrimitiveType::Float),
			Kind::Double => Type::Primitive(PrimitiveType::Double),
			Kind::Byte => Type::Primitive(PrimitiveType::Byte),
			Kind::Short => Type::Primitive(PrimitiveType::Short),
			Kind::Int => Type::Primitive(PrimitiveType::Int),
			Kind::Long => Type::Primitive(PrimitiveType::Long),
		};

		Type::Array(ArrayType::from_component(component_ty))
	}
}

impl ToJava for ArrayRef {
	fn to_java(self, runtime: &Vm) -> eyre::Result<AnyValue> {
		self.reference.to_java(runtime)
	}
}

impl FromJava for ArrayRef {
	fn from_java(value: AnyValue, runtime: &Vm) -> eyre::Result<Self> {
		let reference = Reference::from_java(value, runtime)?;
		Ok(reference.to_array().ok().ok_or_else(|| CastTypeError {
			expected: ArrayType::ObjectArray().into(),
			found: value.ty(runtime),
		})?)
	}
}

impl JavaTyped for ArrayRef {
	fn java_type() -> Type {
		Reference::java_type()
	}
}

#[derive(Clone, Copy)]
pub struct Array<V> {
	array: ArrayRef,
	_p: PhantomData<V>,
}

impl<K: JavaKind> Array<K> {
	pub fn new(array: ArrayRef) -> Array<K> {
		Self::try_new(array).unwrap()
	}

	pub fn try_new(array: ArrayRef) -> Option<Array<K>> {
		if array.component_kind() == K::kind() {
			Some(Array {
				array,
				_p: Default::default(),
			})
		} else {
			None
		}
	}

	unsafe fn slot<V: Value>(&self, index: i32) -> Option<ValueCell<V>> {
		if index >= 0 && index < self.array.length() {
			unsafe {
				Some(ValueCell::new(
					(self.array.data_ptr() as *mut V).add(index as usize),
				))
			}
		} else {
			None
		}
	}

	pub fn length(&self) -> i32 {
		self.array.length()
	}
}

impl<B: Bindable + JavaKind> Array<B> {
	pub fn bind(&self, index: i32, vm: &Vm) -> Option<B::Cell> {
		let field = unsafe { self.slot::<B::Value>(index)? };
		Some(B::bind(vm, field))
	}
}

impl<V: Value> Array<V> {
	pub fn get(&self, index: i32) -> Option<V> {
		let field = unsafe { self.slot(index)? };
		Some(*field)
	}

	pub fn set(&mut self, index: i32, value: V) {
		let cell = unsafe { self.slot(index) };
		let Some(mut field) = cell else {
			return;
		};

		*field = value;
	}
}
//impl<V: Value> Value for Array<V> {
// 	fn kind() -> Kind {
// 		Kind::Reference
// 	}
//
// 	unsafe fn write(ptr: *mut UnionValue, value: Self) {
// 		Reference::write(ptr, value.reference)
// 	}
//
// 	unsafe fn read(ptr: UnionValue) -> Self {
// 		let reference = Reference::read(ptr);
// 		Array::new(ArrayRef::new(reference))
// 	}
// }
impl<V> ToJava for Array<V> {
	fn to_java(self, runtime: &Vm) -> eyre::Result<AnyValue> {
		self.array.to_java(runtime)
	}
}

impl<V: JavaKind> FromJava for Array<V> {
	fn from_java(value: AnyValue, vm: &Vm) -> eyre::Result<Self> {
		let any_array = ArrayRef::from_java(value, vm)?;
		Ok(Array::try_new(any_array).ok_or_else(|| CastTypeError {
			expected: ArrayType::ObjectArray().into(),
			found: value.ty(vm),
		})?)
	}
}

impl<V: Bindable + JavaTyped> JavaTyped for Array<V> {
	fn java_type() -> Type {
		ArrayType::from_component(V::java_type()).into()
	}
}

impl<V: JavaKind> Castable for Array<V> {
	fn cast_from(vm: &Vm, value: AnyValue) -> Self {
		let reference = Reference::cast_from(vm, value);
		Array::new(reference.to_array().unwrap())
	}
}

impl<V> Deref for Array<V> {
	type Target = ArrayRef;

	fn deref(&self) -> &Self::Target {
		&self.array
	}
}

impl<V: Typed> Typed for Array<V> {
	fn ty() -> Type {
		Type::Array(ArrayType::from_component(<V as Typed>::ty()))
	}
}

impl TryFrom<AnyValue> for ArrayRef {
	type Error = AnyValue;

	fn try_from(value: AnyValue) -> Result<Self, Self::Error> {
		let reference: Reference = AnyValue::try_into(value)?;
		Ok(ArrayRef::try_new(reference).ok_or(value)?)
	}
}

impl Into<AnyValue> for ArrayRef {
	fn into(self) -> AnyValue {
		AnyValue::Reference(self.reference)
	}
}

impl<V: JavaKind> TryFrom<AnyValue> for Array<V> {
	type Error = AnyValue;

	fn try_from(value: AnyValue) -> Result<Self, Self::Error> {
		let array: ArrayRef = AnyValue::try_into(value)?;
		Array::try_new(array).ok_or(value)
	}
}

impl<V: JavaKind> Into<AnyValue> for Array<V> {
	fn into(self) -> AnyValue {
		AnyValue::Reference(self.array.reference)
	}
}
