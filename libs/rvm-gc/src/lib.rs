#![feature(const_align_offset)]
#![feature(pointer_is_aligned_to)]
#![feature(slice_ptr_get)]

mod collector;
mod header;
mod reference;
mod sweeper;

pub use collector::*;
pub use header::*;
pub use reference::*;
use std::marker::PhantomData;
pub use sweeper::*;

pub trait GcUser: Sized {
	type Header: Sized;

	unsafe fn drop_ref(reference: GcRef<Self>);

	// Go through all the references which this reference contains.
	fn visit_refs(reference: &GcRef<Self>, visitor: impl FnMut(GcRef<Self>));

	// Go through all the references which this reference contains, and replace them with the new value given by visitor.
	fn map_refs(reference: &GcRef<Self>, visitor: impl FnMut(GcRef<Self>) -> GcRef<Self>);
}

pub trait RootProvider<U: GcUser> {
	fn mark_roots(&mut self, marker: GcMarker);

	fn remap_roots(&mut self, mapper: impl FnMut(GcRef<U>) -> GcRef<U>);

	fn sweeper(&mut self) -> &mut GcSweeper;
}

pub struct VecRootProvider<U: GcUser> {
	references: Vec<Option<GcRef<U>>>,
	gc_sweeper: GcSweeper,
	_u: PhantomData<fn() -> U>,
}

impl<U: GcUser> VecRootProvider<U> {
	pub fn new(sweeper: GcSweeper) -> VecRootProvider<U> {
		VecRootProvider {
			references: vec![],
			gc_sweeper: sweeper,
			_u: Default::default(),
		}
	}

	pub fn wait_until_gc(&mut self) {
		GcSweeper::wait_until_gc(self);
	}

	pub fn add(&mut self, reference: GcRef<U>) -> usize {
		if self.references.iter().any(|v| v == &Some(reference)) {
			panic!("Already in here");
		}
		self.references.push(Some(reference));
		self.references.len() - 1
	}

	pub fn remove(&mut self, index: usize) {
		self.references[index] = None;
	}
	pub fn get(&self, index: usize) -> GcRef<U> {
		self.references[index].unwrap()
	}
}
impl<U: GcUser> RootProvider<U> for VecRootProvider<U> {
	fn mark_roots(&mut self, marker: GcMarker) {
		for reference in self.references.iter().flatten() {
			marker.mark(*reference);
		}
	}

	fn remap_roots(&mut self, mut mapper: impl FnMut(GcRef<U>) -> GcRef<U>) {
		for reference in self.references.iter_mut().flatten() {
			*reference = mapper(*reference);
		}
	}

