use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;
use crate::arena::Arena;
use crate::Runtime;
use mmtk::util::{Address, ObjectReference};
use parking_lot::MappedRwLockReadGuard;
use rvm_core::{FieldAccessFlags, Id, Storage, StorageValue};
use rvm_object::{Class, ClassKind, ClassLoader, DynValue, Field, ObjectClass, Value};

pub struct Object {
	pub reference: ObjectReference,
	class: Arc<Class>,
}

impl Object {
	pub fn new<S: Into<String>>(runtime: &Runtime, class: Id<Class>, fields: impl IntoIterator<Item = (S, DynValue)>) -> Object {
		let reference = runtime.arena.alloc(class, &runtime.cl);
		let object = Object::wrap(reference, &runtime.cl);
		for (string, value) in fields.into_iter() {
			let string: String = string.into();
			let value: DynValue = value;
			object.set_dyn_field(&string, value);
		}
		object
    }

	pub fn wrap(object: ObjectReference, class_loader: &ClassLoader) -> Object {
		let id = unsafe {
			let id: u32 = object.to_header::<Arena>().load();
			class_loader.get(Id::new(id as usize))
		};

		Object {
			reference: object,
			class: id,
		}
	}


	fn class(&self) -> &ObjectClass {
		match &self.class.kind {
			ClassKind::Object(obj) => obj,
			_ => {
				panic!("Invalid type")
			}
		}
	}

	pub fn set_dyn_field(&self, field: impl Selector<String, Field>, value: DynValue) {
		let field = field.get(&self.class().fields);
		unsafe {
			if field.ty.kind() != value.ty() {
				panic!("Field mismatch")
			}
			if field.flags.contains(FieldAccessFlags::STATIC) {
				panic!("Field is static");
			}
			let field_ptr = self.ptr().add(field.offset as usize);
			value.write(field_ptr);
		}
	}

	pub fn set_field<V: Value>(&self, field: impl Selector<String, Field>, value: V) {
		let field = field.get(&self.class().fields);
		unsafe {
			if field.ty.kind() != V::ty() {
				panic!("Field mismatch")
			}
			if field.flags.contains(FieldAccessFlags::STATIC) {
				panic!("Field is static");
			}
			let field_ptr = self.ptr().add(field.offset as usize);
			V::write(field_ptr, value);
		}
	}

	pub fn get_dyn_field(&self, field: impl Selector<String, Field>) -> DynValue {
		let field = field.get(&self.class().fields);
		unsafe {
			if field.flags.contains(FieldAccessFlags::STATIC) {
				panic!("Field is static");
			}
			let field_ptr = self.ptr().add(field.offset as usize);
			DynValue::read(field_ptr, field.ty.kind())
		}
	}

	pub fn get_field<V: Value>(&self, field: impl Selector<String, Field>) -> V {
        let field = field.get(&self.class().fields);
		unsafe {
			if field.ty.kind() != V::ty() {
				panic!("Field mismatch")
			}

			if field.flags.contains(FieldAccessFlags::STATIC) {
				panic!("Field is static");
			}
			let field_ptr = self.ptr().add(field.offset as usize);
			V::read(field_ptr)
		}
	}

	fn ptr(&self) -> *mut u8 {
		self.reference.to_address::<Arena>().to_mut_ptr()
	}
}

pub trait Selector<K: Hash + Eq + Debug, V: StorageValue>: Copy {
	fn get(self, storage: &Storage<K, V>) -> &V;
}

impl<'a, K: Hash + Eq + Debug, V: StorageValue> Selector<K, V> for &'a K {
    fn get(self, storage: &Storage<K, V>) -> &V {
        storage.get_keyed(self).unwrap()
    }
}

impl<'a,  V: StorageValue> Selector<String, V> for &'a str {
	fn get(self, storage: &Storage<String, V>) -> &V {
		storage.get_keyed(self).unwrap()
	}
}

impl<K: Hash + Eq + Debug, V: StorageValue> Selector<K, V> for Id<V> {
	fn get(self, storage: &Storage<K, V>) -> &V {
		storage.get(self)
	}
}
