use crate::object::Class;
use crate::{
	ClassMethods, ClassResolver, FieldData, FieldLayout, FieldTable, InstanceRef, Runtime, Vm,
};
use eyre::{Context, ContextCompat};
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
	pub field_layout: FieldLayout,
	pub static_field_layout: FieldLayout,

	pub methods: ClassMethods,

	// Linking
	companion: Option<ClassCompanion>,
}

unsafe impl Send for InstanceClass {}

unsafe impl Sync for InstanceClass {}

struct ResolvedSuper {
	id: Id<Class>,
	class: Arc<Class>,
	ty: ObjectType,
}
impl InstanceClass {
	fn resolve_super(
		info: &ClassInfo,
		cl: &mut ClassResolver,
	) -> eyre::Result<Option<ResolvedSuper>> {
		let Some(super_class) = info.cp.get(info.super_class) else {
			return Ok(None);
		};

		let super_ty = ObjectType::new(info.cp[super_class.name].to_string());
		let super_id = cl.resolve(&Type::Object(super_ty.clone()))?;

		Ok(Some(ResolvedSuper {
			id: super_id,
			class: cl.get(super_id),
			ty: super_ty,
		}))
	}

	fn resolve_interfaces(
		info: &ClassInfo,
		cl: &mut ClassResolver,
	) -> eyre::Result<Vec<ResolvedClassId>> {
		info.interfaces
			.iter()
			.map(|interface| {
				let class = &info.cp[*interface];
				let class_name = &info.cp[class.name];

				let object_type = ObjectType::new(class_name.to_string());
				let id = cl
					.resolve(&object_type.clone().into())
					.wrap_err_with(|| format!("Failed to resolve interface {object_type}"))?;

				Ok(ResolvedClassId {
					ty: object_type,
					id,
				})
			})
			.try_collect()
	}

	fn resolve_fields(info: &ClassInfo, _cl: &mut ClassResolver) -> eyre::Result<Vec<FieldData>> {
		info.fields
			.iter()
			.map(|v| FieldData::from_info(v, &info.cp).wrap_err("Failed to parse descriptor"))
			.try_collect()
	}
	pub fn new(
		id: Id<Class>,
		info: ClassInfo,
		cl: &mut ClassResolver,
	) -> eyre::Result<InstanceClass> {
		let class = &info.cp[info.this_class];
		let name = &info.cp[class.name];

		let fields: Vec<FieldData> =
			Self::resolve_fields(&info, cl).wrap_err("Resolving fields")?;

		let super_class = Self::resolve_super(&info, cl).wrap_err("Resolving super-class")?;

		let interfaces: Vec<ResolvedClassId> =
			Self::resolve_interfaces(&info, cl).wrap_err("Resolving interfaces")?;

		// Create field layouts
		let field_layout = FieldLayout::new_instance(
			&fields,
			super_class
				.as_ref()
				.map(|v| &v.class.as_instance().unwrap().field_layout),
		);
		let static_field_layout = FieldLayout::new_static(&fields);

		// Class
		//cl.resolve(&Type::Object(ObjectType::new("java/lang/Class")))?;

		Ok(InstanceClass {
			id,
			ty: ObjectType::new(name.to_string()),
			super_class: super_class.map(|v| ResolvedClassId { ty: v.ty, id: v.id }),
			interfaces,
			methods: ClassMethods::parse(info.methods, &info.cp)
				.wrap_err_with(|| format!("in CLASS \"{}\"", name.as_str()))?,
			//static_object: unsafe { ObjectData::new(fields.size(true) as usize) },
			field_layout,
			static_field_layout,
			cp: Arc::new(info.cp),
			companion: None,
		})
	}

	pub fn link(&mut self, ctx: &mut Runtime) -> eyre::Result<()> {
		let class = ctx.std().c_class;
		let class = ctx.classes.get(class);
		let class = class.to_instance();

		let result = ctx.alloc_object(class)?;
		self.companion = Some(ClassCompanion {
			static_ref: ctx.alloc_static_instance(self)?,
			class: result.raw(),
		});
		Ok(())
	}

	pub fn companion(&self) -> &ClassCompanion {
		self.companion
			.as_ref()
			.expect("Class has never been linked")
	}

	pub fn static_fields(&self) -> FieldTable<'_> {
		let companion = self.companion();
		unsafe { FieldTable::new(&self.static_field_layout, companion.static_ref.data_ptr()) }
	}
}

pub struct ClassCompanion {
	pub static_ref: InstanceRef,
	pub class: InstanceRef,
}

impl ClassCompanion {}
