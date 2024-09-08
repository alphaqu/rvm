use crate::{CallStack, Frame, FrameTicket};

/// This ensures that the ticket points to a location on the stack which is not outside the used memory.
///
/// # Safety
/// Caller must ensure that position points to the start of a stack.
unsafe fn create_ticket(pos: usize, stack: &CallStack) -> Option<FrameTicket> {
	if pos == stack.pos {
		return None;
	} else if pos > stack.pos {
		panic!("Overstepped");
	}

	Some(FrameTicket::new(pos))
}

pub struct CallStackIter<'a, 'd> {
	stack: &'a CallStack<'d>,
	current_pos: usize,
}

impl<'a, 'd> CallStackIter<'a, 'd> {
	pub(crate) fn new(stack: &'a CallStack<'d>) -> Self {
		CallStackIter {
			stack,
			current_pos: 0,
		}
	}
}
impl<'a, 'd> Iterator for CallStackIter<'a, 'd> {
	type Item = &'a Frame;

	fn next(&mut self) -> Option<Self::Item> {
		let ticket = unsafe {
			// SAFETY: we manage current_pos so that we increment by the whole frame size
			create_ticket(self.current_pos, self.stack)?
		};
		let frame = self.stack.get(&ticket);

		self.current_pos += frame.header.frame_size();
		Some(frame)
	}
}

pub struct CallStackIterMut<'a, 'd> {
	stack: &'a mut CallStack<'d>,
	current_pos: usize,
}

impl<'a, 'd> CallStackIterMut<'a, 'd> {
	pub(crate) fn new(stack: &'a mut CallStack<'d>) -> Self {
		CallStackIterMut {
			stack,
			current_pos: 0,
		}
	}
}

impl<'a, 'd> Iterator for CallStackIterMut<'a, 'd> {
	type Item = &'a mut Frame;

	fn next(&mut self) -> Option<Self::Item> {
		let ticket = unsafe {
			// SAFETY: we manage current_pos so that we increment by the whole frame size
			create_ticket(self.current_pos, self.stack)?
		};

		let frame = unsafe {
			// This is required to return a mutable reference to the frame.
			&mut *(self.stack.get_mut(&ticket) as *mut Frame)
		};

		self.current_pos += frame.header.frame_size();
		Some(frame)
	}
}
