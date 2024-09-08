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
use rvm_gc::{AllocationError, GcSweeper};
use std::cell::Cell;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Weak};
use std::thread::{spawn, Builder, Thread};
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

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum CallType {
	Virtual,
	Static,
	Special,
	Interface,
}

impl CallType {
	pub fn is_static(&self) -> bool {
		matches!(self, CallType::Static)
	}
	pub fn is_special(&self) -> bool {
		matches!(self, CallType::Special)
	}
	pub fn is_interface(&self) -> bool {
		matches!(self, CallType::Interface)
	}
}

pub trait ThreadContext: 'static {
	fn yield_gc(&mut self);
	fn wait_until_gc(&mut self);

	fn run(
		&mut self,
		call_type: CallType,
		ty: &ObjectType,
		method: &MethodIdentifier,
		parameters: Vec<AnyValue>,
	) -> eyre::Result<Option<AnyValue>>;
}

pub struct Runtime<'thread> {
	pub vm: Vm,
	pub thread: Option<&'thread mut dyn ThreadContext>,
}

impl<'thread> Runtime<'thread> {
	pub fn gc(&mut self) {
		if let Some(thread) = &mut self.thread {
			let runtime = self.vm.clone();
			// TODO gc-thread
			let handle = spawn(move || {
				runtime.gc.gc();
			});
			thread.wait_until_gc();
			handle.join().unwrap();
		} else {
			// TODO gc-thread?
			// This is not a managed context, so we make this be the gc-thread
			self.vm.gc.gc();
		}
	}

	fn try_gc_op<O>(
		&mut self,
		mut func: impl FnMut(&mut Vm) -> Result<O, AllocationError>,
	) -> Result<O, AllocationError> {
		for _ in 0..5 {
			match func(&mut self.vm) {
				Ok(value) => {
					return Ok(value);
				}
				Err(AllocationError::OutOfHeap) => {
					self.gc();
				}
				err => {
					err?;
				}
			}
		}

		Err(AllocationError::OutOfHeap)
	}

	pub fn resolve_class(&mut self, ty: &Type) -> eyre::Result<Id<Class>> {
		let vm = self.vm.clone();

		let mut resolver = ClassResolver::new(&vm.classes);
		if vm.std.read().is_none() {
			let std = StdClasses {
				c_object: resolver.resolve(&ObjectType::Object().into())?,
				c_string: resolver.resolve(&ObjectType::String().into())?,
				c_class: resolver.resolve(&ObjectType::Class().into())?,
			};

			*vm.std.write() = Some(std);
		}

		let id = resolver
			.resolve(ty)
			.wrap_err_with(|| format!("Resolving class {ty:?}"))?;

		resolver.link_all(self).wrap_err("Linking")?;
		Ok(id)
	}

	pub fn alloc_object(&mut self, class: &InstanceClass) -> Result<AnyInstance, AllocationError> {
		self.try_gc_op(|runtime| Ok(runtime.gc.alloc_instance(class)?.resolve(runtime.clone())))
	}

	pub fn alloc_array(
		&mut self,
		component: &Class,
		length: u32,
	) -> Result<ArrayRef, AllocationError> {
		self.try_gc_op(|runtime| runtime.gc.alloc_array(component, length))
	}

	pub fn alloc_static_instance(
		&mut self,
		class: &InstanceClass,
	) -> Result<InstanceRef, AllocationError> {
		self.try_gc_op(|runtime| runtime.gc.alloc_static_instance(class))
	}

	pub fn run(
		&mut self,
		call_type: CallType,
		ty: &ObjectType,
		method: &MethodIdentifier,
		parameters: Vec<AnyValue>,
	) -> eyre::Result<Option<AnyValue>> {
		if let Some(thread) = &mut self.thread {
			thread.run(call_type, ty, method, parameters)
		} else {
			// TODO look into this
			let thread = self.vm.create_thread(ThreadConfig {
				name: "run".to_string(),
			});
			thread.run(ty.clone(), method.clone(), parameters);
			thread.join()
		}
	}
}

