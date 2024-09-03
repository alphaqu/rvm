use std::mem::{size_of, transmute};
use std::ops::{Deref, DerefMut};

use stackalloc::alloca_zeroed;
use tracing::{debug, trace};

use crate::thread::frame::Frame;

pub const STACK_HEADER_SIZE: usize = size_of::<usize>() + size_of::<usize>();

#[repr(C)]
pub struct ThreadStack {
	data_size: usize,
	data_pos: usize,
	stack: [u8; 0],
}

impl ThreadStack {
	pub fn new<T>(size: usize, func: impl FnOnce(&mut ThreadStack) -> T) -> T {
		debug!("Allocating new thread stack ({size}B)");
		alloca_zeroed(size + STACK_HEADER_SIZE, |v| unsafe {
			let stacks: &mut ThreadStack = transmute(v.as_mut_ptr());
			stacks.data_size = size;
			stacks.data_pos = 0;
			func(stacks)
		})
	}

	pub fn create<'f>(&mut self, stack_size: u16, local_size: u16) -> ThreadFrame<'f> {
		let frame_size = Frame::get_size(stack_size, local_size);
		trace!(target: "exe", "Allocating ThreadFrame ({}+{frame_size})", self.data_pos);
		if self.data_pos + frame_size >= self.data_size {
			panic!("Java Stack out of space");
		}

		unsafe {
			let ptr = (&mut self.stack as *mut [u8] as *mut Frame).byte_add(self.data_pos);
			self.data_pos += frame_size;

			let frame: &mut Frame = &mut *ptr;
			frame.stack_size = stack_size;
			frame.local_size = local_size;
			ThreadFrame {
				finished: false,
				frame,
			}
		}
	}

	pub fn finish(&mut self, mut frame: ThreadFrame) {
		if frame.finished {
			panic!("Double finished");
		}
		let frame_size = frame.size();
		self.data_pos -= frame_size;
		trace!(target: "exe", "Finishing ThreadFrame ({}+{frame_size})", self.data_pos);
		frame.finished = true;
	}

	pub fn visit_frames_mut(&mut self, mut func: impl FnMut(&mut Frame)) {
		debug!("Visiting thread frames");
		let ptr = &mut self.stack as *mut [u8] as *mut Frame;
		let mut pos = 0;
		loop {
			unsafe {
				let frame: &mut Frame = &mut *ptr.byte_add(pos);
				func(frame);
				pos += frame.size();
				if pos == self.data_pos {
					return;
				}
			}
		}
	}
}

pub struct ThreadFrame<'f> {
	finished: bool,
	frame: &'f mut Frame,
}

impl<'f> Deref for ThreadFrame<'f> {
	type Target = Frame;

	fn deref(&self) -> &Self::Target {
		&self.frame
	}
}

impl<'f> DerefMut for ThreadFrame<'f> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.frame
	}
}

impl<'f> Drop for ThreadFrame<'f> {
	fn drop(&mut self) {
		if !self.finished {
			panic!("ThreadFrame dropped without being finished.")
		}
	}
}
