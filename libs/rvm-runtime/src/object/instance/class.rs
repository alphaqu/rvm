use crate::{ClassMethodManager, FieldData, ObjectFieldLayout};
use eyre::Context;
use rvm_core::{Id, ObjectType, Type};
use rvm_reader::{ClassInfo, ConstantPool};
use std::sync::Arc;

use crate::object::{Class, ClassLoader};

#[non_exhaustive]
pub struct ClassRef {
	pub ty: ObjectType,
	pub id: Id<Class>,
}

impl ClassRef {
	pub fn new(ty: ObjectType) -> ClassRef {
		ClassRef { ty, id: Id::null() }
	}
}
pub struct InstanceClass {
	pub id: Id<Class>,
	pub ty: ObjectType,

	pub super_class: Option<ClassRef>,

	pub cp: Arc<ConstantPool>,
	pub fields: ObjectFieldLayout,
	pub static_fields: ObjectFieldLayout,
	pub methods: ClassMethodManager,

	pub interfaces: Vec<ClassRef>,
}

unsafe impl Send for InstanceClass {}

unsafe impl Sync for InstanceClass {}

impl InstanceClass {
	pub fn parse(info: ClassInfo, cl: &ClassLoader) -> eyre::Result<InstanceClass> {
		let super_class = info
			.constant_pool
			.get(info.super_class)
			.and_then(|v| info.constant_pool.get(v.name))
			.map(|v| ObjectType(v.to_string()));
		let super_id = super_class
			.as_ref()
			.map(|v| cl.resolve_class(&Type::Object(v.clone())));

		let super_object = super_id.map(|super_id| cl.get(super_id));
		let super_fields = super_object
			.as_ref()
			.map(|v| &v.as_instance().as_ref().unwrap().fields);

		let class = info.constant_pool.get(info.this_class).unwrap();
		let name = info.constant_pool.get(class.name).unwrap();

		let fields: Vec<FieldData> = info
			.fields
			.iter()
			.map(|v| FieldData::from_info(v, &info.constant_pool).unwrap())
			.collect();

		let interfaces: Vec<ClassRef> = info
			.interfaces
			.iter()
			.map(|v| {
				let class = v.get(&info.constant_pool).unwrap();
				let class_name = class.name.get(&info.constant_pool).unwrap();

				let object_type = ObjectType(class_name.to_string());
				let id = cl.resolve_class(&object_type.clone().into());
				ClassRef {
					ty: object_type,
					id,
				}
			})
			.collect();

		Ok(InstanceClass {
			id: Id::null(),
			ty: ObjectType(name.to_string()),
			super_class: super_class.map(|v| ClassRef {
				ty: v,
				id: super_id.unwrap(),
			}),
			interfaces,
			methods: ClassMethodManager::parse(info.methods, &info.constant_pool)
				.wrap_err_with(|| format!("in CLASS \"{}\"", name.as_str()))?,
			//static_object: unsafe { ObjectData::new(fields.size(true) as usize) },
			fields: ObjectFieldLayout::new(&fields, super_fields, false),
			static_fields: ObjectFieldLayout::new(&fields, None, true),
			cp: Arc::new(info.constant_pool),
		})
	}
}
