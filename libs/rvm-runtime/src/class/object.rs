use crate::class::field::ClassFieldManager;
use crate::class::method::ClassMethodManager;
use crate::class::ClassKind;
use crate::executor::StackValue;
use crate::object::ObjectData;
use crate::{Class, ClassInfo, ClassLoader, ConstantPool, Field, JResult, Ref, Runtime};
use anyways::ext::AuditExt;
use anyways::Result;
use parking_lot::MappedRwLockReadGuard;
use rvm_consts::FieldAccessFlags;
use rvm_core::Id;
use std::alloc::dealloc;
use std::ops::Deref;

pub struct ObjectClass {
	pub simple_name: String,

	pub cp: ConstantPool,
	pub fields: ClassFieldManager,
	pub methods: ClassMethodManager,
	pub static_object: ObjectData,
}

impl ObjectClass {
	pub fn parse(info: ClassInfo, class_loader: &ClassLoader) -> Result<Class> {
		let class = info.constant_pool.get(info.this_class);
		let name = info.constant_pool.get(class.name);

		let fields = ClassFieldManager::parse(info.fields, &info.constant_pool, class_loader);
		let binary_name = name.to_string().replace('/', ".");
		Ok(Class {
			kind: ClassKind::Object(ObjectClass {
				simple_name: {
					let value = binary_name.rsplit('.').next().unwrap();
					value.to_string()
				},
				methods: ClassMethodManager::parse(
					info.methods,
					name.as_str(),
					&info.constant_pool,
					class_loader,
				)
				.wrap_err_with(|| format!("in CLASS \"{}\"", name.as_str()))?,
				static_object: unsafe { ObjectData::new(fields.size(true) as usize) },
				cp: info.constant_pool,
				fields,
			}),
			binary_name,
		})
	}

	pub fn get_static(&self, field: Id<Field>) -> StackValue {
		let field = self.fields.get(field);
		unsafe {
			if !field.flags.contains(FieldAccessFlags::STATIC) {
				panic!("Field not static");
			}
			let field_ptr = self.static_object.ptr().add(field.offset as usize);
			StackValue::from_value(field.ty.read(field_ptr))
		}
	}

	pub fn set_static(&self, field: Id<Field>, value: StackValue) {
		let field = self.fields.get(field);
		unsafe {
			if !field.flags.contains(FieldAccessFlags::STATIC) {
				panic!("Field not static");
			}
			let value = field.ty.new_val(value);
			let field_ptr = self.static_object.ptr().add(field.offset as usize);
			field.ty.write(field_ptr, value);
		}
	}

	pub fn size(&self, static_obj: bool) -> usize {
		self.fields.size(static_obj) as usize
	}
}

impl<'a> Runtime<'a> {
	pub fn new_object(&self, class_id: Id<Class>) -> JResult<Object> {
		let class = self.cl.get_obj_class(class_id);

		unsafe {
			let reference = self.gc.write().unwrap().alloc(class_id, class.size(false));
			Ok(Object { reference, class })
		}
	}

	pub fn get_object(&self, class_id: Id<Class>, reference: Ref) -> JResult<Object> {
		reference.assert_matching_class(class_id, self)?;
		let class = self.cl.get_obj_class(class_id);
		Ok(Object { reference, class })
	}
}

pub struct Object<'a> {
	reference: Ref,
	pub class: MappedRwLockReadGuard<'a, ObjectClass>,
}

impl<'a> Object<'a> {
	pub fn set_field(&self, field: Id<Field>, value: StackValue) {
		let field = self.class.fields.get(field);
		unsafe {
			if field.flags.contains(FieldAccessFlags::STATIC) {
				panic!("Field is static");
			}
			let value = field.ty.new_val(value);
			let field_ptr = self.ptr().add(field.offset as usize);
			field.ty.write(field_ptr, value);
		}
	}

	pub fn get_field(&self, field: Id<Field>) -> StackValue {
		let field = self.class.fields.get(field);
		unsafe {
			if field.flags.contains(FieldAccessFlags::STATIC) {
				panic!("Field is static");
			}
			let field_ptr = self.ptr().add(field.offset as usize);
			StackValue::from_value(field.ty.read(field_ptr))
		}
	}
}

impl<'a> Deref for Object<'a> {
	type Target = Ref;

	fn deref(&self) -> &Self::Target {
		&self.reference
	}
}

impl Drop for ObjectClass {
	fn drop(&mut self) {
		unsafe {
			// drop static class fields
			dealloc(
				self.static_object.ptr(),
				ObjectData::layout(self.fields.size(true) as usize),
			)
		}
	}
}
