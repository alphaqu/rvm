pub use crate::value::reference::*;
use mmtk::util::{Address, ObjectReference};
use rvm_core::Kind;
use std::ptr::{read, write};

mod reference;

pub trait Value: Sized {
	fn ty() -> Kind;
	unsafe fn write(ptr: *mut u8, value: Self);
	unsafe fn read(ptr: *mut u8) -> Self;
}

#[derive(Copy, Clone, Debug)]
pub enum DynValue {
	Byte(i8),
	Short(i16),
	Int(i32),
	Long(i64),
	Char(u16),
	Float(f32),
	Double(f64),
	Boolean(bool),
	Ref(ObjectReference),
}

macro_rules! impl_from {
	($TY:ty, $KIND:ident) => {
		impl From<$TY> for DynValue {
			fn from(value: $TY) -> Self {
				DynValue::$KIND(value)
			}
		}

		impl TryInto<$TY> for DynValue {
			type Error = DynValue;

			fn try_into(self) -> Result<$TY, Self::Error> {
				match self {
					DynValue::$KIND(v) => Ok(v),
					_ => Err(self),
				}
			}
		}
	};
}

impl_from!(i8, Byte);
impl_from!(i16, Short);
impl_from!(i32, Int);
impl_from!(i64, Long);
impl_from!(u16, Char);
impl_from!(f32, Float);
impl_from!(f64, Double);
impl_from!(bool, Boolean);
impl_from!(ObjectReference, Ref);

impl DynValue {
	pub fn ty(&self) -> Kind {
		match self {
			DynValue::Byte(_) => i8::ty(),
			DynValue::Short(_) => i16::ty(),
			DynValue::Int(_) => i32::ty(),
			DynValue::Long(_) => i64::ty(),
			DynValue::Char(_) => u16::ty(),
			DynValue::Float(_) => f32::ty(),
			DynValue::Double(_) => f64::ty(),
			DynValue::Boolean(_) => bool::ty(),
			DynValue::Ref(_) => ObjectReference::ty(),
		}
	}

	pub unsafe fn write(self, ptr: *mut u8) {
		match self {
			DynValue::Byte(v) => i8::write(ptr, v),
			DynValue::Short(v) => i16::write(ptr, v),
			DynValue::Int(v) => i32::write(ptr, v),
			DynValue::Long(v) => i64::write(ptr, v),
			DynValue::Char(v) => u16::write(ptr, v),
			DynValue::Float(v) => f32::write(ptr, v),
			DynValue::Double(v) => f64::write(ptr, v),
			DynValue::Boolean(v) => bool::write(ptr, v),
			DynValue::Ref(v) => ObjectReference::write(ptr, v),
		}
	}
	pub unsafe fn read(ptr: *mut u8, kind: Kind) -> Self {
		match kind {
			Kind::Byte => DynValue::Byte(i8::read(ptr)),
			Kind::Short => DynValue::Short(i16::read(ptr)),
			Kind::Int => DynValue::Int(i32::read(ptr)),
			Kind::Long => DynValue::Long(i64::read(ptr)),
			Kind::Char => DynValue::Char(u16::read(ptr)),
			Kind::Float => DynValue::Float(f32::read(ptr)),
			Kind::Double => DynValue::Double(f64::read(ptr)),
			Kind::Boolean => DynValue::Boolean(bool::read(ptr)),
			Kind::Reference => DynValue::Ref(ObjectReference::read(ptr)),
		}
	}
}

macro_rules! impl_direct {
	($VAR:ident $TY:ty) => {
		impl Value for $TY {
			fn ty() -> Kind {
				Kind::$VAR
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

	unsafe fn write(ptr: *mut u8, value: Self) {
		write(ptr, value as u8)
	}

	unsafe fn read(ptr: *mut u8) -> Self {
		read(ptr) != 0
	}
}

impl Value for ObjectReference {
	fn ty() -> Kind {
		Kind::Reference
	}

	unsafe fn write(ptr: *mut u8, value: Self) {
		write_arr(ptr, {
			let x: *mut u8 = value.to_raw_address().to_mut_ptr();
			let i = x as usize;
			i.to_le_bytes()
		})
	}

	unsafe fn read(ptr: *mut u8) -> Self {
		ObjectReference::from_raw_address(Address::from_ptr(
			usize::from_le_bytes(read_arr(ptr)) as *mut u8
		))
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
