mod source;

use ahash::{HashMap, HashMapExt};
use eyre::{bail, Context, ContextCompat};
use parking_lot::{Mutex, RwLock};
use std::io::{Cursor, Read};
use std::ops::Deref;
use std::sync::Arc;
use tracing::{debug, info, instrument, warn};

use rvm_core::{Id, Kind, ObjectType, Storage, Type};
use rvm_reader::ClassInfo;

use crate::object::class::Class;
use crate::{ArrayClass, CallType, InstanceClass, MethodIdentifier, Runtime, Vm};

pub use source::*;

pub struct ClassLoader {
	sources: Mutex<Vec<Box<dyn ClassSource>>>,
	classes: RwLock<Storage<Type, Class, Option<Arc<Class>>>>,
}

impl ClassLoader {
	pub fn new() -> ClassLoader {
		ClassLoader {
			sources: Mutex::new(Vec::new()),
			classes: RwLock::new(Storage::new()),
		}
	}

	pub fn add_source(&self, source: Box<dyn ClassSource>) {
		self.sources.lock().push(source);
	}

	pub fn get(&self, id: Id<Class>) -> Arc<Class> {
		if id == Id::null() {
			panic!("Null value");
		}
		let guard = self.classes.read();
		let value = guard.get(id).clone();
		if value.is_none() {
			for (id2, ty, _) in guard.iter_keys_unordered() {
				if id2 == id {
					panic!("Class {ty:?} is not available");
				}
			}
		}

		value.unwrap()
	}

	pub fn get_named(&self, ty: &Type) -> Option<Id<Class>> {
		self.classes.read().get_id(ty)
	}

	// Make this return Arc<Class>??
	//pub fn resolve(&self, runtime: &Runtime, desc: &Type) -> eyre::Result<Id<Class>> {
	//	// if its in the match the lock wont get dropped
	//	let option = self.classes.read().get_id(desc);
	//	match option {
	//		Some(value) => Ok(value),
	//		None => {
	//			let mut resolver = ClassResolver {
	//				cl: self,
	//				to_load: vec![],
	//			};
	//			let id = resolver.resolve(desc);
	//			resolver.link(runtime).wrap_err("Error linking class")?;
	//			Ok(id)
	//			//let id = self.allocate_id(desc.clone());
	//			//info!("resolving class {desc:?}");
	//			//let class = match desc {
	//			//	Type::Primitive(_) => {
	//			//		panic!("Tried to resolve primitive class.")
	//			//	}
	//			//	Type::Object(object) => {
	//			//		let class = self.load(object).unwrap();
	//			//		Class::Instance(class)
	//			//	}
	//			//	Type::Array(value) => {
	//			//		let mut component_id = None;
	//			//		if let Kind::Reference = value.component().kind() {
	//			//			// ensure loaded
	//			//			component_id = Some(self.resolve(value.component()));
	//			//		}
	//			//
	//			//		Class::Array(ArrayClass::new(
	//			//			id,
	//			//			(*value.component()).clone(),
	//			//			component_id,
	//			//		))
	//			//	}
	//			//};
	//			//
	//			//self.define(id, class);
	//			//id
	//		}
	//	}
	//}

	//fn load(&self, id: Id<Class>, ty: &ObjectType) -> eyre::Result<InstanceClass> {
	//	let guard = self.sources.lock();
	//	for source in guard.iter() {
	//		let Some(data) = source
	//			.try_load(ty)
	//			.wrap_err("Failed to load class from source")?
	//		else {
	//			continue;
	//		};
	//
	//		// Deadlock... oh shit, this is trademarked now
	//		drop(guard);
	//
	//		let info = ClassInfo::parse_complete(&data).wrap_err("Failed to parse .class file")?;
	//		let class = InstanceClass::new(id, info, self)?;
	//		return Ok(class);
	//	}
	//
	//	bail!("Failed to find a way to load {}", &**ty)
	//}

	///// Forcefully loads all classes in a jar. This is used only in bootstrapping the java standard library.
	//pub fn load_jar(
	//	&self,
	//	id: Id<Class>,
	//	data: &[u8],
	//	filter: impl Fn(&str) -> bool,
	//) -> eyre::Result<()> {
	//	let reader = Cursor::new(data);
	//	let mut archive = zip::read::ZipArchive::new(reader)?;
	//	let mut map: Vec<String> = archive.file_names().map(|v| v.to_string()).collect();
	//	map.sort();
	//	for name in map {
	//		let mut file = archive.by_name(&name)?;
	//		if file.is_file() && file.name().ends_with(".class") && filter(file.name()) {
	//			let mut data = Vec::with_capacity(file.size() as usize);
	//			file.read_to_end(&mut data)?;
	//			self.load_class(id, &data)
	//				.wrap_err_with(|| format!("Failed to load {}", file.name()))?;
	//		}
	//	}
	//	Ok(())
	//}

	///// Loads a java class to the JVM and injects it to the class table by locking it.
	//#[instrument(skip_all)]
	//pub fn load_class(&self, id: Id<Class>, data: &[u8]) -> eyre::Result<Id<Class>> {
	//	let info = ClassInfo::parse_complete(data).wrap_err("Failed to parse .class file")?;
	//	let class = InstanceClass::new(id, info, self)?;
	//
	//	debug!("Parsed class {}", class.ty);
	//
	//	let class = Class::Instance(class);
	//	let id = self.allocate_id(class.cloned_ty());
	//	self.define(id, class);
	//	Ok(id)
	//}

