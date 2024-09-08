use crate::{GcUser, ALIGNMENT};
use bitflags::bitflags;
use rvm_core::align_size;
use std::mem::size_of;
use std::ops::{Deref, DerefMut};
use std::ptr::null_mut;

pub type ObjectSize = u16;

bitflags! {
	#[repr(C)]
	pub struct ObjectFlags: u8 {
		const MARK = 1;
	}
}

#[repr(C)]
pub struct GcHeader<U: GcUser> {
	pub(crate) flags: ObjectFlags,
	/// The forwarding pointer is used in garbage collection to move old object references to their new location.
	pub(crate) forward: *mut u8,
	/// The size (in bytes) of the additional object data (not including header)
	raw_size: ObjectSize,
	pub(crate) user: U::Header,
}

impl<U: GcUser> GcHeader<U> {
	pub fn new(flags: ObjectFlags, total_size: usize, external: U::Header) -> Option<GcHeader<U>> {
		let data_size = total_size - Self::SIZE;
		debug_assert!(data_size % ALIGNMENT == 0, "Bad size");
		let raw_size = data_size / ALIGNMENT;

		if raw_size > (ObjectSize::MAX as usize) {
			return None;
		}

		Some(GcHeader {
			flags,
			forward: null_mut(),
			raw_size: raw_size as ObjectSize,
			user: external,
		})
	}

	pub fn user(&self) -> &U::Header {
		&self.user
	}
	pub fn data_size(&self) -> usize {
		self.raw_size as usize * ALIGNMENT
	}
}

impl<U: GcUser> Deref for GcHeader<U> {
	type Target = U::Header;

	fn deref(&self) -> &Self::Target {
		&self.user
	}
}
impl<U: GcUser> DerefMut for GcHeader<U> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.user
	}
}

impl<U: GcUser> GcHeader<U> {
	pub const SIZE: usize = align_size(size_of::<GcHeader<U>>(), ALIGNMENT);
	pub const MAX_DATA_SIZE: usize = ObjectSize::MAX as usize * ALIGNMENT;
}
