use rvm_core::{Id, Kind, Type};
use std::mem::size_of;

pub const ARRAY_BASE_OFFSET: usize = size_of::<i32>();

pub struct ArrayClass {
	component: Type,
}

impl ArrayClass {
	pub fn new(component: Type) -> ArrayClass {
		ArrayClass { component }
	}

	//pub fn size(&self, obj: Ref) -> usize {
	//	// reuse that method
	//	let length = Array::<()> {
	//		reference: obj,
	//		_p: Default::default(),
	//	}
	//	.get_length();
	//	ARRAY_BASE_OFFSET + (self.component.kind().size() * (length as usize))
	//}

	pub fn component(&self) -> &Type {
		&self.component
	}
}

// TODO useful for rust interop crate
// pub struct Array<T> {
// 	reference: Ref,
// 	_p: PhantomData<T>,
// }
//
// impl<T> Array<T> {
// 	pub fn get_length(&self) -> i32 {
// 		unsafe {
// 			let ptr = self.reference.ptr();
// 			i32::from_le_bytes(read_arr(ptr))
// 		}
// 	}
// }
//
// impl<T: Value> Array<T> {
// 	pub fn load(&self, idx: i32) -> T {
// 		unsafe {
// 			let ptr = self.get_index_ptr(idx);
// 			T::read(ptr)
// 		}
// 	}
//
// 	pub fn store(&self, idx: i32, value: T) {
// 		unsafe {
// 			let ptr = self.get_index_ptr(idx);
// 			T::write(ptr, value);
// 		}
// 	}
//
// 	unsafe fn get_index_ptr(&self, idx: i32) -> *mut u8 {
// 		self.reference
// 			.ptr()
// 			.add(ARRAY_BASE_OFFSET + (T::ty().size() * (idx as usize)))
// 	}
// }
//
// impl<T: Value> Deref for Array<T> {
// 	type Target = Ref;
//
// 	fn deref(&self) -> &Self::Target {
// 		&self.reference
// 	}
// }
