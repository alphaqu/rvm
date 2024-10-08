use bytemuck::Zeroable;
use derive_more::From;
use rvm_core::{CastKindError, Kind, StackKind};
use rvm_runtime::{AnyValue, Reference, ReferenceKind};
use std::fmt::{Display, Formatter};

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, From)]
pub enum StackValue {
	Char(u16),
	Int(i32),
	Float(f32),
	Long(i64),
	Double(f64),
	Reference(Reference),
}

unsafe impl Zeroable for StackValue {}

impl Display for StackValue {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			StackValue::Int(v) => write!(f, "{v}"),
			StackValue::Float(v) => write!(f, "{v}F"),
			StackValue::Char(char) => write!(f, "{}C", char),
			StackValue::Long(v) => write!(f, "{v}L"),
			StackValue::Double(v) => write!(f, "{v}.0"),
			StackValue::Reference(v) => {
				write!(f, "{v:?}")
			}
		}
	}
}

macro_rules! to_impl {
	($($METHOD_NAME:ident $KIND:ident $TY:ty),*) => {
		impl StackValue {
			$(
				pub fn $METHOD_NAME(self) -> Result<$TY, CastKindError> {
					match self {
						StackValue::$KIND(value) => Ok(value),
						_ => Err(CastKindError {
							expected: Kind::$KIND,
							found: self.kind().kind(),
						}),
					}
				}
			)*
		}

		$(
			impl TryInto<$TY> for StackValue {
				type Error = CastKindError;

				fn try_into(self) -> Result<$TY, Self::Error> {
					self.$METHOD_NAME()
				}
			}
		)*
	};
}

to_impl!(
	to_int Int i32,
	to_long Long i64,
	to_float Float f32,
	to_double Double f64,
	to_ref Reference Reference
);
impl StackValue {
	pub fn kind(&self) -> StackKind {
		match self {
			StackValue::Char(_) => StackKind::Char,
			StackValue::Int(_) => StackKind::Int,
			StackValue::Float(_) => StackKind::Float,
			StackValue::Long(_) => StackKind::Long,
			StackValue::Double(_) => StackKind::Double,
			StackValue::Reference(_) => StackKind::Reference,
		}
	}

	pub fn category(&self) -> u8 {
		match self {
			StackValue::Char(_) => 1,
			StackValue::Int(_) => 1,
			StackValue::Float(_) => 1,
			StackValue::Reference(_) => 1,
			StackValue::Long(_) => 2,
			StackValue::Double(_) => 2,
		}
	}

	pub fn category_1(&self) -> bool {
		self.category() == 1
	}

	pub fn category_2(&self) -> bool {
		self.category() == 2
	}

	pub fn from_any(value: AnyValue) -> StackValue {
		match value {
			AnyValue::Byte(value) => StackValue::Int(value as i32),
			AnyValue::Short(value) => StackValue::Int(value as i32),
			AnyValue::Int(value) => StackValue::Int(value),
			AnyValue::Long(value) => StackValue::Long(value),
			AnyValue::Float(value) => StackValue::Float(value),
			AnyValue::Double(value) => StackValue::Double(value),
			AnyValue::Boolean(value) => StackValue::Int(value as u8 as i32),
			AnyValue::Reference(value) => StackValue::Reference(value),
			AnyValue::Char(value) => StackValue::Char(value),
		}
	}

