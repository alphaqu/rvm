use std::marker::PhantomData;
use std::mem::size_of;
use std::ops::Deref;
use rvm_core::{Kind, Ref, Value};

pub const ARRAY_BASE_OFFSET: usize = size_of::<i32>();

pub struct ArrayDesc {
	component: Kind,
}

impl ArrayDesc {
	pub fn new(component: Kind) -> ArrayDesc {
		ArrayDesc { component }
	}

	pub fn size(&self, obj: Ref) -> usize {
		// reuse that method
		let length = Array::<()> {
			reference: obj,
			_p: Default::default(),
		}
		.get_length();
		ARRAY_BASE_OFFSET + (self.component.size() * (length as usize))
	}
	
	pub fn component(&self) -> Kind {
		self.component
	}
}

pub struct Array<V> {
	reference: Ref,
	_p: PhantomData<V>,
}

impl<T> Array<T> {
	pub unsafe fn new(reference: Ref) -> Array<T> {
		Array {
			reference,
			_p: Default::default(),
		}
	}

	pub fn get_length(&self) -> i32 {
		unsafe {
			let ptr = self.reference.ptr();
			i32::read(ptr)
		}
	}
}

impl<T: Value> Array<T> {
	pub fn load(&self, idx: i32) -> T {
		unsafe {
			let ptr = self.get_index_ptr(idx);
			T::read(ptr)
		}
	}

	pub fn store(&self, idx: i32, value: T) {
		unsafe {
			let ptr = self.get_index_ptr(idx);
			T::write(ptr, value);
		}
	}

	unsafe fn get_index_ptr(&self, idx: i32) -> *mut u8 {
		self.reference
			.ptr()
			.add(ARRAY_BASE_OFFSET + (T::ty().size() * (idx as usize)))
	}
}

impl<T: Value> Deref for Array<T> {
	type Target = Ref;

	fn deref(&self) -> &Self::Target {
		&self.reference
	}
}
