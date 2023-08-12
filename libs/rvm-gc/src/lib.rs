use std::alloc::{alloc_zeroed, Layout};
use std::mem::transmute;
use std::ptr::copy;

use tracing::{debug, trace};

pub use crate::object::{Object, ObjectFlags, ObjectPointer, ObjectSize, OBJECT_HEADER};

mod object;

pub const OBJECT_ALIGNMENT: usize = 32;
pub struct GarbageCollector<C: GCClient> {
	client: C,
	mark: bool,
	size: usize,
	objects: usize,
	free: *mut u8,
	data: *mut u8,
}

impl<C: GCClient> GarbageCollector<C> {
	pub fn new(client: C, size: usize) -> GarbageCollector<C> {
		let layout = Layout::from_size_align(size, 32).unwrap();
		let data = unsafe { alloc_zeroed(layout) };

		GarbageCollector {
			client,
			mark: false,
			size: size * 8,
			objects: 0,
			free: data,
			data,
		}
	}

	pub fn client(&mut self) -> &mut C {
		&mut self.client
	}

	pub fn gc(&mut self) -> GCStatistics {
		debug!("Starting garbage collection");
		self.mark = !self.mark;

		// Visit all of the objects, and mark them as visitable.
		self.client.visit_roots(|pointer| {
			self.visit_pointer(pointer);
		});

		// Go through all objects, and find the location where the object will soon be moved to,
		// we store this in the forward field in the object so we can move references in step 3.
		let mut new_free = self.data;
		let mut alive_objects = 0;
		self.walk_alive(|pointer| unsafe {
			alive_objects += 1;

			// Set the forward field to the soon to be the new object location.
			(*pointer.0).forward = new_free as usize;

			trace!("Object {:?} next is {:?} ", pointer.0, new_free);

			// Increment the free pointer by the size of the object
			let object_size = (*pointer.0).size as usize + OBJECT_HEADER;
			new_free = new_free.add(Self::align_size(object_size));
		});

		// Update all of the object edges to the new object locations.
		self.walk_alive(|pointer| {
			// Go through all of the objects edges, and move them to the new child object location.
			self.client.map_edges(pointer, |edge| unsafe {
				if edge.is_null() {
					return ObjectPointer::NULL;
				}

				let new = (*edge.0).forward;
				trace!(
					"Moving {:?}'s field {:?} to {:?}",
					pointer.0,
					edge.0,
					new as *mut u8
				);
				ObjectPointer(new as *mut Object)
			});
		});

		// Go through the live objects, and move them to their new locations.
		self.walk_alive(|pointer| unsafe {
			let new_location = (*pointer.0).forward;
			let size = Self::align_size((*pointer.0).size as usize + OBJECT_HEADER);
			copy(pointer.0 as *mut u8, new_location as *mut u8, size);
		});

		// Set the free pointer to the new limit.
		self.free = new_free;

		let statistics = GCStatistics {
			objects_cleared: self.objects - alive_objects,
			objects_remaining: alive_objects,
		};
		self.objects = alive_objects;
		statistics
	}

	fn visit_pointer(&self, pointer: ObjectPointer) {
		if pointer.is_null() {
			return;
		}
		unsafe {
			let object_mark = (*pointer.0).flags.contains(ObjectFlags::MARK);
			if object_mark == self.mark {
				// We have already visited this object so we return here.
				return;
			}

			trace!("Visiting {:?}", pointer.0);
			// we toggle the mark to say that we have visited/visiting this object.
			(*pointer.0).flags.set(ObjectFlags::MARK, self.mark);

			self.client.visit_edges(pointer, |value| {
				self.visit_pointer(value);
			})
		}
	}

	pub fn allocate(&mut self, size: usize) -> Result<ObjectPointer, AllocationError> {
		if size > ObjectSize::MAX as usize {
			return Err(AllocationError::ObjectTooBig);
		}

		// in bits
		let object_bytes = Self::align_size(size + OBJECT_HEADER);
		let used = unsafe { self.free.sub(self.data as usize) } as usize;
		if used + (object_bytes * 8) > self.size {
			return Err(AllocationError::OutOfHeap);
		}

		self.objects += 1;
		trace!(
			"Allocating {}+{} at {} {:?}.",
			size * 8,
			OBJECT_HEADER * 8,
			used,
			self.free
		);
		unsafe {
			// Get the part of the heap which is the object and set it up.
			let object = self.free;
			let value: *mut Object = transmute(object);
			(*value).flags = if self.mark {
				ObjectFlags::MARK
			} else {
				ObjectFlags::empty()
			};
			(*value).size = size as ObjectSize;

			// Increment the free pointer by the total object size. and align it to the 32bit alignment

			self.free = self.free.add(object_bytes);

			trace!("Allocated {:?}-{:?}", object, self.free);

			Ok(ObjectPointer(value))
		}
	}

