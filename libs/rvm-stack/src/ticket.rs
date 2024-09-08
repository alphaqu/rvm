use std::marker::PhantomData;

/// A token is a non-clonable/copyable address location to the frame,
/// when this is returned to the callstack, the frame is removed.
///
/// It holds the guarantee that this frame exists.
#[repr(transparent)]
#[derive(Ord, PartialOrd, Eq, PartialEq)]
pub struct FrameTicket<V>(usize, PhantomData<fn() -> V>);

impl<V> FrameTicket<V> {
	/// # Safety
	/// Caller must ensure that the pos points to a valid frame location.
	pub(crate) unsafe fn new(pos: usize) -> FrameTicket<V> {
		FrameTicket(pos, PhantomData)
	}

	pub(crate) fn start_pos(&self) -> usize {
		self.0
	}
}
