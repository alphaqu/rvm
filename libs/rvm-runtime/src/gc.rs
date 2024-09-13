use crate::{ArrayRef, Class, InstanceClass, InstanceRef, Reference, ReferenceKind};
use rvm_core::{Id, Kind};
pub use rvm_gc::*;

pub type GcRef = rvm_gc::GcRef<JavaUser>;

pub struct GarbageCollector {
	gc: rvm_gc::GarbageCollector<JavaUser>,
}

impl GarbageCollector {
	pub fn new(size: usize) -> Self {
		Self {
			gc: rvm_gc::GarbageCollector::new(size),
		}
	}

	pub fn new_sweeper(&self) -> GcSweeper {
		self.gc.new_sweeper()
	}

	pub fn remove_sweeper(&self, sweeper: GcSweeper) {
		self.gc.remove_sweeper(sweeper);
	}

	pub fn add_frozen(&self, reference: Reference) {
		self.gc.add_frozen(*reference)
	}

	pub fn remove_frozen(&self, reference: Reference) {
		self.gc.remove_frozen(*reference)
	}

	pub fn gc(&self) -> GCStatistics {
		self.gc.gc()
	}

	pub fn used(&self) -> usize {
		self.gc.used()
	}
	pub fn alloc_static_instance(
		&self,
		class: &InstanceClass,
	) -> Result<InstanceRef, AllocationError> {
		let fields = &class.static_field_layout;
		let gc_ref = self.gc.alloc_raw(
			fields.fields_size as usize,
			JavaHeader::InstanceStatic(InstanceHeader {
				id: class.id,
				ref_fields: fields.reference_count,
			}),
		)?;

		Ok(InstanceRef::new(Reference::new(gc_ref)))
	}
	pub fn alloc_instance(&self, class: &InstanceClass) -> Result<InstanceRef, AllocationError> {
		let fields = &class.field_layout;
		let gc_ref = self.gc.alloc_raw(
			fields.fields_size as usize,
			JavaHeader::Instance(InstanceHeader {
				id: class.id,
				ref_fields: fields.reference_count,
			}),
		)?;

		Ok(InstanceRef::new(Reference::new(gc_ref)))
	}

	pub fn alloc_array(&self, component: &Class, length: u32) -> Result<ArrayRef, AllocationError> {
		let (component_id, kind) = match component {
			Class::Instance(class) => (Some(class.id), Kind::Reference),
			Class::Array(class) => (Some(class.id), Kind::Reference),
			Class::Primitive(ty) => (None, ty.kind()),
		};

		let gc_ref = self.gc.alloc_raw(
			length as usize * kind.size(),
			JavaHeader::Array(ArrayHeader {
				component_id,
				kind,
				length,
			}),
		)?;
		Ok(ArrayRef::new(Reference::new(gc_ref)))
	}
}

pub struct JavaUser {}

impl GcUser for JavaUser {
	type Header = JavaHeader;

	unsafe fn drop_ref(_: GcRef) {
		// This is unused because all of our values are not heap allocated
	}

	fn visit_refs(reference: &GcRef, mut visitor: impl FnMut(GcRef)) {
		Reference::new(*reference).visit_refs(|reference| {
			visitor(*reference);
		});
	}

	fn map_refs(reference: &GcRef, mut visitor: impl FnMut(GcRef) -> GcRef) {
		Reference::new(*reference).map_refs(|reference| Reference::new(visitor(*reference)));
	}
}

pub enum JavaHeader {
	Array(ArrayHeader),
	Instance(InstanceHeader),
	InstanceStatic(InstanceHeader),
}

impl JavaHeader {
	pub fn kind(&self) -> ReferenceKind {
		match self {
			JavaHeader::Array(_) => ReferenceKind::Array,
			JavaHeader::Instance(_) => ReferenceKind::Instance,
			JavaHeader::InstanceStatic(_) => ReferenceKind::Instance,
		}
	}
}
pub struct ArrayHeader {
	pub kind: Kind,
	pub component_id: Option<Id<Class>>,
	pub length: u32,
}
pub struct InstanceHeader {
	pub id: Id<Class>,
	pub ref_fields: u16,
}
