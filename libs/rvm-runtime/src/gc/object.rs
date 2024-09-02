use std::mem::size_of;

use bitflags::bitflags;

pub type ObjectSize = u16;

pub const OBJECT_HEADER: usize = size_of::<GcHeader>();

bitflags! {
	#[repr(C)]
	pub struct ObjectFlags: u8 {
		const MARK = 1;
	}
}

#[repr(C)]
pub struct GcHeader {
	pub(crate) flags: ObjectFlags,
	/// The forwarding pointer is used in garbage collection to move old object references to their new location.
	pub(crate) forward: usize,
	/// The size (in bytes) of the additional object data (not including header)
	pub(crate) size: ObjectSize,
}