	pub fn walk_alive(&self, mut visitor: impl FnMut(ObjectPointer)) {
		let mark = self.mark;
		unsafe {
			let mut current = self.data;
			while (current as usize) < (self.free as usize) {
				let object: *mut Object = transmute(current);
				let object_mark = (*object).flags.contains(ObjectFlags::MARK);

				if object_mark == mark {
					visitor(ObjectPointer(object));
				}
				// Increment by this objects size
				current = current.add(Self::align_size((*object).size as usize + OBJECT_HEADER));
			}
		}
	}

	pub(crate) fn align_size(bytes: usize) -> usize {
		let unaligned = bytes as *mut u8;
		bytes + unaligned.align_offset(OBJECT_ALIGNMENT)
	}
}

pub struct GCStatistics {
	objects_cleared: usize,
	objects_remaining: usize,
}

#[derive(Debug, Clone)]
pub enum AllocationError {
	OutOfHeap,
	ObjectTooBig,
}

pub trait GCClient {
	/// Requests the client to go through and show the GC all of the root objects which will later be used to traverse the entire graph.
	fn visit_roots(&self, visitor: impl FnMut(ObjectPointer));

	/// Requests the client to visit all of the objects inner references for marking.
	fn visit_edges(&self, object: ObjectPointer, visitor: impl FnMut(ObjectPointer));

	/// Requests the client to change all of the objects inner references to their new addresses.
	fn map_edges(&self, object: ObjectPointer, mapper: impl FnMut(ObjectPointer) -> ObjectPointer);
}

#[cfg(test)]
mod tests {
	use std::mem::size_of;
	use std::ptr::{addr_of_mut, read_unaligned, write_unaligned};

	use super::*;

	pub struct TestClient {
		roots: Vec<ObjectPointer>,
	}

	#[derive(Copy, Clone)]
	pub struct ClassRef(ObjectPointer);

	impl ClassRef {
		pub fn set(&mut self, field: usize, value: Option<ClassRef>) {
			ClassObject::visit(self.0, |fields| {
				fields[field] = value.map(|v| v.0).unwrap_or(ObjectPointer::NULL);
			});
		}

		pub fn get(&mut self, field: usize) -> Option<ClassRef> {
			ClassObject::visit(self.0, |fields| {
				let pointer = fields[field];
				if pointer.is_null() {
					None
				} else {
					Some(ClassRef(pointer))
				}
			})
		}
	}
	#[repr(packed, C)]
	pub struct ClassObject {
		fields: [ObjectPointer; 5],
	}
	impl ClassObject {
		pub fn visit<T>(
			pointer: ObjectPointer,
			visitor: impl FnOnce(&mut [ObjectPointer; 5]) -> T,
		) -> T {
			unsafe {
				println!("{}", OBJECT_HEADER);
				println!("pointer {:?}", pointer.0);
				let x = (pointer.0 as *mut u8).add(OBJECT_HEADER);
				let data = pointer.data();
				println!("+u8add {:?}", x);
				println!("+manual {:x?}", 140445499777088usize + 88);
				println!(
					"+code-diff {:?}",
					data as usize - pointer.0 as *mut u8 as usize
				);
				println!("+code {:?}", data);

				let object: *mut ClassObject = transmute(data);
				let addr = addr_of_mut!((*object).fields);
				let mut unaligned = read_unaligned(addr);
				let t = visitor(&mut unaligned);
				write_unaligned(addr, unaligned);

				t
			}
		}
	}

	impl GCClient for TestClient {
		fn visit_roots(&self, mut visitor: impl FnMut(ObjectPointer)) {
			for root in &self.roots {
				visitor(*root);
			}
		}

		fn visit_edges(&self, object: ObjectPointer, mut visitor: impl FnMut(ObjectPointer)) {
			ClassObject::visit(object, |fields| unsafe {
				for pointer in fields {
					visitor(*pointer);
				}
			});
		}

		fn map_edges(
			&self,
			object: ObjectPointer,
			mut mapper: impl FnMut(ObjectPointer) -> ObjectPointer,
		) {
			ClassObject::visit(object, |fields| unsafe {
				for pointer in fields {
					*pointer = mapper(*pointer);
				}
			});
		}
	}

	pub struct Gc {
		collector: GarbageCollector<TestClient>,
	}

	impl Gc {
		pub fn new() -> Gc {
			rvm_core::init();
			Gc {
				collector: GarbageCollector::new(TestClient { roots: vec![] }, 1024 * 1024 * 4),
			}
		}

