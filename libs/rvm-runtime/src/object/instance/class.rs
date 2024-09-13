use crate::object::Class;
use crate::{
	ClassLoader, ClassMethods, ClassResolver, FieldData, FieldLayout, FieldTable, InstanceRef,
	Runtime, Vm,
};
use eyre::{Context, ContextCompat};
use rvm_core::{Id, ObjectType, Type};
use rvm_reader::{ClassInfo, ConstantPool};
use std::sync::Arc;
use tracing::trace;

#[non_exhaustive]
#[derive(Clone)]
pub struct ResolvedClassId {
	pub ty: ObjectType,
	pub id: Id<Class>,
}

#[derive(Clone)]
pub struct InstanceClass {
	pub id: Id<Class>,
	pub ty: ObjectType,

	pub super_class: Option<ResolvedClassId>,
	pub interfaces: Vec<ResolvedClassId>,

	pub cp: Arc<ConstantPool>,

	pub field_layout: FieldLayout,
	pub static_field_layout: FieldLayout,

	pub methods: ClassMethods,

	companion: Option<ClassCompanion>,
}

unsafe impl Send for InstanceClass {}

unsafe impl Sync for InstanceClass {}

struct ResolvedSuper {
	id: Id<Class>,
	ty: ObjectType,
}

impl InstanceClass {
	fn resolve_super(
		info: &ClassInfo,
		cl: &mut ClassResolver,
	) -> eyre::Result<Option<ResolvedClassId>> {
		let Some(super_class) = info.cp.get(info.super_class) else {
			return Ok(None);
		};

		let super_ty = ObjectType::new(info.cp[super_class.name].to_string());
		let super_id = cl.resolve(&Type::Object(super_ty.clone()))?;

		Ok(Some(ResolvedClassId {
			id: super_id,
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
				let class = cl
					.resolve(&object_type.clone().into())
					.wrap_err_with(|| format!("Failed to resolve interface {object_type}"))?;

				Ok(ResolvedClassId {
					ty: object_type,
					id: class,
				})
			})
			.try_collect()
	}

	fn resolve_fields(info: &ClassInfo, _cl: &ClassLoader) -> eyre::Result<Vec<FieldData>> {
		info.fields
			.iter()
			.map(|v| {
				let field =
					FieldData::from_info(v, &info.cp).wrap_err("Failed to parse descriptor")?;
				trace!("field: {:?} {}", field.ty, field.name);

				Ok(field)
			})
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
		let super_instance = super_class.as_ref().map(|v| cl.get(v.id));

		let field_layout = FieldLayout::new_instance(
			&fields,
			super_instance
				.as_ref()
				.map(|v| &v.to_instance().field_layout),
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

	pub fn initialize(&self, ctx: &mut Runtime) -> eyre::Result<InstanceClass> {
		let class = ctx.std().c_class;
		let class = ctx.classes.get(class);
		let class = class.to_instance();

		let result = ctx.alloc_object(class)?;
		let companion = ClassCompanion {
			static_ref: ctx.alloc_static_instance(self)?,
			class: result.raw(),
		};

		Ok(InstanceClass {
			companion: Some(companion),
			..self.clone()
		})
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

#[derive(Clone)]
pub struct ClassCompanion {
	pub static_ref: InstanceRef,
	pub class: InstanceRef,
}

impl ClassCompanion {}