impl<'a> Deref for Runtime<'a> {
	type Target = Vm;

	fn deref(&self) -> &Self::Target {
		&self.vm
	}
}

/// A runtime which (almost never) conforms to [The Java Virtual Machine Specification, Java SE 19 Edition][jvms]
///
/// The runtime includes a bootstrap class source, a classloader
///
/// [jvms]: https://docs.oracle.com/javase/specs/jvms/se19/html/index.html
#[derive(Clone)]
pub struct Vm {
	inner: Arc<InnerVm>,
}

impl Vm {
	pub fn new(heap_size: usize, engine: Box<dyn Engine>) -> Vm {
		Vm {
			inner: Arc::new(InnerVm {
				classes: ClassLoader::new(),
				engine,
				gc: GarbageCollector::new(heap_size),
				bindings: RustBinder::new(),
				linker: Mutex::new(JNILinker::new()),
				started: Instant::now(),
				std: RwLock::new(None),
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

	//pub fn resolve_class(&self, ty: &Type) -> eyre::Result<Id<Class>> {
	// 		let mut resolver = ClassResolver::new(&self.classes);
	// 		let id = resolver
	// 			.resolve(ty)
	// 			.wrap_err_with(|| format!("Resolving class {ty:?}"))?;
	// 		resolver.link_all(self).wrap_err("Linking")?;
	// 		Ok(id)
	// 	}
	//
	// 	pub fn gc(&self) {
	// 		let inner = self.inner.clone();
	// 		spawn(move || {
	// 			let statistics = inner.gc.gc();
	// 			debug!(
	// 				"GC Complete: removed {} objects, {} remaining",
	// 				statistics.objects_cleared, statistics.objects_remaining
	// 			);
	// 		});
	// 	}
	//
	// 	pub fn alloc_object(&self, id: Id<Class>) -> Result<AnyInstance, AllocationError> {
	// 		let class = self.inner.classes.get(id);
	//
	// 		let class = class
	// 			.as_instance()
	// 			.expect("Id does not point to an instance class");
	//
	// 		Ok(self.gc.alloc_instance(class)?.resolve(self.clone()))
	// 	}
	//
	// 	pub fn simple_run(
	// 		&self,
	// 		ty: ObjectType,
	// 		method: MethodIdentifier,
	// 		parameters: Vec<AnyValue>,
	// 	) -> eyre::Result<Option<AnyValue>> {
	// 		let thread = self.create_thread(ThreadConfig {
	// 			name: "run".to_string(),
	// 		});
	// 		thread.run(ty, method, parameters);
	// 		thread.join()
	// 	}

	//pub fn weak(&self) -> WeakRuntime {
	//	WeakRuntime(Arc::downgrade(&self.inner))
	//}
}

impl Deref for Vm {
	type Target = InnerVm;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

//pub struct WeakRuntime(Weak<InnerRuntime>);
//
//impl WeakRuntime {
//	pub fn get(&self) -> Runtime {
//		Runtime {
//			inner: self
//				.0
//				.upgrade()
//				.expect("Tried to get a weak runtime, when it has been dropped."),
//		}
//	}
//}

pub struct InnerVm {
	pub classes: ClassLoader,
	engine: Box<dyn Engine>,
	pub gc: GarbageCollector,
	pub bindings: RustBinder,
	pub linker: Mutex<JNILinker>,
	pub started: Instant,
	pub std: RwLock<Option<StdClasses>>,
}

impl InnerVm {
	pub fn std(&self) -> StdClasses {
		self.std.read().unwrap()
	}
}

#[derive(Copy, Clone)]
pub struct StdClasses {
	pub c_object: Id<Class>,
	pub c_string: Id<Class>,
	pub c_class: Id<Class>,
}
