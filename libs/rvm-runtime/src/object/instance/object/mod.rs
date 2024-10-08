mod binding;
mod field;

use rvm_core::{CastTypeError, Id, Kind, ObjectType, StorageValue, Type};
use std::mem::size_of;
use std::ops::Deref;
use std::sync::Arc;

pub use binding::{Instance, InstanceBinding};
pub use field::*;

use crate::conversion::{FromJava, JavaTyped, ToJava};
use crate::gc::{InstanceHeader, JavaHeader};
use crate::{
	read_arr, write_arr, AnyValue, Castable, Class, Field, InstanceClass, Reference, ReferenceKind,
	UnionValue, Value, Vm,
};

#[derive(Copy, Clone)]
pub struct InstanceRef {
	// 1: class (u32)
	// 2: ref_fields (u16)
	reference: Reference,
}

impl Deref for InstanceRef {
	type Target = Reference;

	fn deref(&self) -> &Self::Target {
		&self.reference
	}
}

impl InstanceRef {
	pub const CLASS_ID_SIZE: usize = size_of::<<Class as StorageValue>::Idx>();
	pub const REF_FIELD_HEADER_SIZE: usize = size_of::<u16>();
	pub const FULL_HEADER_SIZE: usize =
		Reference::HEADER_SIZE + Self::CLASS_ID_SIZE + Self::REF_FIELD_HEADER_SIZE;

	pub fn new(reference: Reference) -> InstanceRef {
		reference.to_instance().unwrap()
	}

	pub fn try_new(reference: Reference) -> Option<InstanceRef> {
		if reference.reference_kind() != Some(ReferenceKind::Instance) {
			return None;
		}

		Some(unsafe { Self::new_unchecked(reference) })
	}

	pub unsafe fn new_unchecked(reference: Reference) -> InstanceRef {
		InstanceRef { reference }
	}

	pub fn header(&self) -> &InstanceHeader {
		let JavaHeader::Instance(header) = self.reference.header().user() else {
			panic!("Wrong header type");
		};
		header
	}

	pub fn class(&self) -> Id<Class> {
		self.header().id
	}

	pub fn ref_fields(&self) -> u16 {
		self.header().ref_fields
	}

	#[inline(always)]
	pub unsafe fn fields(&self) -> *mut UnionValue {
		self.reference.data_ptr() as *mut UnionValue
	}

	pub fn visit_refs(&self, mut visitor: impl FnMut(Reference)) {
		unsafe {
			let fields = self.fields();
			for i in 0..self.ref_fields() {
				let field = fields.add(i as usize);
				let value = field.read();
				visitor(value.reference);
			}
		}
	}

	pub fn map_refs(&self, mut mapper: impl FnMut(Reference) -> Reference) {
		unsafe {
			let fields = self.fields();
			for i in 0..self.ref_fields() {
				let field = fields.add(i as usize);
				let value = field.read();

				field.write(UnionValue {
					reference: mapper(value.reference),
				});
			}
		}
	}

	pub(crate) unsafe fn get_mut_ptr<V: Value>(&self, offset: usize) -> *mut V {
		let data = self.fields().add(offset);
		V::cast_pointer(data)
	}

	pub(super) unsafe fn get_any(&self, offset: usize, kind: Kind) -> AnyValue {
		let data = self.fields().add(offset).read();
		AnyValue::read(data, kind)
	}

	pub(super) unsafe fn get<V: Value>(&self, offset: usize) -> V {
		let data = self.fields().add(offset).read();
		V::read(data)
	}

	pub(super) unsafe fn put_any(&self, offset: usize, value: AnyValue) {
		let data = self.fields().add(offset);
		AnyValue::write(value, data)
	}

	pub(super) unsafe fn put<V: Value>(&self, offset: usize, value: V) {
		let data = self.fields().add(offset);
		V::write(data, value)
	}

	pub fn resolve(self, runtime: Vm) -> AnyInstance {
		AnyInstance::new(runtime, self)
	}

	pub fn ty(&self, runtime: &Vm) -> Type {
		self.resolve(runtime.clone()).ty(runtime)
	}
}

impl ToJava for InstanceRef {
	fn to_java(self, runtime: &Vm) -> eyre::Result<AnyValue> {
		self.reference.to_java(runtime)
	}
}

