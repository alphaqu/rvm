use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crossbeam::channel::{unbounded, Receiver, Sender};
use crossbeam::sync::{Parker, Unparker};
use tracing::trace;

use crate::gc::{move_reference, ref_to_header, ObjectFlags, RootProvider};
use crate::object::Reference;

pub(super) fn new_sweeper() -> (GcSweeperHandle, GcSweeper) {
	let complete_parker: Parker = Default::default();
	let complete_unparker = complete_parker.unparker().clone();
	let parker: Parker = Default::default();
	let unparker = parker.unparker().clone();

	let should_yield: Arc<AtomicBool> = Arc::new(Default::default());

	let (sender, receiver) = unbounded();
	(
		GcSweeperHandle {
			unparker,
			complete_parker,
			sender,
			should_yield: should_yield.clone(),
		},
		GcSweeper {
			receiver,
			should_yield: should_yield.clone(),
			parker,
			complete: complete_unparker,
		},
	)
}

pub struct GcSweeperHandle {
	pub(super) unparker: Unparker,
	pub(super) complete_parker: Parker,
	pub(super) sender: Sender<bool>,
	pub(super) should_yield: Arc<AtomicBool>,
}

impl GcSweeperHandle {
	pub(super) fn start(&self, mark: bool) {
		self.should_yield.store(true, Ordering::Relaxed);
		self.sender.send(mark).unwrap();
		self.complete_parker.park();
		self.should_yield.store(false, Ordering::Relaxed);
	}

	pub(super) fn start_marking(&self) {
		self.unparker.unpark();
		self.complete_parker.park();
	}

	pub(super) fn move_roots(&self) {
		self.unparker.unpark();
		self.complete_parker.park();
	}

	pub(super) fn continue_execution(&self) {
		self.unparker.unpark();
	}
}

pub struct GcSweeper {
	// Gives which mark
	pub(super) receiver: Receiver<bool>,
	pub(super) should_yield: Arc<AtomicBool>,
	pub(super) parker: Parker,
	pub(super) complete: Unparker,
}

impl GcSweeper {
	pub fn yield_gc(roots: &mut impl RootProvider) {
		let sweeper = roots.sweeper();
		if sweeper.should_yield.load(Ordering::Relaxed) {
			if let Ok(mark) = sweeper.receiver.try_recv() {
				Self::gc(mark, roots);
			}
		}
	}

	pub fn wait_until_gc(roots: &mut impl RootProvider) {
		let mark = roots
			.sweeper()
			.receiver
			.recv_timeout(Duration::from_secs_f32(5.0))
			.expect("GC timeout");
		Self::gc(mark, roots);
	}

	fn gc(mark: bool, roots: &mut impl RootProvider) {
		// Wait until gc is ready to start marking
		let sweeper = roots.sweeper();
		sweeper.complete.unpark();
		sweeper.parker.park();

		// mark all of the objects
		roots.mark_roots(GcMarker { mark });

		// Wait until all marking has been complete, and the objects have found their new location
		let sweeper = roots.sweeper();
		sweeper.complete.unpark();
		sweeper.parker.park();
		roots.remap_roots(|r| unsafe { move_reference(r) });

		// Wait until gc has moved all of the objects
		let sweeper = roots.sweeper();
		sweeper.complete.unpark();
		sweeper.parker.park();
	}
}

pub struct GcMarker {
	pub(super) mark: bool,
}

impl GcMarker {
	pub fn mark(&self, reference: Reference) {
		if reference.is_null() {
			return;
		}

		unsafe {
			let obj = ref_to_header(reference);
			let object_mark = (*obj).flags.contains(ObjectFlags::MARK);
			if object_mark == self.mark {
				// We have already visited this object so we return here.
				return;
			}

			trace!("Visiting {:?}", obj);
			// we toggle the mark to say that we have visited/visiting this object.
			(*obj).flags.set(ObjectFlags::MARK, self.mark);

			reference.visit_refs(|value| {
				self.mark(value);
			});
		}
	}
}
