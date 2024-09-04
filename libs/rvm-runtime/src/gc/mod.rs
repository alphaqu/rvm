use ahash::HashSetExt;
use std::alloc::{alloc_zeroed, Layout};
use std::collections::HashSet;
use std::ptr::copy;
use thiserror::Error;
use tracing::{debug, trace};

pub use object::{GcHeader, ObjectFlags, ObjectSize, OBJECT_HEADER};
use rvm_core::{Id, Kind, PrimitiveType};

use crate::gc::sweep::{new_sweeper, GcSweeperHandle};
pub use crate::gc::sweep::{GcMarker, GcSweeper};
use crate::object::{AnyArray, Class, InstanceClass, InstanceReference, Reference};

mod object;
mod sweep;

pub const OBJECT_ALIGNMENT: usize = 8;

pub struct GarbageCollector {
	handles: Vec<GcSweeperHandle>,
	frozen: HashSet<Reference>,
	mark: bool,
	size: usize,
	objects: usize,
	free: *mut u8,
	data: *mut u8,
}

unsafe impl Sync for GarbageCollector {}

unsafe impl Send for GarbageCollector {}

pub trait RootProvider {
	fn mark_roots(&mut self, marker: GcMarker);

	fn remap_roots(&mut self, mapper: impl FnMut(Reference) -> Reference);

	fn sweeper(&mut self) -> &mut GcSweeper;
}

impl GarbageCollector {
	pub fn new(size: usize) -> GarbageCollector {
		let layout = Layout::from_size_align(size, 32).unwrap();
		let data = unsafe { alloc_zeroed(layout) };

		GarbageCollector {
			handles: vec![],
			frozen: HashSet::new(),
			mark: false,
			size,
			objects: 0,
			free: data,
			data,
		}
	}

	pub fn new_sweeper(&mut self) -> GcSweeper {
		let (handle, sweeper) = new_sweeper();
		self.handles.push(handle);
		sweeper
	}

	pub fn add_frozen(&mut self, reference: Reference) {
		if !self.frozen.insert(reference) {
			panic!("Double insertion!");
		}
	}

