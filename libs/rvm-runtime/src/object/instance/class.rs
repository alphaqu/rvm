use crate::gc::GarbageCollector;
use crate::object::{Class, ClassLoader};
use crate::{ClassFields, ClassMethods, FieldData, Reference};
use eyre::Context;
use rvm_core::{Id, ObjectType, Type};
use rvm_reader::{ClassInfo, ConstantPool};
use std::sync::Arc;

#[non_exhaustive]
pub struct ResolvedClassId {
	pub ty: ObjectType,
	pub id: Id<Class>,
}

impl ResolvedClassId {
	pub fn new(ty: ObjectType) -> ResolvedClassId {
		ResolvedClassId { ty, id: Id::null() }
	}
}

pub struct InstanceClass {
	pub id: Id<Class>,
	pub ty: ObjectType,

	pub super_class: Option<ResolvedClassId>,
	pub interfaces: Vec<ResolvedClassId>,

	pub cp: Arc<ConstantPool>,
	pub fields: ClassFields,
	pub static_fields: ClassFields,

	pub methods: ClassMethods,
}

unsafe impl Send for InstanceClass {}

unsafe impl Sync for InstanceClass {}

impl InstanceClass {
	pub fn new(info: ClassInfo, cl: &ClassLoader) -> eyre::Result<InstanceClass> {
		let super_class = info
			.constant_pool
			.get(info.super_class)
			.and_then(|v| info.constant_pool.get(v.name))
			.map(|v| ObjectType::new(v.to_string()));
		let super_id = super_class
			.as_ref()
			.map(|v| cl.resolve(&Type::Object(v.clone())));

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

		let interfaces: Vec<ResolvedClassId> = info
			.interfaces
			.iter()
			.map(|v| {
				let class = v.get(&info.constant_pool).unwrap();
				let class_name = class.name.get(&info.constant_pool).unwrap();

				let object_type = ObjectType::new(class_name.to_string());
				let id = cl.resolve(&object_type.clone().into());
				ResolvedClassId {
					ty: object_type,
					id,
				}
			})
			.collect();

		Ok(InstanceClass {
			id: Id::null(),
			ty: ObjectType::new(name.to_string()),
			super_class: super_class.map(|v| ResolvedClassId {
				ty: v,
				id: super_id.unwrap(),
			}),
			interfaces,
			methods: ClassMethods::parse(info.methods, &info.constant_pool)
				.wrap_err_with(|| format!("in CLASS \"{}\"", name.as_str()))?,
			//static_object: unsafe { ObjectData::new(fields.size(true) as usize) },
			fields: ClassFields::new(&fields, super_fields, false),
			static_fields: ClassFields::new(&fields, None, true),
			cp: Arc::new(info.constant_pool),
		})
	}
}
