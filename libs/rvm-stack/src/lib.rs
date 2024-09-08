#![feature(pointer_is_aligned_to)]

mod iter;
mod ticket;
mod value;

use crate::iter::{CallStackIter, CallStackIterMut};
use crate::ticket::FrameTicket;
use crate::value::StackValue;
use rvm_core::align_size;
use stackalloc::alloca_zeroed;
use std::intrinsics::transmute;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::slice::{from_raw_parts, from_raw_parts_mut};
use tracing::{info, trace};

const DATA_ALIGNMENT: usize = align_of::<Frame>();
const FRAME_ALIGNMENT: usize = 2;

pub struct CallStack<'d> {
	aligned_ptr: *mut u8,
	size: usize,
	padding: usize,
	pos: usize,
	_p: PhantomData<&'d mut [u8]>,
}

impl<'d> CallStack<'d> {
	//pub fn new_stack(size: usize, func: impl FnOnce(&mut CallStack)) {
	// 		alloca_zeroed(size, |v| unsafe {
	// 			let mut stack = CallStack::create(v as *mut [u8]);
	// 			func(&mut stack);
	// 		})
	// 	}
	//
	// 	pub fn new_heap(size: usize) {
	// 		Box::new_zeroed_slice()
	// 	}
	//
	// 	pub unsafe fn create(data: *mut [u8]) -> Self {
	// 		let size = data.len();
	// 		Self {
	// 			data: data as *mut u8,
	// 			pos: 0,
	// 			size,
	// 		}
	// 	}

	pub fn new(data: &'d mut [u8]) -> CallStack<'d> {
		let unaligned_ptr = data.as_mut_ptr();
		let padding = unaligned_ptr.align_offset(DATA_ALIGNMENT);
		if padding == usize::MAX || padding > data.len() {
			panic!("Could not find a way to align data");
		}

		info!("Padding: {padding}");

		let ptr = unsafe { (data as *mut [u8] as *mut u8).add(padding) };
		let stack = CallStack {
			aligned_ptr: ptr,
			size: data.len() - padding,
			padding,
			pos: 0,
			_p: Default::default(),
		};

		assert!(stack.data_ptr().is_aligned_to(DATA_ALIGNMENT));
		stack
	}

	pub fn size(&self) -> usize {
		self.size
	}

	#[inline(always)]
	fn data_ptr_mut(&mut self) -> *mut u8 {
		self.aligned_ptr
	}

	#[inline(always)]
	fn data_ptr(&self) -> *const u8 {
		self.aligned_ptr as *const u8
	}

	pub fn get(&self, id: &FrameTicket) -> &'_ Frame {
		unsafe {
			let pointer = self.data_ptr().add(id.start_pos());
			let header = pointer as *mut Frame as *const Frame;
			&*header
		}
	}

	pub fn get_mut(&mut self, id: &FrameTicket) -> &'_ mut Frame {
		unsafe {
			let pointer = self.data_ptr_mut().add(id.start_pos());
			info!("{pointer:?}");

			let frame = pointer.cast::<Frame>();
			&mut *frame
		}
	}

	pub fn pop(&mut self, ticket: FrameTicket) {
		let frame = self.get_mut(&ticket);
		let frame_size = Frame::size(frame.header.stack_size, frame.header.local_size);
		if ticket.start_pos() + frame_size != self.pos {
			panic!("Tried to pop non-last frame.");
		}
		trace!(
			"Popping frame {}-{} ({}) at {:?}",
			self.pos - frame_size,
			self.pos,
			frame_size,
			unsafe { self.data_ptr().add(self.pos - frame_size) }
		);
		self.pos -= frame_size;
	}

	pub fn push(&mut self, stack_size: u16, local_size: u16) -> Option<FrameScope<'_, 'd>> {
		let frame_size = Frame::size(stack_size, local_size);
		let remaining = self.size() - self.pos;

		if frame_size > remaining {
			return None;
		}

		let id = unsafe { FrameTicket::new(self.pos) };
		trace!(
			"Pushing frame {}-{} ({}) at {:?}",
			self.pos,
			self.pos + frame_size,
			frame_size,
			unsafe { self.data_ptr().add(self.pos) }
		);
		self.pos += frame_size;

		let mut frame = FrameScope {
			stack: self,
			ticket: Some(id),
		};

		frame.ensure_zeroed(stack_size, local_size);

		Some(frame)
	}

	pub fn iter(&self) -> CallStackIter<'_, 'd> {
		CallStackIter::new(self)
	}

	pub fn iter_mut(&mut self) -> CallStackIterMut<'_, 'd> {
		CallStackIterMut::new(self)
	}
}

