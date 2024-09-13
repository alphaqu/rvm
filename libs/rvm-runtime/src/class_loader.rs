//use crate::{Class, ClassSource, InstanceClass};
// use eyre::{bail, Context};
// use parking_lot::{Mutex, RwLock};
// use rvm_core::{Id, ObjectType, Storage, Type};
// use rvm_reader::ClassInfo;
// use std::sync::Arc;
//
// // LOAD: a class is loaded from its jar
// // LINKING: load all of the child classes.
// // Second classes resolve their superclasses, fields and methods.
// pub struct Classes {
// 	sources: Mutex<Vec<Box<dyn ClassSource>>>,
// 	classes: RwLock<Storage<Type, Class, Option<Arc<Class>>>>,
// }
//
// impl Classes {
// 	pub fn new() -> Classes {
// 		Classes {
// 			sources: Default::default(),
// 			classes: RwLock::new(Storage::new()),
// 		}
// 	}
//
// 	pub fn resolve(&self, ty: &Type) -> eyre::Result<Arc<Class>> {
// 		let guard = self.classes.read();
// 		if let Some(id) = guard.get_id(ty) {
// 			return Ok(guard.get(id).clone().unwrap());
// 		}
// 		drop(guard);
//
// 		let id = self.load(ty)?;
// 		Ok(self.get(id))
// 	}
//
// 	pub fn get(&self, id: Id<Class>) -> Arc<Class> {
// 		self.classes
// 			.read()
// 			.get(id)
// 			.clone()
// 			.expect("Class has not been loaded")
// 	}
//
// 	fn load(&self, ty: &Type) -> eyre::Result<Id<Class>> {
// 		let id = self.classes.write().insert(ty.clone(), None);
//
// 		match ty {
// 			Type::Primitive(_) => todo!(),
// 			Type::Object(object) => {
// 				self.load_object(id, object)?;
// 			}
// 			Type::Array(_) => todo!(),
// 		}
//
// 		Ok(id)
// 	}
//
// 	fn load_object(&self, id: Id<Class>, ty: &ObjectType) -> eyre::Result<()> {
// 		let data = self.read(ty)?;
// 		let info = ClassInfo::parse_complete(&data).wrap_err("Failed to parse .class file")?;
//
// 		let class = InstanceClass::new(id, info, self)?;
// 		return Ok(());
// 	}
//
// 	fn read(&self, ty: &ObjectType) -> eyre::Result<Vec<u8>> {
// 		let guard = self.sources.lock();
// 		for source in guard.iter() {
// 			let Some(data) = source
// 				.try_load(ty)
// 				.wrap_err("Failed to load class from source")?
// 			else {
// 				continue;
// 			};
//
// 			return Ok(data);
// 		}
//
// 		bail!("Failed to find a way to load {}", &**ty)
// 	}
// }
//
// pub struct RuntimeClasses {}
