use parking_lot::Mutex;
use std::alloc::{alloc_zeroed, dealloc, Layout};
use std::collections::HashSet;
use thiserror::Error;
use tracing::{debug, trace};

use crate::{
	new_sweeper, GcHeader, GcMarker, GcRef, GcSweeper, GcSweeperHandle, GcUser, ObjectFlags,
	ObjectSize, ALIGNMENT, ALIGNMENT_BITS,
};
use rvm_core::{Kind, PrimitiveType};

pub const OBJECT_ALIGNMENT: usize = 8;

pub struct GarbageCollector<U: GcUser> {
	inner: Mutex<InnerGarbageCollector<U>>,
}

unsafe impl<U: GcUser> Sync for GarbageCollector<U> {}

unsafe impl<U: GcUser> Send for GarbageCollector<U> {}

impl<U: GcUser> GarbageCollector<U> {
	pub fn new(size: usize) -> Self {
		let layout = Layout::from_size_align(size, ALIGNMENT_BITS).unwrap();
		let data = unsafe { alloc_zeroed(layout) };

		assert!(data.is_aligned_to(ALIGNMENT));

		Self {
			inner: Mutex::new(InnerGarbageCollector {
				handles: vec![],
				frozen: HashSet::new(),
				mark: false,
				size,
				objects: 0,
				layout,
				free: data,
				data,
			}),
		}
	}

	pub fn new_sweeper(&self) -> GcSweeper {
		self.inner.lock().new_sweeper()
	}

	pub fn add_frozen(&self, reference: GcRef<U>) {
		self.inner.lock().add_frozen(reference)
	}

	pub fn remove_frozen(&self, reference: GcRef<U>) {
		self.inner.lock().remove_frozen(reference)
	}

	pub fn gc(&self) -> GCStatistics {
		self.inner.lock().gc()
	}

	pub fn used(&self) -> usize {
		self.inner.lock().used()
	}

	pub fn alloc_raw(
		&self,
		data_size: usize,
		header: U::Header,
	) -> Result<GcRef<U>, AllocationError> {
		self.inner.lock().allocate(data_size, header)
	}
}

pub struct InnerGarbageCollector<U: GcUser> {
	handles: Vec<GcSweeperHandle>,
	frozen: HashSet<GcRef<U>>,
	mark: bool,
	size: usize,
	objects: usize,
	layout: Layout,
	/// This is the pointer to the end of the used data
	free: *mut u8,
	/// This is the start of the heap
	data: *mut u8,
}

impl<U: GcUser> InnerGarbageCollector<U> {
	pub fn new_sweeper(&mut self) -> GcSweeper {
		let (handle, sweeper) = new_sweeper();
		self.handles.push(handle);
		sweeper
	}

	pub fn add_frozen(&mut self, reference: GcRef<U>) {
		if !self.frozen.insert(reference) {
			panic!("Double insertion!");
		}
	}

	pub fn remove_frozen(&mut self, reference: GcRef<U>) {
		if !self.frozen.remove(&reference) {
			panic!("Removed a reference which was not frozen. (did you double free?)")
		}
	}

	pub fn allocate(
		&mut self,
		data_size: usize,
		header: U::Header,
	) -> Result<GcRef<U>, AllocationError> {
		if data_size > ObjectSize::MAX as usize {
			return Err(AllocationError::ObjectTooBig);
		}
		assert!(self.free.is_aligned_to(ALIGNMENT));

		// in bits
		let total_size = GcRef::<U>::calc_total_size(data_size);
		let used = self.used();
		let after_bits = used + total_size;

		if after_bits >= self.size {
			return Err(AllocationError::OutOfHeap);
		}

		self.objects += 1;
		trace!(
			"Allocating {}/{} {}+{} at {} {:?}.",
			after_bits,
			self.size,
			data_size * 8,
			GcHeader::<U>::SIZE * 8,
			used,
			self.free
		);

		unsafe {
			// Get the part of the heap which is the object and set it up.
			let object = self.free;
			assert!(object.is_aligned_to(ALIGNMENT));

			let gc_ref = GcRef::create_at(
				object,
				GcHeader::<U>::new(
					if self.mark {
						ObjectFlags::MARK
					} else {
						ObjectFlags::empty()
					},
					total_size,
					header,
				)
				.ok_or(AllocationError::ObjectTooBig)?,
			);

			// Increment the free pointer by the total object size. and align it to the 32bit alignment
			self.free = self.free.add(gc_ref.total_size());
			assert!(self.free.is_aligned_to(ALIGNMENT));

			trace!("Allocated {:?}-{:?}", object, self.free);
			Ok(gc_ref)
		}
	}

