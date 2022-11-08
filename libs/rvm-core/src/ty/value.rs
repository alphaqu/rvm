use crate::ty::kind::Kind;
use std::ptr::{read, write};
use crate::Ref;

#[derive(Copy, Clone, PartialOrd, PartialEq, Debug)]
pub enum ValueEnum {
	Boolean(bool),
	Byte(i8),
	Short(i16),
	Int(i32),
	Long(i64),
	Char(u16),
	Float(f32),
	Double(f64),
	Reference(Ref),
}

pub trait Value: Sized {
	fn ty() -> Kind;
	fn to_enum(self) -> ValueEnum;
	unsafe fn write(ptr: *mut u8, value: Self);
	unsafe fn read(ptr: *mut u8) -> Self;
}

macro_rules! impl_direct {
	($VAR:ident $TY:ty) => {
		impl Value for $TY {
			fn ty() -> Kind {
				Kind::$VAR
			}

			fn to_enum(self) -> ValueEnum {
				ValueEnum::$VAR(self)
			}

			unsafe fn write(ptr: *mut u8, value: Self) {
				write_arr(ptr, value.to_le_bytes())
			}

			unsafe fn read(ptr: *mut u8) -> Self {
				<$TY>::from_le_bytes(read_arr(ptr))
			}
		}
	};
}
impl_direct!(Byte i8);
impl_direct!(Short i16);
impl_direct!(Int i32);
impl_direct!(Long i64);
impl_direct!(Char u16);
impl_direct!(Float f32);
impl_direct!(Double f64);

impl Value for bool {
	fn ty() -> Kind {
		Kind::Boolean
	}

	fn to_enum(self) -> ValueEnum {
		ValueEnum::Boolean(self)
	}

	unsafe fn write(ptr: *mut u8, value: Self) {
		write(ptr, value as u8)
	}

	unsafe fn read(ptr: *mut u8) -> Self {
		read(ptr) != 0
	}
}

impl Value for Ref {
	fn ty() -> Kind {
		Kind::Reference
	}

	fn to_enum(self) -> ValueEnum {
		ValueEnum::Reference(self)
	}

	unsafe fn write(ptr: *mut u8, value: Self) {
		write_arr(ptr, {
			let i = value.ptr() as usize;
			i.to_le_bytes()
		})
	}

	unsafe fn read(ptr: *mut u8) -> Self {
		Ref::new(usize::from_le_bytes(read_arr(ptr)) as *mut u8)
	}
}

#[inline(always)]
pub(crate) unsafe fn read_arr<const C: usize>(ptr: *mut u8) -> [u8; C] {
	let mut out = [0; C];
	for i in 0..C {
		*out.get_unchecked_mut(i) = read(ptr.add(i));
	}
	out
}

#[inline(always)]
pub(crate) unsafe fn write_arr<const C: usize>(ptr: *mut u8, value: [u8; C]) {
	for i in 0..C {
		write(ptr.add(i), *value.get_unchecked(i));
	}
}