#[repr(C)]
pub struct FrameHeader {
	pub stack_size: u16,
	pub stack_pos: u16,
	pub local_size: u16,
	// This is to ensure alignment
	pub _0: u16,
}

impl FrameHeader {
	fn frame_size(&self) -> usize {
		Frame::size(self.stack_pos, self.local_size)
	}
}

// TODO Do stack watermarking where we dont have an extra bit saying what kind of data we have, we just need to know where on the stack the references are.

#[repr(C)]
pub struct Frame {
	header: FrameHeader,
	data: [u8; 1],
}

impl Frame {
	const fn size(stack_size: u16, local_size: u16) -> usize {
		let size = size_of::<FrameHeader>()
			+ size_of::<StackValue>() * (stack_size as usize + local_size as usize);
		align_size(size, FRAME_ALIGNMENT)
	}

	fn ensure_zeroed(&mut self, stack_size: u16, local_size: u16) {
		self.header = FrameHeader {
			stack_size,
			stack_pos: 0,
			local_size,
			_0: 0,
		};

		for local in self.local_slice_mut() {
			*local = StackValue::Int(0);
		}

		for stack in self.stack_slice_mut(true) {
			*stack = StackValue::Int(0);
		}
	}

	pub fn header(&self) -> &FrameHeader {
		&self.header
	}

	pub fn local_slice(&self) -> &[StackValue] {
		let data = self.data.as_ptr() as *const StackValue;
		unsafe { from_raw_parts(data, self.header.local_size as usize) }
	}

	pub fn local_slice_mut(&mut self) -> &mut [StackValue] {
		let data = self.data.as_mut_ptr() as *mut StackValue;
		unsafe { from_raw_parts_mut(data, self.header.local_size as usize) }
	}

	pub fn stack_slice(&self, full: bool) -> &[StackValue] {
		let size = if full {
			self.header.stack_size
		} else {
			self.header.stack_pos
		} as usize;
		let data = self.data.as_ptr() as *const StackValue;
		unsafe {
			let stack = data.add(self.header.local_size as usize);
			from_raw_parts(stack, size)
		}
	}

	pub fn stack_slice_mut(&mut self, full: bool) -> &mut [StackValue] {
		let size = if full {
			self.header.stack_size
		} else {
			self.header.stack_pos
		} as usize;
		let data = self.data.as_mut_ptr() as *mut StackValue;
		unsafe {
			let stack = data.add(self.header.local_size as usize);
			from_raw_parts_mut(stack, size)
		}
	}

	pub fn push(&mut self, value: StackValue) {
		let pos = self.header.stack_pos as usize;
		if pos == self.header.stack_size as usize {
			panic!("Stack overflow! len: {}", self.header.stack_size as usize)
		}
		let slice = self.stack_slice_mut(true);
		slice[pos] = value;
		self.header.stack_pos += 1;
	}

	pub fn pop(&mut self) -> StackValue {
		let pos = self.header.stack_pos as usize;
		if pos == 0 {
			panic!("Stack underflow!");
		}
		let slice = self.stack_slice_mut(true);
		let value = slice[pos - 1];
		self.header.stack_pos -= 1;
		value
	}
}
pub struct FrameScope<'a, 'd> {
	pub stack: &'a mut CallStack<'d>,
	ticket: Option<FrameTicket>,
}

impl<'a, 'd> FrameScope<'a, 'd> {
	pub fn to_ticket(mut self) -> FrameTicket {
		self.ticket.take().unwrap()
	}
}

impl<'a, 'd> Drop for FrameScope<'a, 'd> {
	fn drop(&mut self) {
		if let Some(ticket) = self.ticket.take() {
			self.stack.pop(ticket);
		}
	}
}

impl<'a, 'd> Deref for FrameScope<'a, 'd> {
	type Target = Frame;

	fn deref(&self) -> &Self::Target {
		self.stack.get(self.ticket.as_ref().unwrap())
	}
}

impl<'a, 'd> DerefMut for FrameScope<'a, 'd> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.stack.get_mut(self.ticket.as_ref().unwrap())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn compile() {
		rvm_core::init();

		let thing = align_of::<Frame>();
		info!("{}", thing);

		let mut data = vec![0u8; 1024];
		let mut stack = CallStack::new(&mut data);
		let mut scope = stack.push(4, 4).expect("Frame allocation error");
		scope.push(StackValue::Int(214));
		scope.push(StackValue::Int(214));
		scope.push(StackValue::Int(214));
		scope.push(StackValue::Int(214));
		let scope2 = scope.stack.push(4, 4).expect("Frame allocation error");
		let ticket = scope2.to_ticket();

		scope.stack.pop(ticket);
	}
}