	fn sweeper(&mut self) -> &mut GcSweeper {
		&mut self.gc_sweeper
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crossbeam::sync::{Parker, Unparker};
	use rand::distributions::{Alphanumeric, DistString};
	use rand::{random, thread_rng, Rng};
	use std::ops::Deref;
	use std::ptr;
	use std::ptr::slice_from_raw_parts_mut;
	use std::slice::{from_raw_parts, from_raw_parts_mut};
	use std::sync::Arc;
	use std::thread::spawn;
	use std::thread::{sleep, JoinHandle};
	use std::time::Duration;
	use tracing::{info, warn};

	pub struct SimpleHeader {
		is_good: u32,
		fields: u32,
	}

	#[derive(Copy, Clone, Eq, PartialEq, Debug)]
	pub struct Reference(GcRef<SimpleUser>);

	impl Reference {
		pub fn new(
			gc: &GarbageCollector<SimpleUser>,
			fields: &[Field],
		) -> Result<Reference, AllocationError> {
			let mut result = Reference(gc.alloc_raw(
				size_of_val(fields),
				SimpleHeader {
					is_good: 0xff00ff00,
					fields: fields.len() as u32,
				},
			)?);

			unsafe {
				let dst = result.fields_mut_ptr();
				let src_len = fields.len();
				let dst_len = dst.len();
				assert_eq!(dst_len, src_len);

				for i in 0..src_len {
					dst.as_mut_ptr().add(i).write(fields[i].clone());
				}
			}

			Ok(result)
		}

		pub fn fields(&self) -> &[Field] {
			let header = self.0.header();
			let field_count = header.user.fields;
			let fields = self.0.data_ptr() as *const Field;

			unsafe { from_raw_parts(fields, field_count as usize) }
		}
		pub fn fields_mut_ptr(&mut self) -> *mut [Field] {
			let header = self.0.header();
			let field_count = header.user.fields;
			slice_from_raw_parts_mut(self.0.data_ptr() as *mut Field, field_count as usize)
		}
		pub fn fields_mut(&mut self) -> &mut [Field] {
			let header = self.0.header();
			let field_count = header.user.fields;
			let fields = self.0.data_ptr() as *mut Field;

			unsafe { from_raw_parts_mut(fields, field_count as usize) }
		}
	}

	pub struct SimpleUser {}
	impl GcUser for SimpleUser {
		type Header = SimpleHeader;

		unsafe fn drop_ref(reference: GcRef<Self>) {
			let reference = Reference(reference);
			let header = reference.0.header();
			let field_count = header.user.fields;
			let fields = reference.0.data_ptr() as *mut Field;
			ptr::drop_in_place(slice_from_raw_parts_mut(fields, field_count as usize))
		}

		fn visit_refs(reference: &GcRef<Self>, mut visitor: impl FnMut(GcRef<Self>)) {
			let reference = Reference(*reference);
			for field in reference.fields() {
				if let Field::Ref(reference) = field {
					visitor(reference.0);
				}
			}
		}

		fn map_refs(reference: &GcRef<Self>, mut visitor: impl FnMut(GcRef<Self>) -> GcRef<Self>) {
			let mut reference = Reference(*reference);
			for field in reference.fields_mut() {
				if let Field::Ref(reference) = field {
					*reference = Reference(visitor(reference.0));
				}
			}
		}
	}

	#[derive(Clone)]
	pub struct Gc {
		inner: Arc<GarbageCollector<SimpleUser>>,
	}

	impl Gc {
		pub fn new(size: usize) -> Gc {
			rvm_core::init();
			Gc {
				inner: Arc::new(GarbageCollector::<SimpleUser>::new(size)),
			}
		}

		pub fn alloc(&self, fields: &[Field]) -> Reference {
			self.try_alloc(fields).unwrap()
		}

		pub fn try_alloc(&self, fields: &[Field]) -> Result<Reference, AllocationError> {
			Reference::new(&self.inner, fields)
		}

		pub fn gc(&self) -> GCStatistics {
			self.inner.gc()
		}
	}

	#[derive(Clone, Eq, PartialEq, Debug)]
	pub enum Field {
		Name(String),
		Ref(Reference),
	}

	impl Field {
		pub fn reference(&self) -> Reference {
			if let Field::Ref(reference) = self {
				return *reference;
			}

			panic!("Not ref")
		}
	}
	#[test]
	fn test_simple_alloc() {
		let mut gc = Gc::new(1024);
		let original_fields = vec![
			Field::Name("Cringe man".to_string()),
			Field::Name("Cringe man 2".to_string()),
			Field::Name("Cringe man 3".to_string()),
		];
		let mut result = gc.alloc(&original_fields);

		// Header validation
		assert_eq!(result.0.header().user.is_good, 0xff00ff00);
		assert_eq!(result.fields().len(), 3);

		// Field validation
		let fields = result.fields_mut();
		for i in 0..3 {
			assert_eq!(fields[i], original_fields[i]);
		}

		assert_eq!(result.0.header().user.is_good, 0xff00ff00);
		assert_eq!(result.fields().len(), 3);

		let field = result.0.data_ptr() as *const Field;
		assert_eq!(unsafe { &*field }, &Field::Name("Cringe man".to_string()));

		// Hi
	}

	#[test]
	fn simple_gc() {
		let mut gc = Gc::new(1024);
		let mut result = gc.alloc(&vec![
			Field::Name("Cringe man".to_string()),
			Field::Name("Cringe man 2".to_string()),
			Field::Name("Cringe man 3".to_string()),
		]);

		let stats = gc.inner.gc();
		assert_eq!(stats.objects_cleared, 1);
		assert_eq!(stats.objects_remaining, 0);
	}

	#[test]
	fn simple_gc_frozen() {
		let mut gc = Gc::new(1024);

		let mut result = gc.alloc(&vec![
			Field::Name("Cringe man".to_string()),
			Field::Name("Cringe man 2".to_string()),
			Field::Name("Cringe man 3".to_string()),
		]);
		gc.inner.add_frozen(result.0);

		let stats = gc.inner.gc();
		assert_eq!(stats.objects_cleared, 0);
		assert_eq!(stats.objects_remaining, 1);
	}

	pub struct RootedTester {
		gc: Gc,
		users: Vec<(Parker, JoinHandle<()>)>,
	}

	pub struct RootedUser {
		roots: VecRootProvider<SimpleUser>,
		unparker: Unparker,
		gc: Gc,
	}
	impl RootedUser {
		pub fn wait_for_gc(&mut self) {
			//warn!("Waiting for gc, (unparking)");
			self.unparker.unpark();
			self.roots.wait_until_gc();
		}

		pub fn keep(&mut self, reference: Reference) -> usize {
			self.roots.add(reference.0)
		}

		pub fn get(&mut self, id: usize) -> Reference {
			Reference(self.roots.get(id))
		}
		pub fn unkeep(&mut self, id: usize) {
			self.roots.remove(id);
		}
	}
	impl Deref for RootedUser {
		type Target = Gc;

		fn deref(&self) -> &Self::Target {
			&self.gc
		}
	}

	impl RootedTester {
		pub fn new(size: usize) -> RootedTester {
			RootedTester {
				gc: Gc::new(size),
				users: vec![],
			}
		}
		pub fn spawn_user(&mut self, func: impl FnOnce(RootedUser) + Send + 'static) {
			let sweeper = self.gc.inner.new_sweeper();
			let parker = Parker::new();
			let unparker = parker.unparker().clone();

			let user_gc = self.gc.clone();
			let handle = spawn(move || {
				info!("Spawning user");
				let user = RootedUser {
					roots: VecRootProvider::<SimpleUser>::new(sweeper),
					unparker,
					gc: user_gc,
				};
				info!("Finishing spawn user");
				user.unparker.unpark();
				func(user);
				info!("User func finished");
			});

			info!("Waiting for init");
			parker.park();
			self.users.push((parker, handle));
		}

		pub fn gc(&self) -> GCStatistics {
			for (parker, _) in &self.users {
				//	warn!("Waiting for gc unpark, (park)");
				parker.park();
			}
			//info!("Running GC");

			self.inner.gc()
		}
	}

	impl Deref for RootedTester {
		type Target = Gc;

		fn deref(&self) -> &Self::Target {
			&self.gc
		}
	}
	impl Drop for RootedTester {
		fn drop(&mut self) {
			for (user, handle) in self.users.drain(..) {
				handle.join().unwrap();
			}
		}
	}

	fn fields(count: usize) -> Vec<Field> {
		let mut out = Vec::new();

		let mut rng = rand::thread_rng();
		for i in 0..count {
			out.push(Field::Name(Alphanumeric.sample_string(&mut rng, 128)))
		}
		out
	}
	#[test]
	fn simple_gc_root() {
		let mut tester = RootedTester::new(1024);

		let _ = tester.alloc(&[Field::Name("Hi, im about to get yeeted".to_string())]);

		tester.spawn_user(|mut tester| {
			let original_fields = fields(2);
			let reference = tester.alloc(&original_fields);
			let i = tester.keep(reference);
			tester.wait_for_gc(); // GC 1
			let reference = tester.get(i);
			assert_eq!(reference.fields(), &original_fields);
			tester.unkeep(i);
			tester.wait_for_gc(); // GC 2
		});

		let stats = tester.gc(); // GC 1
		assert_eq!(stats.objects_cleared, 1);
		assert_eq!(stats.objects_remaining, 1);

		let stats = tester.gc(); // GC 2
		assert_eq!(stats.objects_cleared, 1);
		assert_eq!(stats.objects_remaining, 0);
	}

	#[test]
	fn cyclic_gc() {
		let mut tester = RootedTester::new(1024);

		let _ = tester.alloc(&fields(3));

		tester.spawn_user(|mut tester| {
			let mut fields_1 = fields(3);
			let mut ref_1 = tester.alloc(&fields_1);

			let mut fields_2 = fields(3);
			let mut ref_2 = tester.alloc(&fields_2);

			ref_1.fields_mut()[1] = Field::Ref(ref_2);
			ref_2.fields_mut()[2] = Field::Ref(ref_1);

			let i_1 = tester.keep(ref_1);

			tester.wait_for_gc(); // GC 1
			let ref_1 = tester.get(i_1);
			let ref_2 = ref_1.fields()[1].reference();

			fields_1[1] = Field::Ref(ref_2);
			fields_2[2] = Field::Ref(ref_1);

			assert_eq!(ref_1.fields(), fields_1);
			assert_eq!(ref_2.fields(), fields_2);

			// We unkeep the only link to the cyclic reference
			tester.unkeep(i_1);

			tester.wait_for_gc(); // GC 2
		});

		let stats = tester.gc(); // GC 1
		assert_eq!(stats.objects_cleared, 1);
		assert_eq!(stats.objects_remaining, 2);

		let stats = tester.gc(); // GC 2
		assert_eq!(stats.objects_cleared, 2);
		assert_eq!(stats.objects_remaining, 0);
	}

	#[test]
	fn lots_of_gc() {
		let mut tester = RootedTester::new(512);

		let mut rng = thread_rng();
		let mut allocated = 0;
		for i in 0..128 {
			info!("Run {i}/128 {:.0}%", (i as f32 / 128.0) * 100.0);

			let field_count = rng.gen_range(1..5);
			match tester.try_alloc(&fields(field_count)) {
				Ok(_) => {
					allocated += 1;
				}
				Err(AllocationError::OutOfHeap) => {
					info!("GARBAGING!!");

					let stats = tester.gc();
					assert_eq!(stats.objects_cleared, allocated);
					assert_eq!(stats.objects_remaining, 0);
					allocated = 0;
				}
				other => {
					other.unwrap();
				}
			}
		}
	}

	#[test]
	fn child_gc() {
		let mut gc = Gc::new(1024);
		let child = gc.alloc(&vec![Field::Name("Hi baby girl".to_string())]);
		let child2 = gc.alloc(&vec![Field::Name("Hi baby girl again".to_string())]);
		let mut result = gc.alloc(&vec![
			Field::Ref(child),
			Field::Name("Cringe man 2".to_string()),
			Field::Name("Cringe man 3".to_string()),
		]);

		result.fields_mut()[1] = Field::Ref(child2);

		let stats = gc.inner.gc();
		assert_eq!(stats.objects_cleared, 3);
		assert_eq!(stats.objects_remaining, 0);
	}
}