	pub fn convert(self, kind: Kind) -> Result<AnyValue, CastKindError> {
		match kind {
			Kind::Boolean => {
				if let StackValue::Int(value) = self {
					return Ok(AnyValue::Boolean(value != 0));
				}
			}
			Kind::Byte => {
				if let StackValue::Int(value) = self {
					return Ok(AnyValue::Byte(value as i8));
				}
			}
			Kind::Short => {
				if let StackValue::Int(value) = self {
					return Ok(AnyValue::Short(value as i16));
				}
			}
			Kind::Int => {
				if let StackValue::Int(value) = self {
					return Ok(AnyValue::Int(value));
				}
			}
			Kind::Long => {
				if let StackValue::Long(value) = self {
					return Ok(AnyValue::Long(value));
				}
			}

			Kind::Float => {
				if let StackValue::Float(value) = self {
					return Ok(AnyValue::Float(value));
				}
			}
			Kind::Double => {
				if let StackValue::Double(value) = self {
					return Ok(AnyValue::Double(value));
				}
			}
			Kind::Reference => {
				if let StackValue::Reference(value) = self {
					return Ok(AnyValue::Reference(value));
				}
			}
			_ => {
				todo!("todo")
			}
		}

		Err(CastKindError {
			expected: kind,
			found: self.kind().kind(),
		})
	}

	pub fn to_any(self) -> AnyValue {
		match self {
			StackValue::Char(value) => AnyValue::Char(value),
			StackValue::Int(value) => AnyValue::Int(value),
			StackValue::Float(value) => AnyValue::Float(value),
			StackValue::Long(value) => AnyValue::Long(value),
			StackValue::Double(value) => AnyValue::Double(value),
			StackValue::Reference(value) => AnyValue::Reference(value),
		}
	}
}
//#[derive(Copy, Clone)]
// #[repr(transparent)]
// pub struct RawLocal(u32);
//
// pub trait Local {
// 	const V: usize;
// 	fn to_raw(self) -> [RawLocal; Self::V];
// 	fn from_raw(value: [RawLocal; Self::V]) -> Self;
// }
//
// macro_rules! transmute_impl {
// 	($TY:ty, $ITY:ty) => {
// 		impl Local for $TY {
// 			const V: usize = 1;
// 			fn to_raw(self) -> [RawLocal; 1] {
// 				[RawLocal(unsafe { transmute::<$TY, $ITY>(self) } as u32)]
// 			}
//
// 			fn from_raw([value]: [RawLocal; 1]) -> Self {
// 				unsafe { transmute::<$ITY, $TY>(value.0 as $ITY) }
// 			}
// 		}
// 	};
// }
//
// transmute_impl!(i8, u8);
// transmute_impl!(i16, u16);
// transmute_impl!(i32, u32);
// transmute_impl!(u16, u16);
// transmute_impl!(f32, u32);
// impl Local for Reference {
// 	const V: usize = 2;
//
// 	fn to_raw(self) -> [RawLocal; 2] {
// 		let v: u64 = self.0 as usize as u64;
// 		[RawLocal((v >> 32) as u32), RawLocal(v as u32)]
// 	}
//
// 	fn from_raw([v0, v1]: [RawLocal; 2]) -> Self {
// 		unsafe {
// 			let value = ((v0.0 as u64) << 32) | (v1.0 as u64);
// 			Reference(value as usize as *mut u8)
// 		}
// 	}
// }
//
// impl Local for bool {
// 	const V: usize = 1;
//
// 	fn to_raw(self) -> [RawLocal; 1] {
// 		[RawLocal(self as u8 as u32)]
// 	}
//
// 	fn from_raw([value]: [RawLocal; 1]) -> Self {
// 		value.0 != 0
// 	}
// }
//
// impl Local for f64 {
// 	const V: usize = 2;
//
// 	fn to_raw(self) -> [RawLocal; 2] {
// 		i64::to_raw(unsafe { transmute(self) })
// 	}
//
// 	fn from_raw(value: [RawLocal; 2]) -> Self {
// 		unsafe { transmute(i64::from_raw(value)) }
// 	}
// }
//
// impl Local for i64 {
// 	const V: usize = 2;
//
// 	fn to_raw(self) -> [RawLocal; 2] {
// 		let v: u64 = unsafe { transmute(self) };
// 		[RawLocal((v >> 32) as u32), RawLocal(v as u32)]
// 	}
//
// 	fn from_raw([v0, v1]: [RawLocal; 2]) -> Self {
// 		unsafe { transmute(((v0.0 as u64) << 32) | (v1.0 as u64)) }
// 	}
// }
