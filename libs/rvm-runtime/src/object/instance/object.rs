use crate::{
	read_arr, write_arr, AnyValue, Class, Field, InstanceClass, ObjectFieldLayout, Reference,
	ReferenceKind, Value,
};
use rvm_core::{Id, StorageValue};
use std::mem::size_of;
use std::ops::Deref;
use std::println;

#[derive(Copy, Clone)]
pub struct AnyInstance {
	// 1: class (u32)
	// 2: ref_fields (u16)
	reference: Reference,
}

impl Deref for AnyInstance {
	type Target = Reference;

	fn deref(&self) -> &Self::Target {
		&self.reference
	}
}

impl AnyInstance {
	pub const CLASS_ID_SIZE: usize = size_of::<<Class as StorageValue>::Idx>();
	pub const REF_FIELD_HEADER_SIZE: usize = size_of::<u16>();
	pub const FULL_HEADER_SIZE: usize =
		Reference::HEADER_SIZE + Self::CLASS_ID_SIZE + Self::REF_FIELD_HEADER_SIZE;

	pub fn new(reference: Reference) -> AnyInstance {
		Self::try_new(reference).unwrap()
	}

	pub fn try_new(reference: Reference) -> Option<AnyInstance> {
		if reference.kind() != Some(ReferenceKind::Instance) {
			return None;
		}

		Some(unsafe { Self::new_unchecked(reference) })
	}

	pub unsafe fn new_unchecked(reference: Reference) -> AnyInstance {
		AnyInstance { reference }
	}

	/// Allocates a new instance object
	pub unsafe fn allocate(
		reference: Reference,
		id: Id<Class>,
		class: &InstanceClass,
	) -> AnyInstance {
		reference.0.write(1);
		let i = id.idx();
		println!("idx: {i}");
		let bytes = i.to_le_bytes();
		write_arr(reference.0.add(Reference::HEADER_SIZE), bytes);
		write_arr(
			reference
				.0
				.add(Reference::HEADER_SIZE + Self::CLASS_ID_SIZE),
			class.fields.ref_fields.to_le_bytes(),
		);
		println!("{:?}", reference.kind());

		let object = AnyInstance { reference };
		println!("{:?}", object.class());
		object
	}

	pub fn class(&self) -> Id<Class> {
		unsafe {
			let ptr = self.reference.0.add(Reference::HEADER_SIZE);
			let i = <Class as StorageValue>::Idx::from_le_bytes(read_arr(ptr));
			Id::new(i as usize)
		}
	}

	pub fn ref_fields(&self) -> u16 {
		unsafe {
			let ptr = self
				.reference
				.0
				.add(Reference::HEADER_SIZE + Self::CLASS_ID_SIZE);
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

	pub fn resolve<'a>(&self, class: &'a InstanceClass) -> Instance<'a> {
		Instance {
			fields: &class.fields,
			raw: *self,
		}
	}
}

pub struct Instance<'a> {
	fields: &'a ObjectFieldLayout,
	raw: AnyInstance,
}

impl<'a> Instance<'a> {
	pub fn get_dyn(&self, id: Id<Field>) -> AnyValue {
		let field = self.fields.get(id);
		unsafe {
			let data = self.raw.fields().add(field.offset as usize);
			AnyValue::read(data, field.ty.kind())
		}
	}

	pub fn put_dyn(&self, id: Id<Field>, value: AnyValue) {
		let field = self.fields.get(id);
		unsafe {
			let data = self.raw.fields().add(field.offset as usize);
			AnyValue::write(value, data)
		}
	}
}
