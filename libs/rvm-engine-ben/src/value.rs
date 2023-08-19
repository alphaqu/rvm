use rvm_core::{Kind, StackKind};
use rvm_runtime::{AnyValue, Reference};
use std::fmt::{Display, Formatter};
use std::mem::transmute;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum StackValue {
	Int(i32),
	Float(f32),
	Long(i64),
	Double(f64),
	Reference(Reference),
}

impl Display for StackValue {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			StackValue::Int(v) => write!(f, "i{v}"),
			StackValue::Float(v) => write!(f, "f{v}"),
			StackValue::Long(v) => write!(f, "l{v}"),
			StackValue::Double(v) => write!(f, "d{v}"),
			StackValue::Reference(v) => write!(f, "{v:?}"),
		}
	}
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

	pub fn to_ref(self) -> Reference {
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

	pub fn category(&self) -> u8 {
		match self {
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

	pub fn from_dyn(value: AnyValue) -> StackValue {
		match value {
			AnyValue::Byte(value) => StackValue::Int(value as i32),
			AnyValue::Short(value) => StackValue::Int(value as i32),
			AnyValue::Int(value) => StackValue::Int(value),
			AnyValue::Long(value) => StackValue::Long(value),
			AnyValue::Float(value) => StackValue::Float(value),
			AnyValue::Double(value) => StackValue::Double(value),
			AnyValue::Boolean(value) => StackValue::Int(value as u8 as i32),
			AnyValue::Reference(value) => StackValue::Reference(value),
			_ => todo!(),
		}
	}

	pub fn convert(self, kind: Kind) -> Option<AnyValue> {
		match kind {
			Kind::Boolean => {
				if let StackValue::Int(value) = self {
					return Some(AnyValue::Boolean(value != 0));
				}
			}
			Kind::Byte => {
				if let StackValue::Int(value) = self {
					return Some(AnyValue::Byte(value as i8));
				}
			}
			Kind::Short => {
				if let StackValue::Int(value) = self {
					return Some(AnyValue::Short(value as i16));
				}
			}
			Kind::Int => {
				if let StackValue::Int(value) = self {
					return Some(AnyValue::Int(value));
				}
			}
			Kind::Long => {
				if let StackValue::Long(value) = self {
					return Some(AnyValue::Long(value));
				}
			}

			Kind::Float => {
				if let StackValue::Float(value) = self {
					return Some(AnyValue::Float(value));
				}
			}
			Kind::Double => {
				if let StackValue::Double(value) = self {
					return Some(AnyValue::Double(value));
				}
			}
			Kind::Reference => {
				if let StackValue::Reference(value) = self {
					return Some(AnyValue::Reference(value));
				}
			}
			_ => {
				todo!("todo")
			}
		}

		None
	}

	pub fn to_dyn(self) -> AnyValue {
		match self {
			StackValue::Int(value) => AnyValue::Int(value),
			StackValue::Float(value) => AnyValue::Float(value),
			StackValue::Long(value) => AnyValue::Long(value),
			StackValue::Double(value) => AnyValue::Double(value),
			StackValue::Reference(value) => AnyValue::Reference(value),
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
impl Local for Reference {
	const V: usize = 2;

	fn to_raw(self) -> [RawLocal; 2] {
		let v: u64 = self.0 as usize as u64;
		[RawLocal((v >> 32) as u32), RawLocal(v as u32)]
	}

	fn from_raw([v0, v1]: [RawLocal; 2]) -> Self {
		unsafe {
			let value = ((v0.0 as u64) << 32) | (v1.0 as u64);
			Reference(value as usize as *mut u8)
		}
	}
}

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
