use crate::value::{Local, RawLocal, StackValue};
use stackalloc::alloca_zeroed;
use std::mem::{size_of, transmute};

pub const FRAME_HEADER_SIZE: usize = size_of::<u16>() + size_of::<u16>() + size_of::<u32>();

/// A frame is a data object which holds a stack and a local variable table for execution within a method.
#[repr(C)]
pub struct Frame {
	pub(crate) stack_size: u16,
	pub(crate) local_size: u16,
	stack_pos: u16,
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
			+ (local_size as usize * size_of::<RawLocal>())
	}

	pub fn size(&self) -> usize {
		Self::get_size(self.stack_size, self.local_size)
	}

	pub fn get_stack(&mut self) -> *mut StackValue {
		unsafe {
			let ptr = (&mut self.data) as *mut [u8] as *mut u8;
			transmute(ptr.add(self.local_size as usize * size_of::<RawLocal>()))
		}
	}

	// Locals
	pub fn get_local_table(&mut self) -> *mut RawLocal {
		unsafe { transmute((&mut self.data) as *mut [u8] as *mut u8) }
	}

	pub fn store_raw(&mut self, idx: u16, value: RawLocal) {
		if idx >= self.local_size {
			panic!("Local out of bounds {idx} >= {}", self.local_size)
		}
		unsafe { self.get_local_table().add(idx as usize).write(value) }
	}

	pub fn load_raw(&mut self, idx: u16) -> RawLocal {
		if idx >= self.local_size {
			panic!("Local out of bounds {idx} >= {}", self.local_size)
		}

		unsafe { self.get_local_table().add(idx as usize).read() }
	}

	pub fn store<L: Local>(&mut self, idx: u16, value: L)
	where
		[RawLocal; L::V]: Sized,
	{
		let data = value.to_raw();
		for i in 0..L::V {
			self.store_raw(i as u16 + idx, data[i])
		}
	}

	pub fn load<L: Local>(&mut self, idx: u16) -> L
	where
		[RawLocal; L::V]: Sized,
	{
		L::from_raw(std::array::from_fn::<RawLocal, { L::V }, _>(|i| {
			self.load_raw(i as u16 + idx)
		}))
	}

	// Stack
	pub fn push(&mut self, value: StackValue) {
		if self.stack_pos >= self.stack_size {
			panic!("Stack overflow {} >= {}", self.stack_pos, self.stack_size)
		}

		unsafe {
			self.get_stack().add(self.stack_pos as usize).write(value);
		}

		self.stack_pos += 1;
	}

	pub fn pop(&mut self) -> StackValue {
		if self.stack_pos == 0 {
			panic!("Stack underflow")
		}
		self.stack_pos -= 1;

		unsafe { self.get_stack().add(self.stack_pos as usize).read() }
	}
}
