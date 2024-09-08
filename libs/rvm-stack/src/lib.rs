#![feature(pointer_is_aligned_to)]
#![feature(new_uninit)]

mod iter;
mod ticket;
mod value;

pub use iter::*;
use std::fmt::Display;
pub use ticket::*;
pub use value::*;

use bytemuck::Zeroable;
use rvm_core::align_size;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::slice::{from_raw_parts, from_raw_parts_mut};
use tracing::{info, trace};

const FRAME_ALIGNMENT: usize = 4;

pub trait StackUser {
	type StackEntry: Sized + Zeroable + Copy;
	type FrameHeader: Sized + Clone + Copy;
}

pub struct CallStack<U> {
	ptr: *mut [u8],
	aligned_ptr: *mut u8,
	size: usize,
	pos: usize,
	_u: PhantomData<fn() -> U>,
}

impl<U> Drop for CallStack<U> {
	fn drop(&mut self) {
		unsafe {
			// drop the box
			let _ = Box::from_raw(self.ptr);
		}
	}
}
impl<U: StackUser> CallStack<U> {
	//pub fn new_on_stack<O>(size: usize, func: impl FnOnce(CallStack<U>) -> O) -> O {
	//	alloca_zeroed(size, |v| {
	//		let stack = CallStack::new(v);
	//		func(stack)
	//	})
	//}

	pub fn new_on_heap(size: usize) -> CallStack<U> {
		let values = Box::<[u8]>::new_uninit_slice(size);
		let values = unsafe {
			// We don't care about init because our stack zeroes out the data on frame creation.
			values.assume_init()
		};

		CallStack::new(values)
	}
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

	pub fn new(mut data: Box<[u8]>) -> Self {
		let unaligned_ptr = data.as_mut_ptr();
		let padding = unaligned_ptr.align_offset(FRAME_ALIGNMENT);
		if padding == usize::MAX || padding > data.len() {
			panic!("Could not find a way to align data");
		}

		info!("Padding: {padding}");

		let raw = Box::into_raw(data);
		let aligned_ptr = unsafe { (raw as *mut u8).add(padding) };
		let stack = CallStack {
			aligned_ptr,
			size: raw.len() - padding,
			ptr: raw,
			pos: 0,
			_u: Default::default(),
		};

		assert!(stack.aligned_ptr.is_aligned_to(FRAME_ALIGNMENT));
		stack
	}

	pub fn size(&self) -> usize {
		self.size
	}

	pub fn get(&self, id: &FrameTicket<U>) -> Frame<'_, U> {
		unsafe {
			let pointer = self.aligned_ptr.add(id.start_pos());

			Frame {
				raw: RawFrame {
					ptr: pointer,
					_v: Default::default(),
				},
				_p: Default::default(),
			}
		}
	}

	pub fn get_mut(&mut self, id: &FrameTicket<U>) -> FrameMut<'_, U> {
		unsafe {
			let pointer = self.aligned_ptr.add(id.start_pos());
			FrameMut {
				raw: RawFrame {
					ptr: pointer,
					_v: Default::default(),
				},
				_p: Default::default(),
			}
		}
	}

	pub fn pop(&mut self, ticket: FrameTicket<U>) {
		let frame = self.get_mut(&ticket);
		let frame_size = frame.frame_size();

		trace!(
			"Popping frame {}-{} ({}) at {:?}",
			self.pos - frame_size,
			self.pos,
			frame_size,
			unsafe { self.aligned_ptr.add(self.pos - frame_size) }
		);
		if ticket.start_pos() + frame_size != self.pos {
			panic!("Tried to pop non-last frame.");
		}

		self.pos -= frame_size;
	}

	pub fn push(
		&mut self,
		stack_size: u16,
		local_size: u16,
		header: U::FrameHeader,
	) -> Option<FrameGuard<'_, U>> {
		let frame_size = RawFrame::<U>::size(stack_size, local_size);
		let remaining = self.size() - self.pos;

		if frame_size > remaining {
			return None;
		}

		trace!(
			"Pushing frame {}-{} ({}) at {:?}",
			self.pos,
			self.pos + frame_size,
			frame_size,
			unsafe { self.aligned_ptr.add(self.pos) }
		);
		let id = unsafe { FrameTicket::<U>::new(self.pos) };
		self.pos += frame_size;

		let mut frame = FrameGuard {
			stack: self,
			ticket: Some(id),
		};

		frame
			.frame_mut()
			.ensure_zeroed(stack_size, local_size, header);
		Some(frame)
	}

	pub fn iter(&self) -> CallStackIter<'_, U> {
		CallStackIter::new(self)
	}

	pub fn iter_mut(&mut self) -> CallStackIterMut<'_, U> {
		CallStackIterMut::new(self)
	}
}

