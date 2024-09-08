use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::header::ObjectFlags;
use crate::reference::GcRef;
use crate::{GcUser, RootProvider};
use crossbeam::channel::{unbounded, Receiver, Sender};
use crossbeam::sync::{Parker, Unparker};
use tracing::{debug, trace};
use uuid::Uuid;

pub(super) fn new_sweeper() -> (GcSweeperHandle, GcSweeper) {
	let complete_parker: Parker = Default::default();
	let complete_unparker = complete_parker.unparker().clone();
	let parker: Parker = Default::default();
	let unparker = parker.unparker().clone();

	let should_yield: Arc<AtomicBool> = Arc::new(Default::default());

	let (sender, receiver) = unbounded();
	let uuid = Uuid::new_v4();
	(
		GcSweeperHandle {
			uuid,
			unparker,
			complete_parker,
			sender,
			should_yield: should_yield.clone(),
		},
		GcSweeper {
			uuid,
			finished: false,
			receiver,
			should_yield: should_yield.clone(),
			parker,
			complete: complete_unparker,
		},
	)
}

pub struct GcSweeperHandle {
	pub(super) uuid: Uuid,
	pub(super) unparker: Unparker,
	pub(super) complete_parker: Parker,
	pub(super) sender: Sender<bool>,
	pub(super) should_yield: Arc<AtomicBool>,
}

impl GcSweeperHandle {
	pub(super) fn start(&self, mark: bool) {
		self.should_yield.store(true, Ordering::Relaxed);
		self.sender.send(mark).unwrap();
		debug!("Waiting for {}", self.uuid);
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
	pub(super) uuid: Uuid,
	pub(super) finished: bool,
	// Gives which mark
	pub(super) receiver: Receiver<bool>,
	pub(super) should_yield: Arc<AtomicBool>,
	pub(super) parker: Parker,
	pub(super) complete: Unparker,
}

impl Drop for GcSweeper {
	fn drop(&mut self) {
		if !self.finished {
			panic!("Sweeper has not been removed from garbage collector");
		}
	}
}

impl GcSweeper {
	pub fn yield_gc<U: GcUser>(roots: &mut impl RootProvider<U>) {
		let sweeper = roots.sweeper();
		if sweeper.should_yield.load(Ordering::Relaxed) {
			if let Ok(mark) = sweeper.receiver.try_recv() {
				Self::gc(mark, roots);
			}
		}
	}

	pub fn wait_until_gc<U: GcUser>(roots: &mut impl RootProvider<U>) {
		let mark = roots
			.sweeper()
			.receiver
			.recv_timeout(Duration::from_secs_f32(5.0))
			.expect("GC timeout");
		Self::gc(mark, roots);
	}

	fn gc<U: GcUser>(mark: bool, roots: &mut impl RootProvider<U>) {
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
		roots.remap_roots(|r| unsafe { r.forward() });

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
	pub fn mark<U: GcUser>(&self, mut reference: GcRef<U>) {
		if reference.is_null() {
			return;
		}

		let header = reference.header_mut();
		let object_mark = header.flags.contains(ObjectFlags::MARK);
		if object_mark == self.mark {
			// We have already visited this object so we return here.
			return;
		}

		trace!("Visiting {:?}", reference);

		// we toggle the mark to say that we have visited/visiting this object.
		let header = reference.header_mut();
		header.flags.set(ObjectFlags::MARK, self.mark);

		reference.visit_refs(|value| {
			self.mark(value);
		});
	}
}
