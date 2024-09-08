use crate::Reference;
use rvm_gc::GcSweeper;

pub trait RuntimeThread {
	fn visit_roots(&mut self, mapper: impl FnMut(&mut Reference));

	fn gc_sweeper(&mut self) -> &mut GcSweeper;
}
