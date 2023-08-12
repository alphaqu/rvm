use crate::value::Value;
use crate::value::{read_arr, write_arr};
use crate::{Class, ClassLoader, DynValue, Field, ObjectClass, ObjectFieldLayout};
use rvm_core::{Id, Reference, ReferenceKind, StorageValue};
use std::mem::size_of;
use std::ops::Deref;

pub enum Object {
	Class(AnyClassObject),
}

impl Object {
	pub const HEADER_SIZE: usize = size_of::<u8>();
	pub fn new(reference: Reference) -> Object {
		match reference.kind() {
			ReferenceKind::Class => Object::Class(AnyClassObject { reference }),
		}
	}

	pub fn as_class(&self) -> Option<&AnyClassObject> {
		match self {
			Object::Class(class) => Some(class),
		}
	}

	pub fn visit_refs(&self, mut visitor: impl FnMut(Reference)) {
		match self {
			Object::Class(raw) => raw.visit_refs(visitor),
		}
	}

	pub fn map_refs(&self, mut mapper: impl FnMut(Reference) -> Reference) {
		match self {
			Object::Class(raw) => raw.map_refs(mapper),
		}
	}
}

#[derive(Copy, Clone)]
pub struct AnyClassObject {
	// 1: class (u32)
	// 2: ref_fields (u16)
	reference: Reference,
}

impl Deref for AnyClassObject {
	type Target = Reference;

	fn deref(&self) -> &Self::Target {
		&self.reference
	}
}

impl AnyClassObject {
	pub const CLASS_ID_SIZE: usize = size_of::<<Class as StorageValue>::Idx>();
	pub const REF_FIELD_HEADER_SIZE: usize = size_of::<u16>();
	pub const FULL_HEADER_SIZE: usize =
		Object::HEADER_SIZE + Self::CLASS_ID_SIZE + Self::REF_FIELD_HEADER_SIZE;

	/// Allocates a new object
	pub unsafe fn allocate(
		reference: Reference,
		id: Id<Class>,
		class: &ObjectClass,
	) -> AnyClassObject {
		reference.0.write(1);
		let i = id.idx();
		println!("idx: {i}");
		let bytes = i.to_le_bytes();
		write_arr(reference.0.add(Object::HEADER_SIZE), bytes);
		write_arr(
			reference.0.add(Object::HEADER_SIZE + Self::CLASS_ID_SIZE),
			class.fields.ref_fields.to_le_bytes(),
		);
		println!("{:?}", reference.kind());

		let object = AnyClassObject { reference };
		println!("{:?}", object.class());
		object
	}

	pub fn class(&self) -> Id<Class> {
		unsafe {
			let ptr = self.reference.0.add(Object::HEADER_SIZE);
			let i = <Class as StorageValue>::Idx::from_le_bytes(read_arr(ptr));
			Id::new(i as usize)
		}
	}

	pub fn ref_fields(&self) -> u16 {
		unsafe {
			let ptr = self
				.reference
				.0
				.add(Object::HEADER_SIZE + Self::CLASS_ID_SIZE);
			u16::from_le_bytes(read_arr(ptr))
		}
	}

	pub unsafe fn fields(&self) -> *mut u8 {
		self.reference.0.add(Self::FULL_HEADER_SIZE)
	}

	pub fn visit_refs(&self, mut visitor: impl FnMut(Reference)) {
		unsafe {
			let fields = self.fields();
			for i in 0..self.ref_fields() {
				let ptr = fields.add(size_of::<Reference>() * i as usize);
				let reference = Reference::read(ptr);
				visitor(reference);
			}
		}
	}

	pub fn map_refs(&self, mut mapper: impl FnMut(Reference) -> Reference) {
		unsafe {
			let fields = self.fields();
			for i in 0..self.ref_fields() {
				let ptr = fields.add(size_of::<Reference>() * i as usize);
				let reference = Reference::read(ptr);
				Reference::write(ptr, mapper(reference));
			}
		}
	}

	pub fn resolve<'a>(&self, class: &'a ObjectClass) -> ResolvedClassObject<'a> {
		ResolvedClassObject {
			fields: &class.fields,
			raw: *self,
		}
	}
}

pub struct ResolvedClassObject<'a> {
	fields: &'a ObjectFieldLayout,
	raw: AnyClassObject,
}

impl<'a> ResolvedClassObject<'a> {
	pub fn get_dyn(&self, id: Id<Field>) -> DynValue {
		let field = self.fields.get(id);
		unsafe {
			let data = self.raw.fields().add(field.offset as usize);
			DynValue::read(data, field.ty.kind())
		}
	}

	pub fn put_dyn(&self, id: Id<Field>, value: DynValue) {
		let field = self.fields.get(id);
		unsafe {
			let data = self.raw.fields().add(field.offset as usize);
			DynValue::write(value, data)
		}
	}
}
