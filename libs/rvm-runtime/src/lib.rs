#![feature(array_try_from_fn)]
#![feature(thread_local)]
#![feature(thread_id_value)]
#![feature(c_variadic)]
#![feature(fn_traits)]
#![feature(new_uninit)]
#![feature(iterator_try_collect)]

use crate::engine::{Engine, ThreadConfig, ThreadHandle};
use crate::gc::GarbageCollector;
use crate::native::JNILinker;
use ahash::HashMap;
pub use binding::*;
pub use conversion::*;
use eyre::Context;
pub use object::*;
use parking_lot::{Mutex, RwLock};
use rvm_core::{Id, ObjectType, Type};
use rvm_gc::AllocationError;
use std::ops::Deref;
use std::sync::{Arc, Weak};
use std::thread::spawn;
use std::time::Instant;
use tracing::debug;
pub use value::*;

mod binding;
mod conversion;
pub mod engine;
pub mod error;
pub mod gc;
pub mod native;
mod object;
pub mod prelude;
mod value;

/// A runtime which (almost never) conforms to [The Java Virtual Machine Specification, Java SE 19 Edition][jvms]
///
/// The runtime includes a bootstrap class source, a classloader
///
/// [jvms]: https://docs.oracle.com/javase/specs/jvms/se19/html/index.html
#[derive(Clone)]
pub struct Runtime {
	inner: Arc<InnerRuntime>,
}

impl Runtime {
	pub fn new(heap_size: usize, engine: Box<dyn Engine>) -> Runtime {
		Runtime {
			inner: Arc::new(InnerRuntime {
				classes: ClassLoader::new(),
				engine,
				gc: GarbageCollector::new(heap_size),
				bindings: RustBinder::new(),
				linker: Mutex::new(JNILinker::new()),
				started: Instant::now(),
			}),
		}
	}

	pub fn create_thread(&self, config: ThreadConfig) -> ThreadHandle {
		self.inner.engine.create_thread(self.clone(), config)
	}

	pub fn is_instance_of(&self, instance: InstanceRef, id: Id<Class>) -> bool {
		let mut this_id = instance.header().id;

		loop {
			if this_id == id {
				return true;
			}

			let this_class = self.classes.get(this_id);
			let instance_class = this_class.as_instance().unwrap();

			// Check if interfaces contain this
			for interface in &instance_class.interfaces {
				if interface.id == id {
					return true;
				}
			}

			if let Some(super_class) = &instance_class.super_class {
				this_id = super_class.id;
			} else {
				return false;
			}
		}
	}

	pub fn resolve_class(&self, ty: &Type) -> eyre::Result<Id<Class>> {
		let mut resolver = ClassResolver::new(&self.classes);
		let id = resolver
			.resolve(ty)
			.wrap_err_with(|| format!("Resolving class {ty:?}"))?;
		resolver.link_all(self).wrap_err("Linking")?;
		Ok(id)
	}

	pub fn gc(&self) {
		let inner = self.inner.clone();
		spawn(move || {
			let statistics = inner.gc.gc();
			debug!(
				"GC Complete: removed {} objects, {} remaining",
				statistics.objects_cleared, statistics.objects_remaining
			);
		});
	}

	pub fn alloc_object(&self, id: Id<Class>) -> Result<AnyInstance, AllocationError> {
		let class = self.inner.classes.get(id);

		let class = class
			.as_instance()
			.expect("Id does not point to an instance class");

		Ok(self.gc.alloc_instance(class)?.resolve(self.clone()))
	}

	pub fn simple_run(
		&self,
		ty: ObjectType,
		method: MethodIdentifier,
		parameters: Vec<AnyValue>,
	) -> eyre::Result<Option<AnyValue>> {
		let thread = self.create_thread(ThreadConfig {
			name: "run".to_string(),
		});
		thread.run(ty, method, parameters);
		thread.join()
	}

	pub fn weak(&self) -> WeakRuntime {
		WeakRuntime(Arc::downgrade(&self.inner))
	}
}

impl Deref for Runtime {
	type Target = InnerRuntime;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

pub struct WeakRuntime(Weak<InnerRuntime>);

impl WeakRuntime {
	pub fn get(&self) -> Runtime {
		Runtime {
			inner: self
				.0
				.upgrade()
				.expect("Tried to get a weak runtime, when it has been dropped."),
		}
	}
}

pub struct InnerRuntime {
	pub classes: ClassLoader,
	engine: Box<dyn Engine>,
	pub gc: GarbageCollector,
	pub bindings: RustBinder,
	pub linker: Mutex<JNILinker>,
	pub started: Instant,
}
