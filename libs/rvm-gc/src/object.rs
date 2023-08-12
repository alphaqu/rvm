use std::mem::size_of;
use std::ptr::null_mut;

use bitflags::bitflags;

pub type ObjectSize = u16;
pub type ObjectRefFieldsHeader = u16;

pub const OBJECT_HEADER: usize =
	size_of::<ObjectFlags>() + size_of::<usize>() + size_of::<ObjectSize>();

bitflags! {
	#[repr(C)]
	pub struct ObjectFlags: u8 {
		const MARK = 1;
	}
}

#[repr(C)]
pub struct Object {
	pub flags: ObjectFlags,
	/// The forwarding pointer is used in garbage collection to move old object references to their new location.
	pub(crate) forward: usize,
	/// The size (in bytes) of the additional object data (not including header)
	pub(crate) size: ObjectSize,
	/// The object data
	pub(crate) data: [u8; 0],
}

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct ObjectPointer(pub(crate) *mut Object);

impl ObjectPointer {
	pub const NULL: ObjectPointer = ObjectPointer(null_mut());
	pub unsafe fn data(self) -> *mut u8 {
		(&mut (*self.0).data) as *mut [u8] as *mut u8
	}

	pub fn is_null(self) -> bool {
		self.0.is_null()
	}
}