	pub(super) fn gc(&mut self) -> GCStatistics {
		debug!("Starting garbage collection");

		// Stops all threads
		debug!("Stopping threads");
		for handle in &self.handles {
			let new_mark = !self.mark;
			handle.start(new_mark);
		}

		self.mark = !self.mark;

		// Makes all threads start marking
		debug!("Marking threads");
		for handle in &self.handles {
			handle.start_marking();
		}
		for reference in &self.frozen {
			GcMarker { mark: self.mark }.mark(*reference)
		}

		//use std::fmt::Write;
		//let mut build = String::new();
		//let mut gc_mark = self.mark;
		//self.walk(|mark, reference| {
		//	if mark == gc_mark {
		//		writeln!(&mut build, "x{} [color=green]", reference.0 as usize,).unwrap();
		//	} else {
		//		writeln!(&mut build, "x{} [color=gray]", reference.0 as usize,).unwrap();
		//	}
		//	reference.visit_refs(|r| {
		//		if r.is_null() {
		//			return;
		//		}
		//		writeln!(&mut build, "x{} -> x{}", reference.0 as usize, r.0 as usize).unwrap();
		//	});
		//});

		//// Visit all of the objects, and mark them as visitable.
		//roots.mark_roots(GcMarker { mark: self.mark });

		debug!("Calculating targets");
		// Go through all objects, and find the location where the object will soon be moved to,
		// we store this in the forward field in the object so we can move references in step 3.
		let mut new_free_ptr = self.data;
		let mut alive_objects = 0;
		self.walk_alive(|mut pointer| unsafe {
			//writeln!(&mut build, "root -> x{}", pointer.0 as usize).unwrap();
			alive_objects += 1;

			// Set the forward field to the soon to be the new object location.
			pointer.set_forward(new_free_ptr);

			trace!(
				"Object {:?} next is {:?} ",
				pointer.data_ptr(),
				new_free_ptr
			);

			// Increment the free pointer by the size of the object
			let object_size = pointer.total_size();
			new_free_ptr = new_free_ptr.add(object_size);
		});

		//write(format!("./out{}.txt", self.objects), build).unwrap();

		debug!("Moving references");
		// Move frozen slots to the new locations
		let mut new_frozen = HashSet::new();
		for reference in &self.frozen {
			unsafe {
				assert!(new_frozen.insert(reference.forward()));
			}
		}
		self.frozen = new_frozen;

		// This sets the roots to the new references
		for handle in &self.handles {
			handle.move_roots();
		}

		// This goes to the ref contents and makes sure that their children are pointing to the new references
		self.walk_alive(|pointer| {
			trace!("Updating {pointer:?}");
			pointer.map_refs(|r| unsafe { r.forward() });
		});

		debug!("Dropping data");
		self.walk_marked_for_deletion(|pointer| unsafe {
			U::drop_ref(pointer);
		});

		debug!("Moving data");
		// This goes through all of the live objects, and moves them to their new location, (which is always behind).
		// Go through the live objects, and move them to their new locations.
		self.walk_alive(|mut pointer| unsafe {
			pointer.move_forward();
		});

		debug!("Finalizing");
		// Set the free pointer to the new limit.
		self.free = new_free_ptr;
		let statistics = GCStatistics {
			objects_cleared: self.objects - alive_objects,
			objects_remaining: alive_objects,
		};
		self.objects = alive_objects;

		// Release all threads
		for handle in &self.handles {
			handle.continue_execution();
		}

		statistics
	}

