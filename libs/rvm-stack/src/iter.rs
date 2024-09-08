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

pub struct CallStackIter<'a, 'd, U: StackUser> {
	stack: &'a CallStack<'d, U>,
	current_pos: usize,
}

impl<'a, 'd, U: StackUser> CallStackIter<'a, 'd, U> {
	pub(crate) fn new(stack: &'a CallStack<'d, U>) -> Self {
		CallStackIter {
			stack,
			current_pos: 0,
		}
	}
}
impl<'a, 'd, U: StackUser> Iterator for CallStackIter<'a, 'd, U> {
	type Item = Frame<'a, 'd, U>;

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

pub struct CallStackIterMut<'a, 'd, U: StackUser> {
	stack: &'a mut CallStack<'d, U>,
	current_pos: usize,
}

impl<'a, 'd, U: StackUser> CallStackIterMut<'a, 'd, U> {
	pub(crate) fn new(stack: &'a mut CallStack<'d, U>) -> Self {
		CallStackIterMut {
			stack,
			current_pos: 0,
		}
	}
}

impl<'a, 'd, U: StackUser> Iterator for CallStackIterMut<'a, 'd, U> {
	type Item = FrameMut<'a, 'd, U>;

	fn next(&mut self) -> Option<Self::Item> {
		let ticket = unsafe {
			// SAFETY: we manage current_pos so that we increment by the whole frame size
			create_ticket(self.current_pos, self.stack)?
		};

		let frame = self.stack.get_mut(&ticket);
		self.current_pos += frame.frame_size();
		Some(unsafe { transmute::<FrameMut<'_, '_, U>, FrameMut<'_, '_, U>>(frame) })
	}
}
