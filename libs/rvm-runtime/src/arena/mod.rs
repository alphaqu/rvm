mod active_plan;
mod collection;
mod object_model;
mod reference_glue;
mod scanning;
pub mod object;

use std::mem::size_of;
use std::ops::Deref;
use crate::arena::reference_glue::VMReferenceGlue;
use mmtk::vm::edge_shape::{SimpleEdge, UnimplementedMemorySlice};
use mmtk::vm::VMBinding;
use std::borrow::BorrowMut;
use std::cell::RefCell;

pub const HEADER_SIZE: usize = size_of::<u32>();

#[derive(Default)]
pub struct Arena {}

impl VMBinding for Arena {
	type VMObjectModel = VMObjectModel;
	type VMScanning = VMScanning;
	type VMCollection = VMCollection;
	type VMActivePlan = VMActivePlan;
	type VMReferenceGlue = VMReferenceGlue;
	type VMEdge = SimpleEdge;
	type VMMemorySlice = UnimplementedMemorySlice;
}
thread_local! {
	pub static MUTATOR: RefCell<Box<Mutator<Arena>>> = RefCell::new(memory_manager::bind_mutator(SINGLETON.deref(), VMMutatorThread(VMThread::UNINITIALIZED)))
}
impl Arena {
	pub fn init(heap_size: usize) -> Arena {
		// set heap size first
		{
			let mut builder = BUILDER.lock().unwrap();
			let success = builder.options.heap_size.set(heap_size);
			assert!(success, "Failed to set heap size to {}", heap_size);
		}

		{
			let mut builder = BUILDER.lock().unwrap();
			let success = builder.options.plan.set(PlanSelector::MarkSweep);
			assert!(success, "Failed to set plan to {:?}", PlanSelector::MarkSweep);
		}

		// Make sure MMTk has not yet been initialized
		assert!(!MMTK_INITIALIZED.load(Ordering::SeqCst));
		// Initialize MMTk here
		lazy_static::initialize(&SINGLETON);
		memory_manager::initialize_collection(&SINGLETON, VMThread::UNINITIALIZED);
		Arena {

		}
	}

	pub fn gc(&self) {
		println!("{:?}", SINGLETON.get_plan().options().plan.deref());
		SINGLETON.get_plan().handle_user_collection_request(VMMutatorThread(VMThread::UNINITIALIZED), true);
	}

	pub fn alloc(&self, id: Id<Class>, class_loader: &ClassLoader) -> ObjectReference {
		let guard = class_loader.get(id);
		match &guard.kind {
			ClassKind::Object(object) => {
				let size = object.size();
				let semantics = AllocationSemantics::Default;

				debug!("Allocating {size}");
				return MUTATOR.with(|mutator| unsafe {
					let mutator = &mut **mutator.borrow_mut();
					let addr = memory_manager::alloc(mutator, size, 8, 0, semantics);

					let object = ObjectReference::from_raw_address(addr.add(OBJECT_REF_OFFSET));
					memory_manager::post_alloc(mutator, object, size, semantics);
					object.to_header::<Arena>().store(id.idx());
					object
				});
			}
			ClassKind::Array(_) => {
				todo!()
			}
			ClassKind::Primitive(_) => {
				panic!("Cannot allocate primative")
			}
		}

		todo!()
	}
}

use crate::arena::active_plan::VMActivePlan;
use crate::arena::collection::VMCollection;
use crate::arena::object_model::{OBJECT_REF_OFFSET, VMObjectModel};
use crate::arena::scanning::VMScanning;
use lazy_static::lazy_static;
use mmtk::{MMTKBuilder, MMTK, memory_manager, Mutator, AllocationSemantics};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use mmtk::util::{ObjectReference, VMMutatorThread, VMThread};
use mmtk::util::options::PlanSelector;
use tracing::debug;
use rvm_core::Id;
use rvm_object::{Class, ClassKind, ClassLoader, ObjectClass};

/// This is used to ensure we initialize MMTk at a specified timing.
pub static MMTK_INITIALIZED: AtomicBool = AtomicBool::new(false);

lazy_static! {
	pub static ref BUILDER: Mutex<MMTKBuilder> = Mutex::new(MMTKBuilder::new());
	pub static ref SINGLETON: MMTK<Arena> = {
		let builder = BUILDER.lock().unwrap();
		debug_assert!(!MMTK_INITIALIZED.load(Ordering::SeqCst));
		let ret = mmtk::memory_manager::mmtk_init(&builder);
		MMTK_INITIALIZED.store(true, std::sync::atomic::Ordering::Relaxed);
		*ret
	};
}
