use crate::{align_size, GcHeader, GcUser};
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::mem;
use std::ptr::{copy, null_mut, slice_from_raw_parts};
use std::slice::from_raw_parts;
// This pointer points to the start of the data (NOT THE START OF THE REFERENCE OBJECT WHICH CONTAINS THE HEADER!)

// HEADER
// HEADER
// DATA <- data_ptr
// DATA
// DATA
#[repr(transparent)]
pub struct GcRef<U: GcUser> {
	data_ptr: *mut u8,
	_u: PhantomData<fn() -> U>,
}
pub const ALIGNMENT: usize = 8;
pub const ALIGNMENT_BITS: usize = ALIGNMENT * 8;

impl<U: GcUser> GcRef<U> {
	pub const NULL: Self = GcRef {
		data_ptr: null_mut(),
		_u: PhantomData,
	};

	/// Creates a GcRef from a pointer.
	///
	///
	/// # Arguments
	///
	/// * `head`: The GC pointer.
	///
	/// returns: GcRef<U> (may return null)
	///
	/// # Safety
	/// The pointer needs to be pointing to a valid location to the garbage collector.
	pub const unsafe fn from_ptr(head: *mut u8) -> Result<Self, Self> {
		if mem::transmute::<_, usize>(head.cast::<()>()) == 0 {
			return Err(GcRef::NULL);
		}
		Ok(Self {
			data_ptr: head.add(GcHeader::<U>::SIZE),
			_u: PhantomData,
		})
	}

	pub unsafe fn create_at(head: *mut u8, gc_header: GcHeader<U>) -> Self {
		let mut gc_ref = Self::from_ptr(head).expect("Tried to create a null reference!");
		*gc_ref.header_mut() = gc_header;
		gc_ref
	}

	pub fn header_mut(&mut self) -> &'_ mut GcHeader<U> {
		unsafe {
			let data = self.head_ptr().cast();
			&mut *data
		}
	}

	pub fn header(&self) -> &'_ GcHeader<U> {
		unsafe {
			let data = self.head_ptr().cast() as *const GcHeader<U>;
			&*data
		}
	}

	pub fn data_size(&self) -> usize {
		let data_size = self.header().data_size();
		debug_assert!(data_size == align_size(data_size, ALIGNMENT));
		data_size
	}

	pub fn calc_total_size(data_size: usize) -> usize {
		let aligned_data_size = align_size(data_size, ALIGNMENT);
		let aligned_header_size = GcHeader::<U>::SIZE;
		debug_assert!(aligned_header_size == align_size(aligned_header_size, ALIGNMENT));
		let total_size = aligned_data_size + aligned_header_size;
		debug_assert!(total_size == align_size(total_size, ALIGNMENT));
		total_size
	}
	pub fn total_size(&self) -> usize {
		let data_size = self.data_size();
		let total_size = data_size + GcHeader::<U>::SIZE;
		debug_assert!(total_size == align_size(total_size, ALIGNMENT));
		total_size
	}

	// Next location is the start of the reference (NOT THE DATA)
	pub(crate) unsafe fn set_forward(&mut self, next_location: *mut u8) {
		let header = self.header_mut();
		header.forward = next_location;
	}

	// Creates a new ref at the forward location
	pub(crate) unsafe fn forward(&self) -> Self {
		if self.is_null() {
			return Self::NULL;
		}
		let header = self.header();
		GcRef::from_ptr(header.forward).unwrap()
	}

	pub(crate) unsafe fn move_forward(&mut self) {
		self.ensure_not_null();
		let object_size = self.total_size();
		let header = self.header();

		let forward_location = header.forward;
		let current_location = self.head_ptr();
		copy(current_location, forward_location, object_size);
	}

	pub fn head_ptr(&self) -> *mut u8 {
		self.ensure_not_null();
		unsafe { self.data_ptr.sub(GcHeader::<U>::SIZE) }
	}

	pub fn data_ptr(&self) -> *mut u8 {
		self.ensure_not_null();
		self.data_ptr
	}

	pub fn data(&self) -> &[u8] {
		self.ensure_not_null();
		let i = self.data_size();
		unsafe { from_raw_parts(self.data_ptr, i) }
	}

	pub fn is_null(&self) -> bool {
		self.data_ptr.is_null()
	}

	pub fn visit_refs(&self, visitor: impl FnMut(GcRef<U>)) {
		self.ensure_not_null();
		U::visit_refs(self, visitor)
	}

	pub fn map_refs(&self, mapper: impl FnMut(GcRef<U>) -> GcRef<U>) {
		self.ensure_not_null();
		U::map_refs(self, mapper)
	}

	fn ensure_not_null(&self) {
		if self.data_ptr.is_null() {
			panic!("Pointer is null");
		}
	}
}

unsafe impl<U: GcUser> Send for GcRef<U> {}

unsafe impl<U: GcUser> Sync for GcRef<U> {}
impl<U: GcUser> Hash for GcRef<U> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.data_ptr.hash(state)
	}
}
impl<U: GcUser> Eq for GcRef<U> {}
impl<U: GcUser> PartialEq for GcRef<U> {
	fn eq(&self, other: &Self) -> bool {
		self.data_ptr.eq(&other.data_ptr)
	}
}
impl<U: GcUser> Copy for GcRef<U> {}
impl<U: GcUser> Clone for GcRef<U> {
	fn clone(&self) -> Self {
		*self
	}
}
impl<U: GcUser> Debug for GcRef<U> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self.data_ptr)
	}
}
