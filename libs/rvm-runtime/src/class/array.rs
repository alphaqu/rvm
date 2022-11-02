use crate::class::{read_arr, write_arr};
use crate::object::{Type, ValueType};
use crate::{Class, ClassKind, JError, JResult, Ref, Runtime};
use parking_lot::lock_api::MappedRwLockReadGuard;
use parking_lot::RawRwLock;
use rvm_core::Id;
use std::marker::PhantomData;
use std::mem::size_of;
use std::ops::Deref;

pub const ARRAY_BASE_OFFSET: usize = size_of::<i32>();

pub struct ArrayClass {
	component: ValueType,
}

impl ArrayClass {
	pub fn new(component: ValueType) -> ArrayClass {
		ArrayClass { component }
	}

	#[deprecated]
	pub fn new_array(&self, class_id: Id<Class>, length: i32, runtime: &Runtime) -> Ref {
		unsafe {
			let object = runtime.gc.write().unwrap().alloc(
				class_id,
				ARRAY_BASE_OFFSET + (self.component.size() * (length as usize)),
			);
			let ptr = object.ptr().add(0);
			write_arr(ptr, i32::to_le_bytes(length));
			object
		}
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

	pub fn component(&self) -> ValueType {
		self.component
	}
}

impl<'a> Runtime<'a> {
	//pub fn new_array<T: Type>(&self, class_id: Id<Class>, length: i32) -> JResult<Array<T>> {
	//	// ensure its an array class
	//	let _ = self.get_array_class(class_id)?;
	//	Ok(unsafe {
	//		let object = self.gc.write().unwrap().alloc(
	//			class_id,
	//			ARRAY_BASE_OFFSET + (T::ty().size() * (length as usize)),
	//		);
	//		let ptr = object.ptr();
	//		write_arr(ptr, i32::to_le_bytes(length));
	//
	//		Array {
	//			reference: object,
	//			_p: Default::default()
	//		}
	//	})
	//}

	pub fn get_untyped_array(&self, reference: Ref) -> JResult<Array<()>> {
		// ensure its an array class
		let _ = self.get_array_class(reference.get_class())?;
		Ok(Array {
			reference,
			_p: Default::default(),
		})
	}

	pub fn get_array<T: Type>(&self, reference: Ref) -> JResult<Array<T>> {
		let component = self.get_array_class(reference.get_class())?.component;
		if T::ty() != component {
			Err(JError::new("Array component mismatch"))
		} else {
			Ok(Array {
				reference,
				_p: Default::default(),
			})
		}
	}

	fn get_array_class(
		&self,
		class_id: Id<Class>,
	) -> JResult<MappedRwLockReadGuard<RawRwLock, ArrayClass>> {
		let guard = self.cl.get(class_id);
		match &guard.kind {
			ClassKind::Array(_) => Ok(MappedRwLockReadGuard::map(guard, |v| {
				if let ClassKind::Array(class) = &v.kind {
					class
				} else {
					panic!("wtf")
				}
			})),
			_ => return Err(JError::new("Expected array class")),
		}
	}
}

pub struct Array<T> {
	reference: Ref,
	_p: PhantomData<T>,
}

impl<T> Array<T> {
	pub fn get_length(&self) -> i32 {
		unsafe {
			let ptr = self.reference.ptr();
			i32::from_le_bytes(read_arr(ptr))
		}
	}
}

impl<T: Type> Array<T> {
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

impl<T: Type> Deref for Array<T> {
	type Target = Ref;

	fn deref(&self) -> &Self::Target {
		&self.reference
	}
}