	pub fn used(&self) -> usize {
		(self.free as usize) - (self.data as usize)
	}
	pub fn walk(&self, mut visitor: impl FnMut(bool, GcRef<U>)) {
		unsafe {
			let mut current = self.data;
			while (current as usize) < (self.free as usize) {
				let gc_ref = GcRef::<U>::from_ptr(current).unwrap();
				let object_mark = gc_ref.header().flags.contains(ObjectFlags::MARK);

				visitor(object_mark, gc_ref);
				// Increment by this objects size
				current = current.add(gc_ref.total_size());
				debug_assert!(current.is_aligned_to(ALIGNMENT));
			}
		}
	}
	pub fn walk_marked_for_deletion(&self, mut visitor: impl FnMut(GcRef<U>)) {
		let mark = self.mark;
		self.walk(|object_mark, reference| {
			if object_mark != mark {
				visitor(reference);
			}
		});
	}
	pub fn walk_alive(&self, mut visitor: impl FnMut(GcRef<U>)) {
		let mark = self.mark;
		self.walk(|object_mark, reference| {
			if object_mark == mark {
				visitor(reference);
			}
		});
	}
}

impl<U: GcUser> Drop for InnerGarbageCollector<U> {
	fn drop(&mut self) {
		self.walk_alive(|value| unsafe {
			U::drop_ref(value);
		});
		unsafe {
			dealloc(self.data, self.layout);
		}
	}
}

pub struct GCStatistics {
	pub objects_cleared: usize,
	pub objects_remaining: usize,
}

#[derive(Error, Debug, Clone)]
pub enum AllocationError {
	#[error("Out of heap space")]
	OutOfHeap,
	#[error("Object is too big to be allocated")]
	ObjectTooBig,
}

#[cfg(test)]
mod tests {
	use rvm_core::align_size;