		pub fn alloc(&mut self) -> ClassRef {
			let object = self.collector.allocate(size_of::<ClassObject>()).unwrap();
			ClassRef(object)
		}

		pub fn add_root(&mut self, obj: ClassRef) {
			self.collector.client.roots.push(obj.0);
		}

		pub fn gc(&mut self) -> GCStatistics {
			self.collector.gc()
		}
	}
	#[test]
	fn simple_alloc() {
		let mut gc = Gc::new();
		gc.alloc();
		assert_eq!(gc.collector.objects, 1);
		assert_eq!(gc.collector.free, unsafe {
			gc.collector
				.data
				.add(GarbageCollector::<TestClient>::align_size(
					OBJECT_HEADER + size_of::<ClassObject>(),
				))
		});
		gc.gc();
		assert_eq!(gc.collector.objects, 0);
		assert_eq!(gc.collector.free, gc.collector.data);
	}

	#[test]
	fn dual_alloc() {
		let mut gc = Gc::new();
		gc.alloc();
		gc.alloc();
		assert_eq!(gc.collector.objects, 2);

		let unaligned_size = OBJECT_HEADER + size_of::<ClassObject>();
		let aligned_size =
			unaligned_size + (unaligned_size as *mut u8).align_offset(OBJECT_ALIGNMENT);
		assert_eq!(gc.collector.free, unsafe {
			gc.collector.data.add(aligned_size * 2)
		});
		gc.gc();
		assert_eq!(gc.collector.objects, 0);
		assert_eq!(gc.collector.free, gc.collector.data);
	}

	#[test]
	fn ensure_root_safe() {
		let mut gc = Gc::new();
		let object = gc.alloc();
		gc.add_root(object);
		let stats = gc.gc();
		assert_eq!(stats.objects_cleared, 0);
		assert_eq!(stats.objects_remaining, 1);
		assert_eq!(gc.collector.objects, 1);
		assert_ne!(gc.collector.free, gc.collector.data);
	}
	#[test]
	fn ensure_children_safe() {
		let mut gc = Gc::new();
		let mut parent = gc.alloc();
		let child1 = gc.alloc();
		let child2 = gc.alloc();
		gc.add_root(parent);
		parent.set(0, Some(child1));
		parent.set(1, Some(child2));

		let stats = gc.gc();
		assert_eq!(stats.objects_cleared, 0);
		assert_eq!(stats.objects_remaining, 3);
		assert_ne!(gc.collector.free, gc.collector.data);

		let mut parent = ClassRef(gc.collector.client.roots[0]);
		assert!(parent.get(0).is_some());
		assert!(parent.get(1).is_some());
		assert!(parent.get(2).is_none());

		parent.set(0, None);
		let stats = gc.gc();
		assert_eq!(stats.objects_remaining, 2);
		assert_eq!(stats.objects_cleared, 1);
		assert_ne!(gc.collector.free, gc.collector.data);

		let mut parent = ClassRef(gc.collector.client.roots[0]);
		assert!(parent.get(0).is_none());
		assert!(parent.get(1).is_some());
		assert!(parent.get(2).is_none());
	}

	#[test]
	fn cyclic() {
		let mut gc = Gc::new();
		let mut parent = gc.alloc();
		let mut child1 = gc.alloc();
		let mut child2 = gc.alloc();
		gc.add_root(parent);
		parent.set(0, Some(child1));
		parent.set(1, Some(child2));

		child1.set(0, Some(child2));
		child2.set(0, Some(child1));

		let stats = gc.gc();
		assert_eq!(stats.objects_cleared, 0);
		assert_eq!(stats.objects_remaining, 3);
		assert_ne!(gc.collector.free, gc.collector.data);

		let mut parent = ClassRef(gc.collector.client.roots[0]);
		assert!(parent.get(0).is_some());
		assert!(parent.get(1).is_some());
		assert!(parent.get(2).is_none());

		parent.set(0, None);
		let stats = gc.gc();
		assert_eq!(stats.objects_remaining, 3);
		assert_eq!(stats.objects_cleared, 0);
		assert_ne!(gc.collector.free, gc.collector.data);

		let mut parent = ClassRef(gc.collector.client.roots[0]);
		assert!(parent.get(0).is_none());
		assert!(parent.get(1).is_some());
		assert!(parent.get(2).is_none());

		parent.set(1, None);
		let stats = gc.gc();
		assert_eq!(stats.objects_remaining, 1);
		assert_eq!(stats.objects_cleared, 2);
		assert_ne!(gc.collector.free, gc.collector.data);

		let mut parent = ClassRef(gc.collector.client.roots[0]);
		assert!(parent.get(0).is_none());
		assert!(parent.get(1).is_none());
		assert!(parent.get(2).is_none());
	}
}
