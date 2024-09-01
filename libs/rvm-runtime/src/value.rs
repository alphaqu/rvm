use rvm_core::Kind;
use std::ptr::{read, write};
use std::sync::Arc;

use crate::object::Reference;
use crate::Runtime;

pub trait Returnable {
	fn from_value(runtime: &Arc<Runtime>, value: Option<AnyValue>) -> Self;
}

impl Returnable for () {
	fn from_value(_: &Arc<Runtime>, value: Option<AnyValue>) -> Self {
		assert!(value.is_none());
		()
	}
}
impl<R: Returnable> Returnable for Option<R> {
	fn from_value(runtime: &Arc<Runtime>, value: Option<AnyValue>) -> Self {
		value.map(|_| R::from_value(runtime, value))
	}
}

pub trait Value: Sized + Copy {
	fn kind() -> Kind;
	unsafe fn write(ptr: *mut u8, value: Self);
	unsafe fn read(ptr: *mut u8) -> Self;
	unsafe fn cast(ptr: *mut u8) -> *mut Self {
		ptr as *mut Self
	}
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum AnyValue {
	Byte(i8),
	Short(i16),
	Int(i32),
	Long(i64),
	Char(u16),
	Float(f32),
	Double(f64),
	Boolean(bool),
	Reference(Reference),
}

macro_rules! impl_from {
	($TY:ty, $KIND:ident) => {
		impl Returnable for $TY {
			fn from_value(_: &Arc<Runtime>, value: Option<AnyValue>) -> Self {
				value.unwrap().try_into().unwrap()
			}
		}

		impl From<$TY> for AnyValue {
			fn from(value: $TY) -> Self {
				AnyValue::$KIND(value)
			}
		}

		impl TryInto<$TY> for AnyValue {
			type Error = AnyValue;

			fn try_into(self) -> Result<$TY, Self::Error> {
				match self {
					AnyValue::$KIND(v) => Ok(v),
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
impl_from!(Reference, Reference);

impl AnyValue {
	pub fn kind(&self) -> Kind {
		match self {
			AnyValue::Byte(_) => i8::kind(),
			AnyValue::Short(_) => i16::kind(),
			AnyValue::Int(_) => i32::kind(),
			AnyValue::Long(_) => i64::kind(),
			AnyValue::Char(_) => u16::kind(),
			AnyValue::Float(_) => f32::kind(),
			AnyValue::Double(_) => f64::kind(),
			AnyValue::Boolean(_) => bool::kind(),
			AnyValue::Reference(_) => Reference::kind(),
		}
	}

	pub unsafe fn write(self, ptr: *mut u8) {
		match self {
			AnyValue::Byte(v) => i8::write(ptr, v),
			AnyValue::Short(v) => i16::write(ptr, v),
			AnyValue::Int(v) => i32::write(ptr, v),
			AnyValue::Long(v) => i64::write(ptr, v),
			AnyValue::Char(v) => u16::write(ptr, v),
			AnyValue::Float(v) => f32::write(ptr, v),
			AnyValue::Double(v) => f64::write(ptr, v),
			AnyValue::Boolean(v) => bool::write(ptr, v),
			AnyValue::Reference(v) => Reference::write(ptr, v),
		}
	}
	pub unsafe fn read(ptr: *mut u8, kind: Kind) -> Self {
		match kind {
			Kind::Byte => AnyValue::Byte(i8::read(ptr)),
			Kind::Short => AnyValue::Short(i16::read(ptr)),
			Kind::Int => AnyValue::Int(i32::read(ptr)),
			Kind::Long => AnyValue::Long(i64::read(ptr)),
			Kind::Char => AnyValue::Char(u16::read(ptr)),
			Kind::Float => AnyValue::Float(f32::read(ptr)),
			Kind::Double => AnyValue::Double(f64::read(ptr)),
			Kind::Boolean => AnyValue::Boolean(bool::read(ptr)),
			Kind::Reference => AnyValue::Reference(Reference::read(ptr)),
		}
	}
}

macro_rules! impl_direct {
	($VAR:ident $TY:ty) => {
		impl Value for $TY {
			fn kind() -> Kind {
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
	fn kind() -> Kind {
		Kind::Boolean
	}

	unsafe fn write(ptr: *mut u8, value: Self) {
		write(ptr, value as u8)
	}

	unsafe fn read(ptr: *mut u8) -> Self {
		read(ptr) != 0
	}
}

impl Value for Reference {
	fn kind() -> Kind {
		Kind::Reference
	}

	unsafe fn write(ptr: *mut u8, value: Self) {
		write_arr(ptr, {
			let x: *mut u8 = value.0;
			let i = x as usize;
			i.to_ne_bytes()
		})
	}

	unsafe fn read(ptr: *mut u8) -> Self {
		Reference(usize::from_ne_bytes(read_arr(ptr)) as *mut u8)
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
