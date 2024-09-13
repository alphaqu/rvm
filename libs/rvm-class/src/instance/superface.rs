use crate::{Class, ClassResolver};
use eyre::Context;
use rvm_core::{Id, ObjectType};
use rvm_reader::ClassInfo;

/// This holds the resolution to the superclass and the interfaces.
pub struct ClassSuperface {
	pub superclass: Option<Superface>,
	pub interfaces: Vec<Superface>,
}

impl ClassSuperface {
	// fn resolve_super(
	// 		info: &ClassInfo,
	// 		cl: &mut ClassResolver,
	// 	) -> eyre::Result<Option<ResolvedSuper>> {
	// 		let Some(super_class) = info.cp.get(info.super_class) else {
	// 			return Ok(None);
	// 		};
	//
	// 		let super_ty = ObjectType::new(info.cp[super_class.name].to_string());
	// 		let super_id = cl.resolve(&Type::Object(super_ty.clone()))?;
	//
	// 		Ok(Some(ResolvedSuper {
	// 			id: super_id,
	// 			class: cl.get(super_id),
	// 			ty: super_ty,
	// 		}))
	// 	}
	//
	// 	fn resolve_interfaces(
	// 		info: &ClassInfo,
	// 		cl: &mut ClassResolver,
	// 	) -> eyre::Result<Vec<ResolvedClassId>> {
	// 		info.interfaces
	// 			.iter()
	// 			.map(|interface| {
	// 				let class = &info.cp[*interface];
	// 				let class_name = &info.cp[class.name];
	//
	// 				let object_type = ObjectType::new(class_name.to_string());
	// 				let id = cl
	// 					.resolve(&object_type.clone().into())
	// 					.wrap_err_with(|| format!("Failed to resolve interface {object_type}"))?;
	//
	// 				Ok(ResolvedClassId {
	// 					ty: object_type,
	// 					id,
	// 				})
	// 			})
	// 			.try_collect()
	// 	}
	pub fn parse(info: &ClassInfo) -> eyre::Result<Self> {
		let cp = &info.cp;
		let super_class = info.super_class.ty(cp);
		Ok(ClassSuperface {
			superclass: super_class.map(Superface::new),
			interfaces: info
				.interfaces
				.iter()
				.map(|v| {
					let interface_type = v.ty(cp).unwrap();
					Superface::new(interface_type)
				})
				.collect(),
		})
	}

	pub fn resolve(&mut self, func: &mut ClassResolver) -> eyre::Result<()> {
		if let Some(superclass) = &mut self.superclass {
			superclass
				.resolve(func)
				.wrap_err_with(|| format!("Resolving superclass {:?}", superclass.ty))?;
		}

		for interface in self.interfaces.iter_mut() {
			interface
				.resolve(func)
				.wrap_err_with(|| format!("Resolving interface {:?}", interface.ty))?;
		}

		Ok(())
	}
}

pub struct Superface {
	pub ty: ObjectType,
	pub resolved_id: Option<Id<Class>>,
}

impl Superface {
	pub fn new(ty: ObjectType) -> Superface {
		Superface {
			ty,
			resolved_id: None,
		}
	}

	pub fn resolve(&mut self, func: &mut ClassResolver) -> eyre::Result<()> {
		self.resolved_id = Some(func(&self.ty)?);
		Ok(())
	}

	pub fn id(&self) -> Id<Class> {
		self.resolved_id
			.expect("Class superface has not been resolved")
	}
}