	fn allocate_id(&self, ty: Type) -> Id<Class> {
		self.classes.write().push(ty, None)
	}

	//fn define(&self, mut class: Class) {
	// 		let ty = class.cloned_ty();
	//
	// 		debug!("Inject and defining new class {ty:?}");
	// 		if self.classes.is_locked() {
	// 			warn!("Classes are locked");
	// 		}
	//
	// 		let mut guard = self.classes.write();
	// 		let class_slot = guard.get_mut(class.id());
	// 		*class_slot = Some(Arc::new(class));
	//
	// 		info!("Loaded class {ty:?} at {id:?}");
	// 	}

	fn define(&self, class: Class) {
		let id = class.id();
		let ty = class.cloned_ty();

		debug!("Inject and defining new class {ty:?}");
		if self.classes.is_locked() {
			warn!("Classes are locked");
		}

		let mut guard = self.classes.write();
		let class_slot = guard.get_mut(id);
		*class_slot = Some(Arc::new(class));

		info!("Loaded class {ty:?} at {id:?}");
	}
}
pub struct ClassResolver<'a> {
	cl: &'a ClassLoader,
	to_link: Vec<Id<Class>>,
}

impl<'a> ClassResolver<'a> {
	pub fn new(class_loader: &'a ClassLoader) -> Self {
		Self {
			cl: class_loader,
			to_link: vec![],
		}
	}

	fn scope<O>(
		cl: &ClassLoader,
		id: Id<Class>,
		func: impl FnOnce(&mut Class) -> eyre::Result<O>,
	) -> eyre::Result<O> {
		let mut guard = cl.classes.write();

		let slot = guard.get_mut(id);
		let class = slot.take().unwrap();
		let mut class = Arc::try_unwrap(class)
			.ok()
			.wrap_err("Class has arc references")?;

		drop(guard);

		let output = func(&mut class)?;

		let mut guard = cl.classes.write();
		let slot = guard.get_mut(id);

		let class = Arc::new(class);
		*slot = Some(class.clone());

		Ok(output)
	}

	pub fn link_all(mut self, ctx: &mut Runtime) -> eyre::Result<()> {
		if self.to_link.is_empty() {
			return Ok(());
		}
		info!("Linking all");

		let class_init = MethodIdentifier {
			name: "<clinit>".into(),
			descriptor: "()V".into(),
		};
		// Linking
		let mut to_initialize = Vec::new();
		for id in self.to_link.drain(..) {
			Self::scope(self.cl, id, |class| {
				let ty = class.cloned_ty();

				info!("Linking class {ty}");
				if let Class::Instance(class) = class {
					class.link(ctx).wrap_err_with(|| format!("Linking {ty}"))?;

					if class.methods.contains(&class_init) {
						to_initialize.push(id);
					}
				}

				Ok(())
			})?;
		}

		// Initializing
		for class in to_initialize {
			let class = self.cl.classes.read().get(class).as_ref().unwrap().clone();
			let class = class.as_instance().unwrap();
			info!("Initializing class {}", class.ty);

			ctx.run(CallType::Static, &class.ty, &class_init, vec![])
				.wrap_err("<clinit>")?;
		}
		Ok(())
	}

	pub fn resolve(&mut self, desc: &Type) -> eyre::Result<Id<Class>> {
		// if its in the match the lock wont get dropped
		let option = self.cl.classes.read().get_id(desc);
		match option {
			Some(value) => Ok(value),
			None => {
				let id = self.cl.allocate_id(desc.clone());
				info!("Resolving class {desc:?}");
				let class = match desc {
					Type::Primitive(_) => {
						panic!("Tried to resolve primitive class.")
					}
					Type::Object(object) => {
						let class = self
							.load_instance(id, object)
							.wrap_err_with(|| format!("Resolving instance {object}"))?;
						Class::Instance(class)
					}
					Type::Array(value) => {
						let mut component_id = None;
						if let Kind::Reference = value.component().kind() {
							// ensure loaded
							component_id = Some(
								self.resolve(value.component())
									.wrap_err("Resolving component type")?,
							);
						}

						Class::Array(ArrayClass::new(
							id,
							(*value.component()).clone(),
							component_id,
						))
					}
				};

				self.to_link.push(class.id());
				self.cl.define(class);
				Ok(id)
			}
		}
	}

	fn load_instance(&mut self, id: Id<Class>, ty: &ObjectType) -> eyre::Result<InstanceClass> {
		let guard = self.cl.sources.lock();
		for source in guard.iter() {
			let Some(data) = source
				.try_load(ty)
				.wrap_err("Failed to load class from source")?
			else {
				continue;
			};

			// Deadlock... oh shit, this is trademarked now
			drop(guard);

			let info = ClassInfo::parse_complete(&data).wrap_err("Failed to parse .class file")?;
			let class = InstanceClass::new(id, info, self)?;
			return Ok(class);
		}

		bail!("Failed to find a way to load {}", &**ty)
	}
}

impl<'a> Drop for ClassResolver<'a> {
	fn drop(&mut self) {
		assert!(self.to_link.is_empty());
	}
}

impl<'a> Deref for ClassResolver<'a> {
	type Target = ClassLoader;

	fn deref(&self) -> &Self::Target {
		self.cl
	}
}