#[repr(C, align(4))]
#[derive(Debug)]
pub struct FrameHeader<U: StackUser> {
	// MODIFYING THESE IS HIGHLY UNSAFE!!!
	stack_size: u16,
	local_size: u16,

	stack_pos: u16,
	custom: U::FrameHeader,
}

impl<U: StackUser> FrameHeader<U> {
	pub fn stack_size(&self) -> u16 {
		self.stack_size
	}

	pub fn local_size(&self) -> u16 {
		self.local_size
	}
}
impl<U: StackUser> Clone for FrameHeader<U> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<U: StackUser> Copy for FrameHeader<U> {}

impl<U: StackUser> Deref for FrameHeader<U> {
	type Target = U::FrameHeader;

	fn deref(&self) -> &Self::Target {
		&self.custom
	}
}

impl<U: StackUser> DerefMut for FrameHeader<U> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.custom
	}
}

// TODO Do stack watermarking where we dont have an extra bit saying what kind of data we have, we just need to know where on the stack the references are.

#[repr(transparent)]
pub struct RawFrame<U: StackUser> {
	ptr: *mut u8,
	_v: PhantomData<U::FrameHeader>,
}

impl<U: StackUser> RawFrame<U> {
	fn ensure_zeroed(&mut self, stack_size: u16, local_size: u16, custom: U::FrameHeader) {
		*self.header_mut() = FrameHeader {
			stack_size,
			stack_pos: 0,
			local_size,
			custom,
		};

		for local in self.local_slice_mut() {
			*local = U::StackEntry::zeroed();
		}

		// This is not needed because that space will be set by push
		//for stack in self.stack_slice_mut(true) {
		//	*stack = V::zeroed();
		//}
	}

	fn frame_size(&self) -> usize {
		let header = self.header();
		RawFrame::<U>::size(header.stack_size, header.local_size)
	}

	const fn size(stack_size: u16, local_size: u16) -> usize {
		let size = size_of::<FrameHeader<U>>()
			+ size_of::<U::StackEntry>() * (stack_size as usize + local_size as usize);
		align_size(size, FRAME_ALIGNMENT)
	}

	#[inline(always)]
	pub fn header(&self) -> &FrameHeader<U> {
		unsafe { &*(self.ptr as *mut FrameHeader<U> as *const FrameHeader<U>) }
	}
	#[inline(always)]
	fn header_mut(&mut self) -> &mut FrameHeader<U> {
		unsafe { &mut *(self.ptr as *mut FrameHeader<U>) }
	}

	#[inline(always)]
	fn data_start(&self) -> *const u8 {
		unsafe { self.ptr.add(size_of::<FrameHeader<U>>()) as *const u8 }
	}

	#[inline(always)]
	fn data_start_mut(&mut self) -> *mut u8 {
		unsafe { self.ptr.add(size_of::<FrameHeader<U>>()) }
	}

	pub fn local_slice(&self) -> &[U::StackEntry] {
		let data = self.data_start() as *const U::StackEntry;
		unsafe { from_raw_parts(data, self.header().local_size as usize) }
	}

	pub fn local_slice_mut(&mut self) -> &mut [U::StackEntry] {
		let data = self.data_start_mut() as *mut U::StackEntry;
		unsafe { from_raw_parts_mut(data, self.header().local_size as usize) }
	}

	pub fn stack_slice(&self) -> &[U::StackEntry] {
		let header = self.header();
		//let size = if full {
		//	header.stack_size
		//} else {
		//	header.stack_pos
		//} as usize;

		let data = self.data_start() as *const U::StackEntry;
		unsafe {
			let stack = data.add(header.local_size as usize);
			from_raw_parts(stack, header.stack_pos as usize)
		}
	}

	pub fn stack_slice_mut(&mut self) -> &mut [U::StackEntry] {
		let header = *self.header();

		//let size = if full {
		//	header.stack_size
		//} else {
		//	header.stack_pos
		//} as usize;

		let data = self.data_start_mut() as *mut U::StackEntry;
		unsafe {
			let stack = data.add(header.local_size as usize);
			from_raw_parts_mut(stack, header.stack_pos as usize)
		}
	}

	pub fn load(&self, index: u16) -> U::StackEntry {
		self.local_slice()[index as usize]
	}

	pub fn store(&mut self, index: u16, value: U::StackEntry) {
		self.local_slice_mut()[index as usize] = value;
	}

