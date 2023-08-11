use std::mem::transmute;

use rvm_core::{Kind, StackKind};
use rvm_object::DynValue;

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(C)]
pub enum StackValue {
	Int(i32),
	Float(f32),
	Long(i64),
	Double(f64),
	Reference(usize),
}

impl StackValue {
	pub fn to_int(self) -> i32 {
		match self {
			StackValue::Int(value) => value,
			_ => {
				panic!("Expected int, got {self:?}");
			}
		}
	}

	pub fn to_ref(self) -> usize {
		match self {
			StackValue::Reference(value) => value,
			_ => {
				panic!("Expected ref, got {self:?}");
			}
		}
	}
	pub fn kind(&self) -> StackKind {
		match self {
			StackValue::Int(_) => StackKind::Int,
			StackValue::Float(_) => StackKind::Float,
			StackValue::Long(_) => StackKind::Long,
			StackValue::Double(_) => StackKind::Double,
			StackValue::Reference(_) => StackKind::Reference,
		}
	}

	pub fn convert(self, kind: Kind) -> Option<DynValue> {
		match kind {
			Kind::Boolean => {
				if let StackValue::Int(value) = self {
					return Some(DynValue::Bool(value != 0));
				}
			}
			Kind::Byte => {
				if let StackValue::Int(value) = self {
					return Some(DynValue::Byte(value as i8));
				}
			}
			Kind::Short => {
				if let StackValue::Int(value) = self {
					return Some(DynValue::Short(value as i16));
				}
			}
			Kind::Int => {
				if let StackValue::Int(value) = self {
					return Some(DynValue::Int(value));
				}
			}
			Kind::Long => {
				if let StackValue::Long(value) = self {
					return Some(DynValue::Long(value));
				}
			}

			Kind::Float => {
				if let StackValue::Float(value) = self {
					return Some(DynValue::Float(value));
				}
			}
			Kind::Double => {
				if let StackValue::Double(value) = self {
					return Some(DynValue::Double(value));
				}
			}
			_ => {
				todo!("todo")
			}
		}

		None
	}

	pub fn to_dyn(self) -> DynValue {
		match self {
			StackValue::Int(value) => DynValue::Int(value),
			StackValue::Float(value) => DynValue::Float(value),
			StackValue::Long(value) => DynValue::Long(value),
			StackValue::Double(value) => DynValue::Double(value),
			StackValue::Reference(value) => {
				panic!()
			}
		}
	}
}

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct RawLocal(u32);

pub trait Local {
	const V: usize;
	fn to_raw(self) -> [RawLocal; Self::V];
	fn from_raw(value: [RawLocal; Self::V]) -> Self;
}

macro_rules! transmute_impl {
	($TY:ty, $ITY:ty) => {
		impl Local for $TY {
			const V: usize = 1;
			fn to_raw(self) -> [RawLocal; 1] {
				[RawLocal(unsafe { transmute::<$TY, $ITY>(self) } as u32)]
			}

			fn from_raw([value]: [RawLocal; 1]) -> Self {
				unsafe { transmute::<$ITY, $TY>(value.0 as $ITY) }
			}
		}
	};
}

transmute_impl!(i8, u8);
transmute_impl!(i16, u16);
transmute_impl!(i32, u32);
transmute_impl!(u16, u16);
transmute_impl!(f32, u32);

impl Local for bool {
	const V: usize = 1;

	fn to_raw(self) -> [RawLocal; 1] {
		[RawLocal(self as u8 as u32)]
	}

	fn from_raw([value]: [RawLocal; 1]) -> Self {
		value.0 != 0
	}
}

impl Local for f64 {
	const V: usize = 2;

	fn to_raw(self) -> [RawLocal; 2] {
		i64::to_raw(unsafe { transmute(self) })
	}

	fn from_raw(value: [RawLocal; 2]) -> Self {
		unsafe { transmute(i64::from_raw(value)) }
	}
}

impl Local for i64 {
	const V: usize = 2;

	fn to_raw(self) -> [RawLocal; 2] {
		let v: u64 = unsafe { transmute(self) };
		[RawLocal((v >> 32) as u32), RawLocal(v as u32)]
	}

	fn from_raw([v0, v1]: [RawLocal; 2]) -> Self {
		unsafe { transmute(((v0.0 as u64) << 32) | (v1.0 as u64)) }
	}
}
