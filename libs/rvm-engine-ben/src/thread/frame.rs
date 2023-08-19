use std::mem::{size_of, transmute};

use stackalloc::alloca_zeroed;

use crate::value::StackValue;

pub const FRAME_HEADER_SIZE: usize = size_of::<u16>() + size_of::<u16>() + size_of::<u32>();

/// A frame is a data object which holds a stack and a local variable table for execution within a method.
#[repr(C)]
pub struct Frame {
	pub(crate) stack_size: u16,
	pub(crate) local_size: u16,
	pub(crate) stack_pos: u16,
	data: [u8],
}

impl Frame {
	pub fn create(stack_size: u16, local_size: u16, callback: impl FnOnce(&mut Frame)) {
		alloca_zeroed(Frame::get_size(stack_size, local_size), |v| unsafe {
			let frame: &mut Frame = transmute(v);
			frame.stack_size = stack_size;
			frame.local_size = local_size;
			callback(frame);
		});
	}

	pub const fn get_size(stack_size: u16, local_size: u16) -> usize {
		FRAME_HEADER_SIZE
			+ (stack_size as usize * size_of::<StackValue>())
			+ (local_size as usize * size_of::<StackValue>())
	}

	pub fn size(&self) -> usize {
		Self::get_size(self.stack_size, self.local_size)
	}

	pub fn get_stack_mut(&mut self) -> *mut StackValue {
		unsafe {
			let ptr = (&mut self.data) as *mut [u8] as *mut u8;
			transmute(ptr.add(self.local_size as usize * size_of::<StackValue>()))
		}
	}

	pub fn get_stack(&self) -> *const StackValue {
		unsafe {
			let ptr = (&self.data) as *const [u8] as *const u8;
			transmute(ptr.add(self.local_size as usize * size_of::<StackValue>()))
		}
	}

	// Locals
	pub fn get_local_table_mut(&mut self) -> *mut StackValue {
		unsafe { transmute((&mut self.data) as *mut [u8] as *mut u8) }
	}
	pub fn get_local_table(&self) -> *const StackValue {
		unsafe { transmute((&self.data) as *const [u8] as *const u8) }
	}

	pub fn store(&mut self, idx: u16, value: StackValue) {
		if idx >= self.local_size {
			panic!("Local {} out of bounds > {}", idx, self.local_size);
		}
		unsafe {
			self.get_local_table_mut().add(idx as usize).write(value);
		}
	}

	pub fn load(&self, idx: u16) -> StackValue {
		if idx >= self.local_size {
			panic!("Local {} out of bounds > {}", idx, self.local_size);
		}
		unsafe { self.get_local_table().add(idx as usize).read() }
	}

	// Stack
	pub fn get_stack_value(&self, index: u16) -> StackValue {
		if index >= self.stack_size {
			panic!("Stack overflow {} >= {}", index, self.stack_size)
		}

		unsafe { self.get_stack().add(index as usize).read() }
	}

	pub fn set_stack_value(&mut self, index: u16, value: StackValue) {
		if index >= self.stack_size {
			panic!("Stack overflow {} >= {}", index, self.stack_size)
		}

		unsafe {
			self.get_stack_mut()
				.add(self.stack_pos as usize)
				.write(value);
		}
	}

	pub fn push(&mut self, value: StackValue) {
		self.set_stack_value(self.stack_pos, value);
		self.stack_pos += 1;
	}

	pub fn pop(&mut self) -> StackValue {
		if self.stack_pos == 0 {
			panic!("Stack underflow")
		}
		self.stack_pos -= 1;

		unsafe { self.get_stack_mut().add(self.stack_pos as usize).read() }
	}
}