impl FromJava for InstanceRef {
	fn from_java(value: AnyValue, runtime: &Vm) -> eyre::Result<Self> {
		let reference = Reference::from_java(value, runtime)?;
		Ok(reference.to_instance().ok().ok_or_else(|| CastTypeError {
			expected: ObjectType::Object().into(),
			found: value.ty(runtime),
		})?)
	}
}

impl JavaTyped for InstanceRef {
	//fn java_type(&self, runtime: &Runtime) -> Type {
	//	self.resolve(runtime.clone()).java_type(runtime)
	//}

	fn java_type() -> Type {
		Reference::java_type()
	}
}
impl From<InstanceRef> for AnyValue {
	fn from(value: InstanceRef) -> Self {
		AnyValue::Reference(value.reference)
	}
}
impl From<AnyInstance> for AnyValue {
	fn from(value: AnyInstance) -> Self {
		AnyValue::from(value.raw)
	}
}

#[derive(Clone)]
pub struct AnyInstance {
	vm: Vm,
	is_static: bool,
	class: Arc<Class>,
	pub(crate) raw: InstanceRef,
}

impl AnyInstance {
	pub fn new(runtime: Vm, instance: InstanceRef) -> AnyInstance {
		Self::try_new(runtime, instance).unwrap()
	}

	pub fn try_new(runtime: Vm, instance: InstanceRef) -> Option<AnyInstance> {
		let arc = runtime.classes.get(instance.class());
		if !arc.is_instance() {
			return None;
		}

		Some(AnyInstance {
			vm: runtime,
			is_static: matches!(
				instance.reference.header().user(),
				JavaHeader::InstanceStatic(_)
			),
			class: arc,
			raw: instance,
		})
	}

	pub fn instance_of(&self, id: Id<Class>) -> bool {
		let mut class = self.class.clone();

		loop {
			let instance_class = class.as_instance().unwrap();
			if instance_class.id == id {
				return true;
			}

			for interface in &instance_class.interfaces {
				if interface.id == id {
					return true;
				}
			}

			if let Some(super_class) = &instance_class.super_class {
				class = self.vm.classes.get(super_class.id);
			} else {
				return false;
			}
		}
	}

	pub fn raw(&self) -> InstanceRef {
		self.raw
	}
	pub fn vm(&self) -> &Vm {
		&self.vm
	}

	pub fn class(&self) -> &InstanceClass {
		self.class.as_instance().unwrap()
	}

	pub fn class_id(&self) -> Id<Class> {
		self.class().id
	}

	pub fn fields(&self) -> FieldTable<'_> {
		let fields = &self.class.as_instance().unwrap().field_layout;
		unsafe { FieldTable::new(fields, self.raw.data_ptr()) }
	}

	pub fn typed<B: InstanceBinding>(self) -> Instance<B> {
		if self.is_static {
			todo!("Implement this!");
		}
		Instance::try_new(self).expect("Wrong type!")
	}

	pub fn ty(&self, _: &Vm) -> Type {
		self.class.cloned_ty()
	}
}
impl Deref for AnyInstance {
	type Target = InstanceRef;

	fn deref(&self) -> &Self::Target {
		&self.raw
	}
}
impl ToJava for AnyInstance {
	fn to_java(self, runtime: &Vm) -> eyre::Result<AnyValue> {
		self.raw.to_java(runtime)
	}
}

impl FromJava for AnyInstance {
	fn from_java(value: AnyValue, runtime: &Vm) -> eyre::Result<Self> {
		let instance = InstanceRef::from_java(value, runtime)?;
		Ok(instance.resolve(runtime.clone()))
	}
}

impl JavaTyped for AnyInstance {
	fn java_type() -> Type {
		Reference::java_type()
	}
}

impl Castable for InstanceRef {
	fn cast_from(runtime: &Vm, value: AnyValue) -> Self {
		let reference = Reference::cast_from(runtime, value);
		InstanceRef::new(reference)
	}
}
impl Castable for AnyInstance {
	fn cast_from(runtime: &Vm, value: AnyValue) -> Self {
		let reference = InstanceRef::cast_from(runtime, value);
		AnyInstance::new(runtime.clone(), reference)
	}
}
