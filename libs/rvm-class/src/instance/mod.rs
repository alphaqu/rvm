mod field;
mod method;
mod superface;

pub use crate::instance::field::*;
pub use crate::instance::method::*;
pub use crate::instance::superface::*;
use crate::ClassResolver;
use eyre::Context;
use rvm_core::{Id, ObjectType, StorageValue, Type};
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

pub struct Class {
	pub id: Id<Class>,
	pub ty: ObjectType,
	/// The constant pool
	pub cp: Arc<ConstantPool>,

	pub fields: ClassFields,
	pub methods: ClassMethods,
	pub superface: ClassSuperface,
}

unsafe impl Send for Class {}

unsafe impl Sync for Class {}

impl Class {
	pub fn new(id: Id<Class>, info: ClassInfo) -> eyre::Result<Class> {
		let class = &info.cp[info.this_class];
		let name = &info.cp[class.name];

		let superface = ClassSuperface::parse(&info).wrap_err("Superfaces")?;
		let fields = ClassFields::parse(&info.fields, &info.cp).wrap_err("Fields")?;
		let methods = ClassMethods::parse(info.methods, &info.cp).wrap_err("Methods")?;

		Ok(Class {
			id,
			ty: ObjectType::new(name.to_string()),
			cp: Arc::new(info.cp),
			methods,
			fields,
			superface,
		})
	}

	pub fn resolve(&mut self, func: &mut ClassResolver) -> eyre::Result<()> {
		self.superface.resolve(func)?;

		for method in self.methods.iter_mut() {
			if let Some(ty) = &method.desc.returns {
				Self::resolve_ty(ty, func)?;
			}

			for ty in &method.desc.parameters {
				Self::resolve_ty(ty, func)?;
			}
		}

		for field in self.fields.iter_mut() {
			Self::resolve_ty(&field.ty, func)?;
		}
		Ok(())
	}

	fn resolve_ty(ty: &Type, func: &mut ClassResolver) -> eyre::Result<()> {
		match ty {
			Type::Object(ty) => {
				func(ty)?;
			}
			Type::Array(ty) => {
				Self::resolve_ty(ty.component(), func)?;
			}
			Type::Primitive(_) => {}
		}
		Ok(())
	}
}

impl StorageValue for Class {
	type Idx = u32;
}
