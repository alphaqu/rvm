use crate::{CallStack, Frame, FrameMut, FrameTicket, StackUser};
use std::mem::transmute;

/// This ensures that the ticket points to a location on the stack which is not outside the used memory.
///
/// # Safety
/// Caller must ensure that position points to the start of a stack.
unsafe fn create_ticket<U: StackUser>(pos: usize, stack: &CallStack<U>) -> Option<FrameTicket<U>> {
	if pos == stack.pos {
		return None;
	} else if pos > stack.pos {
		panic!("Overstepped");
	}

	Some(FrameTicket::new(pos))
}

pub struct CallStackIter<'a, U: StackUser> {
	stack: &'a CallStack<U>,
	current_pos: usize,
}

impl<'a, U: StackUser> CallStackIter<'a, U> {
	pub(crate) fn new(stack: &'a CallStack<U>) -> Self {
		CallStackIter {
			stack,
			current_pos: 0,
		}
	}
}
impl<'a, U: StackUser> Iterator for CallStackIter<'a, U> {
	type Item = Frame<'a, U>;

	fn next(&mut self) -> Option<Self::Item> {
		let ticket = unsafe {
			// SAFETY: we manage current_pos so that we increment by the whole frame size
			create_ticket(self.current_pos, self.stack)?
		};
		let frame = self.stack.get(&ticket);

		self.current_pos += frame.frame_size();
		Some(frame)
	}
}

pub struct CallStackIterMut<'a, U: StackUser> {
	stack: &'a mut CallStack<U>,
	current_pos: usize,
}

impl<'a, U: StackUser> CallStackIterMut<'a, U> {
	pub(crate) fn new(stack: &'a mut CallStack<U>) -> Self {
		CallStackIterMut {
			stack,
			current_pos: 0,
		}
	}
}

impl<'a, U: StackUser> Iterator for CallStackIterMut<'a, U> {
	type Item = FrameMut<'a, U>;

	fn next(&mut self) -> Option<Self::Item> {
		let ticket = unsafe {
			// SAFETY: we manage current_pos so that we increment by the whole frame size
			create_ticket(self.current_pos, self.stack)?
		};

		let frame = self.stack.get_mut(&ticket);
		self.current_pos += frame.frame_size();
		Some(unsafe { transmute::<FrameMut<'_, U>, FrameMut<'_, U>>(frame) })
	}
}
