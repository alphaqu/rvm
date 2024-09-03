mod source;

use eyre::{bail, Context};
use parking_lot::{Mutex, RwLock};
use std::io::{Cursor, Read};
use std::sync::Arc;
use tracing::{debug, info, instrument, warn};

use rvm_core::{Id, Kind, ObjectType, Storage, Type};
use rvm_reader::ClassInfo;

use crate::object::class::Class;
use crate::{ArrayClass, InstanceClass};

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
		self.classes
			.read()
			.get(id)
			.clone()
			.expect("Class never loaded")
	}

	pub fn get_named(&self, ty: &Type) -> Option<Id<Class>> {
		self.classes.read().get_id(ty)
	}

	pub fn resolve(&self, desc: &Type) -> Id<Class> {
		// if its in the match the lock wont get dropped
		let option = self.classes.read().get_id(desc);
		match option {
			Some(value) => value,
			None => {
				let id = self.allocate_id(desc.clone());
				info!("resolving class {desc:?}");
				let class = match desc {
					Type::Primitive(_) => {
						panic!("Tried to resolve primitive class.")
					}
					Type::Object(object) => {
						let class = self.load(object).unwrap();
						Class::Object(class)
					}
					Type::Array(value) => {
						let mut component_id = None;
						if let Kind::Reference = value.component().kind() {
							// ensure loaded
							component_id = Some(self.resolve(value.component()));
						}

						Class::Array(ArrayClass::new((*value.component()).clone(), component_id))
					}
				};

				self.define(id, class);
				id
			}
		}
	}

	fn load(&self, ty: &ObjectType) -> eyre::Result<InstanceClass> {
		let guard = self.sources.lock();
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
			let class = InstanceClass::parse(info, self)?;
			return Ok(class);
		}

		bail!("Failed to find a way to load {}", &**ty)
	}

	/// Forcefully loads all classes in a jar. This is used only in bootstrapping the java standard library.
	pub fn load_jar(&self, data: &[u8], filter: impl Fn(&str) -> bool) -> eyre::Result<()> {
		let reader = Cursor::new(data);
		let mut archive = zip::read::ZipArchive::new(reader)?;
		let mut map: Vec<String> = archive.file_names().map(|v| v.to_string()).collect();
		map.sort();
		for name in map {
			let mut file = archive.by_name(&name)?;
			if file.is_file() && file.name().ends_with(".class") && filter(file.name()) {
				let mut data = Vec::with_capacity(file.size() as usize);
				file.read_to_end(&mut data)?;
				self.load_class(&data)
					.wrap_err_with(|| format!("Failed to load {}", file.name()))?;
			}
		}
		Ok(())
	}

	/// Loads a java class to the JVM and injects it to the class table by locking it.
	#[instrument(skip_all)]
	pub fn load_class(&self, data: &[u8]) -> eyre::Result<Id<Class>> {
		let info = ClassInfo::parse_complete(data).wrap_err("Failed to parse .class file")?;
		let class = InstanceClass::parse(info, self)?;

		debug!("Parsed class {}", class.ty);

		let class = Class::Object(class);
		let id = self.allocate_id(class.cloned_ty());
		self.define(id, class);

		Ok(id)
	}

	fn allocate_id(&self, ty: Type) -> Id<Class> {
		self.classes.write().push(ty, None)
	}

	fn define(&self, id: Id<Class>, mut class: Class) {
		let ty = class.cloned_ty();
		debug!("Inject and defining new class {ty:?}");
		if self.classes.is_locked() {
			warn!("Classes are locked");
		}
		class.set_id(id);

		let mut guard = self.classes.write();
		let class_slot = guard.get_mut(id);
		*class_slot = Some(Arc::new(class));

		info!("Loaded class {ty:?} at {id:?}");
	}
}