	pub fn push(&mut self, value: U::StackEntry) {
		let header = self.header_mut();

		let pos = header.stack_pos as usize;
		if pos == header.stack_size as usize {
			panic!("Stack overflow! len: {}", header.stack_size as usize)
		}
		header.stack_pos += 1;

		let slice = self.stack_slice_mut();
		slice[pos] = value;
	}

	pub fn pop(&mut self) -> U::StackEntry {
		let header = self.header();

		let pos = header.stack_pos as usize;
		if pos == 0 {
			panic!("Stack underflow!");
		}
		let slice = self.stack_slice_mut();
		let value = slice[pos - 1];
		self.header_mut().stack_pos -= 1;
		value
	}
}

impl<U: StackUser> Deref for RawFrame<U> {
	type Target = FrameHeader<U>;

	fn deref(&self) -> &Self::Target {
		self.header()
	}
}
impl<U: StackUser> DerefMut for RawFrame<U> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.header_mut()
	}
}

impl<U: StackUser> RawFrame<U>
where
	U::StackEntry: Display,
{
	// DEBUG
	pub fn stack_values_debug(&self) -> String {
		let mut stack_values = Vec::new();
		for value in self.stack_slice() {
			stack_values.push(format!("{value}"));
		}
		stack_values.join(",")
	}

	pub fn local_values_debug(&self) -> String {
		let mut local_values = Vec::new();
		for value in self.local_slice() {
			local_values.push(format!("{value}"));
		}
		local_values.join(",")
	}
}

pub struct Frame<'a, U: StackUser> {
	raw: RawFrame<U>,
	_p: PhantomData<&'a CallStack<U>>,
}

impl<'a, U: StackUser> Deref for Frame<'a, U> {
	type Target = RawFrame<U>;

	fn deref(&self) -> &Self::Target {
		&self.raw
	}
}

pub struct FrameMut<'a, U: StackUser> {
	raw: RawFrame<U>,
	_p: PhantomData<&'a mut CallStack<U>>,
}

impl<'a, U: StackUser> Deref for FrameMut<'a, U> {
	type Target = RawFrame<U>;

	fn deref(&self) -> &Self::Target {
		&self.raw
	}
}
impl<'a, 'd, U: StackUser> DerefMut for FrameMut<'a, U> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.raw
	}
}

pub struct FrameGuard<'a, U: StackUser> {
	pub stack: &'a mut CallStack<U>,
	ticket: Option<FrameTicket<U>>,
}

impl<'a, U: StackUser> FrameGuard<'a, U> {
	#[must_use = "Must use ticket and CallStack::pop when the frame has been finished. Else you risk loosing the frame chain."]
	pub fn to_ticket(mut self) -> FrameTicket<U> {
		self.ticket.take().unwrap()
	}

	pub fn frame(&self) -> Frame<'_, U> {
		self.stack.get(self.ticket.as_ref().unwrap())
	}

	pub fn frame_mut(&mut self) -> FrameMut<'_, U> {
		self.stack.get_mut(self.ticket.as_ref().unwrap())
	}
}

impl<'a, 'd, U: StackUser> Drop for FrameGuard<'a, U> {
	fn drop(&mut self) {
		if let Some(ticket) = self.ticket.take() {
			self.stack.pop(ticket);
		}
	}
}

//impl<'a, 'd> Deref for FrameScope<'a, 'd> {
// 	type Target = RawFrame;
//
// 	fn deref(&self) -> &Self::Target {
// 		self.stack.get(self.ticket.as_ref().unwrap())
// 	}
// }
//
// impl<'a, 'd> DerefMut for FrameScope<'a, 'd> {
// 	fn deref_mut(&mut self) -> &mut Self::Target {
// 		self.stack.get_mut(self.ticket.as_ref().unwrap())
// 	}
// }

#[cfg(test)]
mod tests {
	use super::*;

	pub struct Hi;

	impl StackUser for Hi {
		type StackEntry = i32;
		type FrameHeader = [u8; 3];
	}
	#[test]
	fn compile() {
		rvm_core::init();

		let mut stack = CallStack::<Hi>::new_on_heap(1024);
		let mut scope = stack.push(4, 4, [4, 4, 2]).expect("Frame allocation error");

		let mut frame = scope.frame_mut();
		frame.push(423);
		frame.push(423);
		frame.push(423);

		let scope2 = scope
			.stack
			.push(4, 4, [4, 4, 2])
			.expect("Frame allocation error");
		let ticket = scope2.to_ticket();

		scope.stack.pop(ticket);
		drop(scope);
		info!("hi");

		//let ticket = scope2.to_ticket();
		//
		//let mut frame = scope.frame_mut();
		//frame.push(StackValue::Int(214));
		//
		//scope.stack.pop(ticket);
	}
}