	pub fn remove_frozen(&mut self, reference: Reference) {
		if !self.frozen.remove(&reference) {
			panic!("Removed a reference which was not frozen. (did you double free?)")
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
		let mut new_free = self.data;
		let mut alive_objects = 0;
		self.walk_alive(|pointer| unsafe {
			//writeln!(&mut build, "root -> x{}", pointer.0 as usize).unwrap();
			alive_objects += 1;

			// Set the forward field to the soon to be the new object location.
			let obj = ref_to_header(pointer);
			(*obj).forward = new_free as usize;

			trace!("Object {:?} next is {:?} ", obj as *mut u8, new_free);

			// Increment the free pointer by the size of the object
			let object_size = (*obj).size as usize + OBJECT_HEADER;
			new_free = new_free.add(align_size(object_size));
		});
		//write(format!("./out{}.txt", self.objects), build).unwrap();

		debug!("Moving roots");
		// Update all of the object edges to the new object locations.
		// Moves all of the roots to their soon to be new location
		//roots.remap_roots(|r| unsafe { move_reference(r) });
		// Move all of the objects children to their new location
		{
			// Frozen
			let mut new_frozen = HashSet::new();
			for reference in &self.frozen {
				unsafe {
					assert!(new_frozen.insert(move_reference(*reference)));
				}
			}
			self.frozen = new_frozen;
		}
		for handle in &self.handles {
			handle.move_roots();
		}

		debug!("Moving references");
		self.walk_alive(|pointer| {
			trace!("Updating {pointer:?}");
			//// Go through all of the objects edges, and move them to the new child object location.
			pointer.map_refs(|r| unsafe { move_reference(r) });
		});

		debug!("Moving data");
		// Go through the live objects, and move them to their new locations.
		self.walk_alive(|pointer| unsafe {
			let obj = ref_to_header(pointer);
			let new_location = (*obj).forward;
			let size = align_size((*obj).size as usize + OBJECT_HEADER);
			copy(obj as *mut u8, new_location as *mut u8, size);
		});

		debug!("Finalizing");
		// Set the free pointer to the new limit.
		self.free = new_free;
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

	pub fn allocate_instance(
		&mut self,
		id: Id<Class>,
		class: &InstanceClass,
	) -> Result<InstanceReference, AllocationError> {
		unsafe {
			let reference = self.allocate(
				(class.fields.fields_size as usize) + InstanceReference::FULL_HEADER_SIZE,
			)?;
			Ok(InstanceReference::allocate(reference, id, class))
		}
	}

	pub fn allocate_ref_array(
		&mut self,
		component: Id<Class>,
		length: i32,
	) -> Result<AnyArray, AllocationError> {
		let size = AnyArray::size(Kind::Reference, length);
		unsafe {
			let reference = self.allocate(size)?;
			Ok(AnyArray::allocate_ref(reference, component, length))
		}
	}

	pub fn allocate_array(
		&mut self,
		kind: PrimitiveType,
		length: i32,
	) -> Result<AnyArray, AllocationError> {
		let size = AnyArray::size(kind.kind(), length);
		unsafe {
			let reference = self.allocate(size)?;
			Ok(AnyArray::allocate_primitive(reference, kind, length))
		}
	}

	/// # Safety
	/// The caller must set the reference kind mark to the type of reference it is.
	pub unsafe fn allocate(&mut self, size: usize) -> Result<Reference, AllocationError> {
		if size > ObjectSize::MAX as usize {
			return Err(AllocationError::ObjectTooBig);
		}

		// in bits
		let object_bytes = align_size(size + OBJECT_HEADER);
		let used = unsafe { self.free.sub(self.data as usize) } as usize;
		let after_bits = used + object_bytes;

		if after_bits >= self.size {
			return Err(AllocationError::OutOfHeap);
		}

		self.objects += 1;
		trace!(
			"Allocating {}/{} {}+{} at {} {:?}.",
			after_bits,
			self.size,
			size * 8,
			OBJECT_HEADER * 8,
			used,
			self.free
		);
		unsafe {
			// Get the part of the heap which is the object and set it up.
			let object = self.free;
			let value = object as *mut GcHeader;
			(*value).flags = if self.mark {
				ObjectFlags::MARK
			} else {
				ObjectFlags::empty()
			};
			(*value).size = size as ObjectSize;

			// Increment the free pointer by the total object size. and align it to the 32bit alignment

			self.free = self.free.add(object_bytes);

			trace!("Allocated {:?}-{:?}", object, self.free);
			Ok(header_to_ref(value))
		}
	}
	pub fn walk(&self, mut visitor: impl FnMut(bool, Reference)) {
		unsafe {
			let mut current = self.data;
			while (current as usize) < (self.free as usize) {
				let object = current as *mut GcHeader;
				let object_mark = (*object).flags.contains(ObjectFlags::MARK);

				visitor(object_mark, header_to_ref(object));
				// Increment by this objects size
				current = current.add(align_size((*object).size as usize + OBJECT_HEADER));
			}
		}
	}
	pub fn walk_alive(&self, mut visitor: impl FnMut(Reference)) {
		let mark = self.mark;
		unsafe {
			let mut current = self.data;
			while (current as usize) < (self.free as usize) {
				let object = current as *mut GcHeader;
				let object_mark = (*object).flags.contains(ObjectFlags::MARK);

				if object_mark == mark {
					visitor(header_to_ref(object));
				}
				// Increment by this objects size
				current = current.add(align_size((*object).size as usize + OBJECT_HEADER));
			}
		}
	}
}

unsafe fn move_reference(reference: Reference) -> Reference {
	if reference.is_null() {
		return Reference::NULL;
	}

	let obj = ref_to_header(reference);
	let new = (*obj).forward;
	trace!("Moving {:?} to {:?}", obj as *mut u8, new as *mut u8);
	header_to_ref(new as *mut GcHeader)
}

unsafe fn ref_to_header(reference: Reference) -> *mut GcHeader {
	let data = reference.0.sub(OBJECT_HEADER);
	data as *mut GcHeader
}

unsafe fn header_to_ref(pointer: *mut GcHeader) -> Reference {
	Reference((pointer as *mut u8).add(OBJECT_HEADER))
}

fn align_size(bytes: usize) -> usize {
	let unaligned = bytes as *mut u8;
	bytes + unaligned.align_offset(OBJECT_ALIGNMENT)
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
