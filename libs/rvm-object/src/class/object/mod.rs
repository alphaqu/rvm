use std::ops::Deref;
use std::sync::Arc;

use anyways::ext::AuditExt;
use anyways::Result;

pub use field::*;
pub use method::*;
use rvm_core::ObjectType;
use rvm_reader::{ClassInfo, ConstantPool};

use crate::Class;

mod field;
mod method;

pub struct ObjectClass {
	pub ty: ObjectType,
	pub cp: Arc<ConstantPool>,
	pub fields: ObjectFieldLayout,
	pub static_fields: ObjectFieldLayout,
	pub methods: ClassMethodManager,
	//pub static_object: Reference,
}

unsafe impl Send for ObjectClass {}
unsafe impl Sync for ObjectClass {}
impl ObjectClass {
	pub fn parse(info: ClassInfo) -> Result<ObjectClass> {
		let class = info.constant_pool.get(info.this_class);
		let name = info.constant_pool.get(class.name);

		let fields: Vec<FieldData> = info
			.fields
			.iter()
			.map(|v| FieldData::from_info(v, &info.constant_pool).unwrap())
			.collect();

		Ok(ObjectClass {
			ty: ObjectType(name.to_string()),
			methods: ClassMethodManager::parse(info.methods, name.as_str(), &info.constant_pool)
				.wrap_err_with(|| format!("in CLASS \"{}\"", name.as_str()))?,
			//static_object: unsafe { ObjectData::new(fields.size(true) as usize) },
			fields: ObjectFieldLayout::new(&fields, false),
			static_fields: ObjectFieldLayout::new(&fields, true),
			cp: Arc::new(info.constant_pool),
		})
	}
}
