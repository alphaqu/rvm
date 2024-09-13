use crate::object::bindable::Bindable;
use crate::{
	AnyInstance, AnyValue, Array, ArrayRef, Field, FieldLayout, Instance, InstanceBinding,
	InstanceRef, JavaKind, Reference, UnionValue, Value, Vm,
};
use rvm_core::{ArrayType, Id, Kind, Type};
use rvm_reader::Op;
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
	pub fn by_id_typed<V: Value>(&self, id: Id<Field>) -> ValueCell<V> {
		self.by_id(id).typed::<V>()
	}

	pub fn by_id_binded<V: Bindable>(&self, vm: &Vm, id: Id<Field>) -> V::Cell {
		self.by_id(id).bind::<V>(vm)
	}

	pub fn by_name(&self, name: &str) -> Option<DynField2> {
		let field = self.layout.get_id(name)?;
		Some(self.by_id(field))
	}

	pub fn by_name_typed<V: Value>(&self, name: &str) -> Option<ValueCell<V>> {
		Some(self.by_name(name)?.typed::<V>())
	}
	pub fn by_name_binded<V: Bindable>(&self, vm: &Vm, name: &str) -> Option<V::Cell> {
		Some(self.by_name(name)?.bind::<V>(vm))
	}
}

pub struct DynField2 {
	ptr: *mut UnionValue,
	kind: Kind,
}

impl DynField2 {
	pub fn kind(&self) -> Kind {
		self.kind
	}
	pub fn get(&self) -> AnyValue {
		unsafe { AnyValue::read(self.ptr.read(), self.kind) }
	}

	pub fn set(&self, value: AnyValue) {
		if self.kind != value.kind() {
			panic!("Tried to set {} field with {:?} value.", self.kind, value);
		}

		unsafe {
			value.write(self.ptr);
		}
	}

	pub fn typed<V: Value>(self) -> ValueCell<V> {
		let ptr = unsafe { V::cast_pointer(self.ptr) };
		ValueCell { ptr }
	}

	pub fn bind<V: Bindable>(self, vm: &Vm) -> V::Cell {
		if self.kind != V::Value::kind() {
			panic!("Invalid type");
		}

		let field = self.typed::<V::Value>();
		V::bind(vm, field)
	}
}

macro_rules! impl_bindable {
	($V:ident) => {
		impl Bindable for $V {
			type Cell = ValueCell<$V>;
			type Value = $V;

			fn ty() -> Type {
				$V::kind().weak_ty()
			}

			fn bind(_: &Vm, value: ValueCell<Self::Value>) -> Self::Cell {
				value
			}
		}
	};
}

impl_bindable!(i8);
impl_bindable!(i16);
impl_bindable!(i32);
impl_bindable!(i64);
impl_bindable!(u16);
impl_bindable!(f32);
impl_bindable!(f64);
impl_bindable!(bool);
impl_bindable!(Reference);
//impl<V: Value> Bindable for V {
// 	type Cell = ValueCell<V>;
// 	type Value = V;
//
// 	fn ty() -> Type {
// 		V::kind().weak_ty()
// 	}
//
// 	fn bind(_: &Vm, value: ValueCell<Self::Value>) -> Self::Cell {
// 		value
// 	}
// }

impl<B: Bindable + JavaKind> Bindable for Array<B> {
	type Cell = ArrayCell<B>;
	type Value = Reference;

	fn ty() -> Type {
		Type::Array(ArrayType::from_component(B::ty()))
	}

	fn bind(_: &Vm, reference: ValueCell<Reference>) -> Self::Cell {
		ArrayCell::new(reference)
	}
}

impl<B: Bindable> JavaKind for Array<B> {
	fn kind() -> Kind {
		Kind::Reference
	}
}

#[derive(Clone)]
pub struct ArrayCell<B: Bindable> {
	ptr: ValueCell<Reference>,
	value: Option<Array<B>>,
}

impl<B: Bindable + JavaKind> ArrayCell<B> {
	pub fn new(field: ValueCell<Reference>) -> ArrayCell<B> {
		let value = if field.is_null() {
			None
		} else {
			Some(Array::new(ArrayRef::new(*field)))
		};
		ArrayCell { value, ptr: field }
	}

	pub fn set(&mut self, value: Array<B>) {
		*self.ptr = **value;
		self.sync_field();
	}

	pub fn sync_field(&mut self) {
		*self = ArrayCell::new(self.ptr);
	}
}

impl<B: Bindable> Deref for ArrayCell<B> {
	type Target = Array<B>;

	fn deref(&self) -> &Self::Target {
		self.value.as_ref().expect("null")
	}
}

impl<B: Bindable> DerefMut for ArrayCell<B> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.value.as_mut().expect("null")
	}
}

#[derive(Clone)]
pub struct InstanceCell<B: InstanceBinding> {
	ptr: ValueCell<Reference>,
	vm: Vm,
	value: Option<Instance<B>>,
}

impl<B: InstanceBinding> InstanceCell<B> {
	pub fn new(vm: &Vm, field: ValueCell<Reference>) -> Self {
		let value = if field.is_null() {
			None
		} else {
			Some(Instance::try_new(AnyInstance::new(vm.clone(), InstanceRef::new(*field))).unwrap())
		};
		Self {
			value,
			vm: vm.clone(),
			ptr: field,
		}
	}

	pub fn set(&mut self, value: Instance<B>) {
		*self.ptr = *value.untyped().raw;
		self.sync_field();
	}

	pub fn sync_field(&mut self) {
		*self = InstanceCell::new(&self.vm, self.ptr);
	}
}

impl<B: InstanceBinding> Deref for InstanceCell<B> {
	type Target = Instance<B>;

	fn deref(&self) -> &Self::Target {
		self.value.as_ref().expect("null")
	}
}

impl<B: InstanceBinding> DerefMut for InstanceCell<B> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.value.as_mut().expect("null")
	}
}

pub struct ValueCell<V: Value> {
	pub(super) ptr: *mut V,
}

impl<V: Value> ValueCell<V> {
	pub unsafe fn new(ptr: *mut V) -> Self {
		Self { ptr: ptr }
	}
}

impl<V: Value> Clone for ValueCell<V> {
	fn clone(&self) -> Self {
		*self
	}
}
impl<V: Value> Copy for ValueCell<V> {}
impl<V: Value> Deref for ValueCell<V> {
	type Target = V;

	fn deref(&self) -> &Self::Target {
		unsafe { &*(self.ptr as *const V) }
	}
}

impl<V: Value> DerefMut for ValueCell<V> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		unsafe { &mut *self.ptr }
	}
}