	#[test]
	fn align_size_test() {
		assert_eq!(align_size(3, 1), 3);
		assert_eq!(align_size(3, 2), 4);
		assert_eq!(align_size(3, 3), 3);
		assert_eq!(align_size(3, 4), 4);
		assert_eq!(align_size(3, 8), 8);
	}
	//use std::mem::size_of;
	// 	use std::sync::Arc;
	//
	// 	use rvm_core::{FieldAccessFlags, Kind, ObjectType, PrimitiveType};
	// 	use rvm_object::{
	// 		Class, ClassLoader, ClassMethodManager, DynValue, FieldData, ObjectFieldLayout,
	// 	};
	// 	use rvm_reader::ConstantPool;
	//
	// 	use crate::{AnyValue, ClassLoader};
	//
	// 	use super::*;
	//
	// 	#[test]
	// 	fn asfd() {
	// 		assert_eq!(align_size(0), 0);
	// 		assert_eq!(align_size(1), 4);
	// 		assert_eq!(align_size(2), 4);
	// 		assert_eq!(align_size(3), 4);
	// 		assert_eq!(align_size(4), 4);
	// 		assert_eq!(align_size(5), 8);
	// 	}
	//
	// 	//	pub struct TestClient {
	// 	// 		roots: Vec<Reference>,
	// 	// 	}
	// 	//
	// 	// 	#[derive(Copy, Clone)]
	// 	// 	pub struct ClassRef(Reference);
	// 	//
	// 	// 	impl ClassRef {
	// 	// 		pub fn set(&mut self, field: usize, value: Option<ClassRef>) {
	// 	// 			ClassObject::visit(self.0, |fields| {
	// 	// 				fields[field] = value.map(|v| v.0).unwrap_or(Reference::NULL);
	// 	// 			});
	// 	// 		}
	// 	//
	// 	// 		pub fn get(&mut self, field: usize) -> Option<ClassRef> {
	// 	// 			ClassObject::visit(self.0, |fields| {
	// 	// 				let pointer = fields[field];
	// 	// 				if pointer.is_null() {
	// 	// 					None
	// 	// 				} else {
	// 	// 					Some(ClassRef(pointer))
	// 	// 				}
	// 	// 			})
	// 	// 		}
	// 	// 	}
	// 	// 	#[repr(packed, C)]
	// 	// 	pub struct ClassObject {
	// 	// 		fields: [Reference; 5],
	// 	// 	}
	// 	// 	impl ClassObject {
	// 	// 		pub fn visit<T>(pointer: Reference, visitor: impl FnOnce(&mut [Reference; 5]) -> T) -> T {
	// 	// 			unsafe {
	// 	// 				println!("{}", OBJECT_HEADER);
	// 	// 				println!("pointer {:?}", pointer.0);
	// 	// 				let x = (pointer.0 as *mut u8).add(OBJECT_HEADER);
	// 	// 				let data = pointer.data();
	// 	// 				println!("+u8add {:?}", x);
	// 	// 				println!("+manual {:x?}", 140445499777088usize + 88);
	// 	// 				println!(
	// 	// 					"+code-diff {:?}",
	// 	// 					data as usize - pointer.0 as *mut u8 as usize
	// 	// 				);
	// 	// 				println!("+code {:?}", data);
	// 	//
	// 	// 				let object: *mut ClassObject = transmute(data);
	// 	// 				let addr = addr_of_mut!((*object).fields);
	// 	// 				let mut unaligned = read_unaligned(addr);
	// 	// 				let t = visitor(&mut unaligned);
	// 	// 				write_unaligned(addr, unaligned);
	// 	//
	// 	// 				t
	// 	// 			}
	// 	// 		}
	// 	// 	}
	// 	//
	//
	// 	//pub struct Gc {
	// 	// 		collector: GarbageCollector,
	// 	// 	}
	// 	//
	// 	// 	impl Gc {
	// 	// 		pub fn new() -> Gc {
	// 	// 			rvm_core::init();
	// 	// 			Gc {
	// 	// 				collector: GarbageCollector::new(1024 * 1024 * 4),
	// 	// 			}
	// 	// 		}
	// 	//
	// 	// 		pub fn alloc(&mut self) -> ClassRef {
	// 	// 			let object = self.collector.allocate(size_of::<ClassObject>()).unwrap();
	// 	// 			ClassRef(object)
	// 	// 		}
	// 	//
	// 	// 		pub fn add_root(&mut self, obj: ClassRef) {
	// 	// 			self.collector.client.roots.push(obj.0);
	// 	// 		}
	// 	//
	// 	// 		pub fn gc(&mut self) -> GCStatistics {
	// 	// 			self.collector.gc()
	// 	// 		}
	// 	// 	}
	//
	// 	#[derive(Default)]
	// 	pub struct TestRoots {
	// 		roots: Vec<Reference>,
	// 	}
	//
	// 	impl RootProvider for TestRoots {
	// 		fn mark_roots(&mut self, mut marker: GcMarker) {
	// 			for x in &self.roots {
	// 				marker.mark(*x);
	// 			}
	// 		}
	//
	// 		fn remap_roots(&mut self, mut mapper: impl FnMut(Reference) -> Reference) {
	// 			for x in &mut self.roots {
	// 				*x = mapper(*x);
	// 			}
	// 		}
	//
	// 		fn sweeper(&mut self) -> &mut GcSweeper {
	// 			todo!()
	// 		}
	// 	}
	//
	// 	fn create_class(loader: &mut ClassLoader, name: &str, fields: &[(&str, Kind)]) -> Id<Class> {
	// 		let fields: Vec<FieldData> = fields
	// 			.iter()
	// 			.map(|(name, kind)| FieldData {
	// 				name: name.to_string(),
	// 				ty: match kind {
	// 					Kind::Boolean => PrimitiveType::Boolean.into(),
	// 					Kind::Byte => PrimitiveType::Byte.into(),
	// 					Kind::Short => PrimitiveType::Short.into(),
	// 					Kind::Int => PrimitiveType::Int.into(),
	// 					Kind::Long => PrimitiveType::Long.into(),
	// 					Kind::Char => PrimitiveType::Char.into(),
	// 					Kind::Float => PrimitiveType::Float.into(),
	// 					Kind::Double => PrimitiveType::Double.into(),
	// 					Kind::Reference => ObjectType("HiBabyGirl".to_string()).into(),
	// 				},
	// 				flags: FieldAccessFlags::empty(),
	// 			})
	// 			.collect();
	//
	// 		let layout = ObjectFieldLayout::new(&fields, false);
	//
	// 		loader.define(Class::Object(InstanceClass {
	// 			ty: name.to_string().into(),
	// 			fields: layout,
	// 			cp: Arc::new(ConstantPool::new(vec![])),
	// 			static_fields: ObjectFieldLayout::new(&[], true),
	// 			methods: ClassMethodManager::empty(),
	// 		}))
	// 	}
	//
	// 	#[test]
	// 	fn test_manipulation() {
	// 		unsafe {
	// 			let obj = ref_to_header(Reference(1000 as *mut u8)) as usize;
	// 			assert_eq!(obj, 1000 - OBJECT_HEADER);
	// 			assert_eq!(header_to_ref(obj as *mut GcHeader).0 as usize, 1000);
	// 		}
	// 	}
	//
	// 	#[test]
	// 	fn root_objects() {
	// 		rvm_core::init();
	// 		let mut gc = GarbageCollector::new(1024 * 1024);
	// 		let mut loader = ClassLoader::new();
	// 		let id = create_class(&mut loader, "hi", &[("field", Kind::Int)]);
	//
	// 		let arc = loader.get(id);
	// 		let object_class = arc.as_instance().unwrap();
	// 		let field_id = object_class.fields.get_id("field").unwrap();
	// 		let mut roots = TestRoots::default();
	// 		unsafe {
	// 			for i in 0..2 {
	// 				let object = gc.allocate_instance(id, object_class).unwrap();
	// 				roots.roots.push(*object);
	//
	// 				let resolved_object = object.resolve(object_class);
	// 				assert_eq!(resolved_object.get_dyn(field_id), AnyValue::Int(0));
	// 				resolved_object.put_dyn(field_id, AnyValue::Int(69));
	// 				assert_eq!(resolved_object.get_dyn(field_id), AnyValue::Int(69));
	//
	// 				assert_eq!(object.class(), id);
	// 			}
	// 		}
	//
	// 		assert_eq!(gc.objects, 2);
	// 		assert_eq!(gc.free, unsafe {
	// 			gc.data.add(
	// 				align_size(OBJECT_HEADER + AnyInstance::FULL_HEADER_SIZE + size_of::<u32>()) * 2,
	// 			)
	// 		});
	//
	// 		gc.gc();
	// 		assert_eq!(gc.objects, 2);
	// 		assert_eq!(gc.free, unsafe {
	// 			gc.data.add(
	// 				align_size(OBJECT_HEADER + AnyInstance::FULL_HEADER_SIZE + size_of::<u32>()) * 2,
	// 			)
	// 		});
	//
	// 		for reference in &roots.roots {
	// 			let object = Object::new(*reference);
	// 			let class = object.as_class().unwrap();
	// 			assert_eq!(class.class(), id);
	// 			let class = class.resolve(object_class);
	// 			assert_eq!(class.get_dyn(field_id), AnyValue::Int(69));
	// 		}
	//
	// 		roots.roots.pop();
	//
	// 		gc.gc();
	// 		assert_eq!(gc.objects, 1);
	// 		assert_eq!(gc.free, unsafe {
	// 			gc.data.add(align_size(
	// 				OBJECT_HEADER + AnyInstance::FULL_HEADER_SIZE + size_of::<u32>(),
	// 			))
	// 		});
	//
	// 		for reference in &roots.roots {
	// 			let object = Object::new(*reference);
	// 			let class = object.as_class().unwrap();
	// 			assert_eq!(class.class(), id);
	// 			let class = class.resolve(object_class);
	// 			assert_eq!(class.get_dyn(field_id), AnyValue::Int(69));
	// 		}
	// 		roots.roots.pop();
	//
	// 		gc.gc();
	// 		assert_eq!(gc.objects, 0);
	// 		assert_eq!(gc.free, gc.data);
	// 	}
	//
	// 	#[test]
	// 	fn direct_child() {
	// 		rvm_core::init();
	// 		let mut gc = GarbageCollector::new(1024 * 1024);
	// 		let mut loader = ClassLoader::new();
	// 		let mut roots = TestRoots::default();
	//
	// 		let parent_id = create_class(
	// 			&mut loader,
	// 			"Parent",
	// 			&[("intimacy", Kind::Int), ("child", Kind::Reference)],
	// 		);
	// 		let child_id = create_class(&mut loader, "Child", &[("iq", Kind::Float)]);
	//
	// 		let parent_arc = loader.get(parent_id);
	// 		let parent_class = parent_arc.as_instance().unwrap();
	// 		let parent_intimacy = parent_class.fields.get_id("intimacy").unwrap();
	// 		let parent_child = parent_class.fields.get_id("child").unwrap();
	//
	// 		let child_arc = loader.get(child_id);
	// 		let child_class = child_arc.as_instance().unwrap();
	// 		let child_iq = child_class.fields.get_id("iq").unwrap();
	//
	// 		unsafe {
	// 			let parent = gc.allocate_instance(parent_id, parent_class).unwrap();
	// 			let child = gc.allocate_instance(child_id, child_class).unwrap();
	//
	// 			roots.roots.push(*parent);
	//
	// 			let parent = parent.resolve(parent_class);
	// 			parent.put_dyn(parent_child, AnyValue::Reference(*child));
	// 			parent.put_dyn(parent_intimacy, AnyValue::Int(6969));
	//
	// 			let child = child.resolve(child_class);
	// 			child.put_dyn(child_iq, AnyValue::Float(420.0));
	// 		}
	//
	// 		let stats = gc.gc();
	// 		assert_eq!(stats.objects_cleared, 0);
	// 		assert_eq!(stats.objects_remaining, 2);
	// 		assert_ne!(gc.free, gc.data);
	//
	// 		let parent = roots.roots[0];
	// 		let parent_obj = Object::new(parent);
	// 		let parent = parent_obj.as_class().unwrap();
	// 		let parent = parent.resolve(parent_class);
	//
	// 		assert_eq!(parent.get_dyn(parent_intimacy), AnyValue::Int(6969));
	// 		let value = parent.get_dyn(parent_child);
	// 		let child_ref = match value {
	// 			AnyValue::Reference(point) => point,
	// 			_ => panic!("wrong type"),
	// 		};
	// 		let child_obj = Object::new(child_ref);
	// 		let child = child_obj.as_class().unwrap();
	// 		let child = child.resolve(child_class);
	// 		assert_eq!(child.get_dyn(child_iq), AnyValue::Float(420.0));
	//
	// 		parent.put_dyn(parent_child, AnyValue::Reference(Reference::NULL));
	//
	// 		let stats = gc.gc();
	// 		assert_eq!(stats.objects_remaining, 1);
	// 		assert_eq!(stats.objects_cleared, 1);
	// 		assert_ne!(gc.free, gc.data);
	// 	}
	// 	//
	// 	// 	#[test]
	// 	// 	fn cyclic() {
	// 	// 		let mut gc = Gc::new();
	// 	// 		let mut parent = gc.alloc();
	// 	// 		let mut child1 = gc.alloc();
	// 	// 		let mut child2 = gc.alloc();
	// 	// 		gc.add_root(parent);
	// 	// 		parent.set(0, Some(child1));
	// 	// 		parent.set(1, Some(child2));
	// 	//
	// 	// 		child1.set(0, Some(child2));
	// 	// 		child2.set(0, Some(child1));
	// 	//
	// 	// 		let stats = gc.gc();
	// 	// 		assert_eq!(stats.objects_cleared, 0);
	// 	// 		assert_eq!(stats.objects_remaining, 3);
	// 	// 		assert_ne!(gc.collector.free, gc.collector.data);
	// 	//
	// 	// 		let mut parent = ClassRef(gc.collector.client.roots[0]);
	// 	// 		assert!(parent.get(0).is_some());
	// 	// 		assert!(parent.get(1).is_some());
	// 	// 		assert!(parent.get(2).is_none());
	// 	//
	// 	// 		parent.set(0, None);
	// 	// 		let stats = gc.gc();
	// 	// 		assert_eq!(stats.objects_remaining, 3);
	// 	// 		assert_eq!(stats.objects_cleared, 0);
	// 	// 		assert_ne!(gc.collector.free, gc.collector.data);
	// 	//
	// 	// 		let mut parent = ClassRef(gc.collector.client.roots[0]);
	// 	// 		assert!(parent.get(0).is_none());
	// 	// 		assert!(parent.get(1).is_some());
	// 	// 		assert!(parent.get(2).is_none());
	// 	//
	// 	// 		parent.set(1, None);
	// 	// 		let stats = gc.gc();
	// 	// 		assert_eq!(stats.objects_remaining, 1);
	// 	// 		assert_eq!(stats.objects_cleared, 2);
	// 	// 		assert_ne!(gc.collector.free, gc.collector.data);
	// 	//
	// 	// 		let mut parent = ClassRef(gc.collector.client.roots[0]);
	// 	// 		assert!(parent.get(0).is_none());
	// 	// 		assert!(parent.get(1).is_none());
	// 	// 		assert!(parent.get(2).is_none());
	// 	// 	}
}
