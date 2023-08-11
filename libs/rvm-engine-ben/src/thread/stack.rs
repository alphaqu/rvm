use crate::thread::frame::Frame;
use stackalloc::alloca_zeroed;
use std::mem::{size_of, transmute};
use std::ops::{Deref, DerefMut};
use tracing::{debug, trace};

pub const STACK_HEADER_SIZE: usize = size_of::<usize>() + size_of::<usize>();

#[repr(C)]
pub struct ThreadStack {
	data_size: usize,
	data_pos: usize,
	stack: [u8; 0],
}

impl ThreadStack {
	pub fn new(size: usize, func: impl FnOnce(&mut ThreadStack)) {
		debug!("Allocating new thread stack ({size}B)");
		alloca_zeroed(size + STACK_HEADER_SIZE, |v| unsafe {
			let stacks: &mut ThreadStack = transmute(v.as_mut_ptr());
			stacks.data_size = size;
			stacks.data_pos = 0;
			func(stacks);
		});
	}
	
	pub fn scope<'f, T>(&mut self, stack_size: u16, local_size: u16, func: impl FnOnce(&mut ThreadStack, ThreadFrame<'f>) -> T) -> T {
		let frame_size = Frame::get_size(stack_size, local_size);
		trace!("Allocating ThreadFrame ({}+{frame_size})", self.data_pos);
		if self.data_pos + frame_size >= self.data_size {
			panic!("Java Stack out of space");
		}

		unsafe {
			let mut ptr = (&mut self.stack as *mut [u8] as *mut Frame).byte_add(self.data_pos);
			self.data_pos += frame_size;

			// Allocate
			let frame: &mut Frame = &mut *ptr;
			frame.stack_size = stack_size;
			frame.local_size = local_size;
			let thread_frame = ThreadFrame {
				finished: false,
				frame,
			};
			
			// run
			let output = func(self, thread_frame);
			
			// Clean up
			let frame_size = frame.size();
			self.data_pos -= frame_size;
			trace!("Finishing ThreadFrame ({}+{frame_size})", self.data_pos);
			frame.finished = true;

			output
		}
	}
	pub fn visit_frames(&self, mut func: impl FnMut(&Frame)) {
		debug!("Visiting thread frames");
		let mut ptr = &self.stack as *const [u8] as *const Frame;
		let mut pos = 0;
		loop {
			unsafe {
				let frame: &Frame = &*ptr.byte_add(pos);
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
