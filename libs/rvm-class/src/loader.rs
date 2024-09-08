use crate::instance::Class;
use crate::source::ClassSource;
use crate::ClassResolver;
use eyre::{bail, Context};
use parking_lot::RwLock;
use rvm_core::{Id, ObjectType, Storage};
use rvm_reader::ClassInfo;
use std::sync::Arc;
use tracing::{debug, info, warn};

pub struct ClassLoader {
	classes: RwLock<Storage<ObjectType, Class, Option<Arc<Class>>>>,
	inner: RwLock<ClassLoaderInner>,
}

impl ClassLoader {
	pub fn new() -> ClassLoader {
		ClassLoader {
			inner: RwLock::new(ClassLoaderInner::new()),
			classes: RwLock::new(Storage::new()),
		}
	}

	pub fn add_source(&self, source: Box<dyn ClassSource>) {
		self.inner.write().sources.push(source);
	}

	pub fn get(&self, id: Id<Class>) -> Arc<Class> {
		if id == Id::null() {
			panic!("Null value");
		}

		self.classes
			.read()
			.get(id)
			.clone()
			.expect("Class never loaded")
	}

	pub fn get_named(&self, ty: &ObjectType) -> Option<Id<Class>> {
		self.classes.read().get_id(ty)
	}

	pub fn load(&self, desc: &ObjectType) -> LoadResult {
		let option = self.classes.read().get_id(desc);
		match option {
			Some(value) => LoadResult::Existing(value),
			None => {
				let func = || {
					let id = self.allocate_id(desc.clone());
					info!("Resolving class {desc:?}");
					let class = self
						.load_instance(id, desc)
						.wrap_err_with(|| format!("Loading instance {desc}"))?;
					self.define(class);
					Ok(id)
				};

				LoadResult::New(func())
			}
		}
	}

	pub fn resolve(&self, id: Id<Class>, mut func: &mut ClassResolver) -> eyre::Result<()> {
		self.modify(id, |class| class.resolve(func))
	}

	pub fn modify<O>(
		&self,
		id: Id<Class>,
		func: impl FnOnce(&mut Class) -> eyre::Result<O>,
	) -> eyre::Result<O> {
		// Acquire class
		let mut guard = self.classes.write();
		let class = guard.get_mut(id).take().expect("Class already modifying");
		drop(guard);

		// Unwrap class
		let mut class = match Arc::try_unwrap(class) {
			Ok(value) => value,
			Err(error) => {
				*self.classes.write().get_mut(id) = Some(error);
				bail!("Could not acquire class lock (there are active Arc references)")
			}
		};

		// Run func
		let result = func(&mut class);

		// Put class back.
		let mut guard = self.classes.write();
		*guard.get_mut(id) = Some(Arc::new(class));

		result
	}

	fn load_instance(&self, id: Id<Class>, ty: &ObjectType) -> eyre::Result<Class> {
		let guard = self.inner.read();
		for source in guard.sources.iter() {
			let Some(data) = source
				.try_load(ty)
				.wrap_err("Failed to load class from source")?
			else {
				continue;
			};

			// Deadlock... oh shit, this is trademarked now
			drop(guard);

			let info = ClassInfo::parse_complete(&data).wrap_err("Failed to parse .class file")?;
			let class = Class::new(id, info)?;
			return Ok(class);
		}

		bail!("Failed to find a way to load {}", &**ty)
	}

	fn allocate_id(&self, ty: ObjectType) -> Id<Class> {
		self.classes.write().push(ty, None)
	}

	fn define(&self, class: Class) {
		let id = class.id;
		let ty = class.ty.clone();

		debug!("Inject and defining new class {ty:?}");
		if self.classes.is_locked() {
			warn!("Classes are locked");
		}

		let mut guard = self.classes.write();
		let class_slot = guard.get_mut(id);
		*class_slot = Some(Arc::new(class));

		info!("Loaded class {ty:?} at {id:?}");
		//self.inner.write().to_resolve.push(id);
	}
}

struct ClassLoaderInner {
	sources: Vec<Box<dyn ClassSource>>,
}

impl ClassLoaderInner {
	pub fn new() -> Self {
		Self { sources: vec![] }
	}
}

pub enum LoadResult {
	Existing(Id<Class>),
	New(eyre::Result<Id<Class>>),
}
impl LoadResult {
	pub fn to_result(self) -> eyre::Result<Id<Class>> {
		match self {
			LoadResult::Existing(id) => Ok(id),
			LoadResult::New(result) => result,
		}
	}
}
