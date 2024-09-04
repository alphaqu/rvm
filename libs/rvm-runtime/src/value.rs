use crate::conversion::JavaTyped;
use crate::object::Reference;
use crate::Runtime;
use rvm_core::{Kind, PrimitiveType, ResultUnwrapOrErr, Type};
use rvm_gc::GcRef;
use std::ptr::{read, write};
use std::sync::Arc;

pub trait Castable {
	fn cast_from(runtime: &Runtime, value: AnyValue) -> Self;
}

pub trait CastableExt<V> {
	fn cast_into(self, runtime: &Runtime) -> V;
}

impl<V: Castable> CastableExt<V> for AnyValue {
	fn cast_into(self, runtime: &Runtime) -> V {
		V::cast_from(runtime, self)
	}
}

pub trait Returnable {
	fn from_value(runtime: &Runtime, value: Option<AnyValue>) -> Self;
}

impl<C: Castable> Returnable for C {
	fn from_value(runtime: &Runtime, value: Option<AnyValue>) -> Self {
		let value = value.unwrap();
		C::cast_from(runtime, value)
	}
}

impl Returnable for () {
	fn from_value(_: &Runtime, value: Option<AnyValue>) -> Self {
		assert!(value.is_none());
		()
	}
}
impl<R: Returnable> Returnable for Option<R> {
	fn from_value(runtime: &Runtime, value: Option<AnyValue>) -> Self {
		value.map(|_| R::from_value(runtime, value))
	}
}

pub trait Value: Sized + Copy {
	fn kind() -> Kind;
	unsafe fn write(ptr: *mut UnionValue, value: Self);
	unsafe fn read(ptr: UnionValue) -> Self;
	unsafe fn cast_pointer(ptr: *mut UnionValue) -> *mut Self {
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

#[derive(Copy, Clone)]
#[repr(C)]
pub union UnionValue {
	pub byte: i8,
	pub short: i16,
	pub int: i32,
	pub long: i64,
	pub char: u16,
	pub float: f32,
	pub double: f64,
	pub boolean: bool,
	pub reference: Reference,
}

// Not implemented because AnyValue has no known value type

//impl JavaTyped for AnyValue {
//	//fn java_type(&self, runtime: &Runtime) -> Type {
//	//	match self {
//	//		AnyValue::Byte(_) => Type::Primitive(PrimitiveType::Byte),
//	//		AnyValue::Short(_) => Type::Primitive(PrimitiveType::Short),
//	//		AnyValue::Int(_) => Type::Primitive(PrimitiveType::Int),
//	//		AnyValue::Long(_) => Type::Primitive(PrimitiveType::Long),
//	//		AnyValue::Char(_) => Type::Primitive(PrimitiveType::Char),
//	//		AnyValue::Float(_) => Type::Primitive(PrimitiveType::Float),
//	//		AnyValue::Double(_) => Type::Primitive(PrimitiveType::Double),
//	//		AnyValue::Boolean(_) => Type::Primitive(PrimitiveType::Boolean),
//	//		AnyValue::Reference(reference) => reference.java_type(runtime),
//	//	}
//	//}
//
//	fn java_type_static() -> Type {
//		match self {
//			AnyValue::Byte(_) => Type::Primitive(PrimitiveType::Byte),
//			AnyValue::Short(_) => Type::Primitive(PrimitiveType::Short),
//			AnyValue::Int(_) => Type::Primitive(PrimitiveType::Int),
//			AnyValue::Long(_) => Type::Primitive(PrimitiveType::Long),
//			AnyValue::Char(_) => Type::Primitive(PrimitiveType::Char),
//			AnyValue::Float(_) => Type::Primitive(PrimitiveType::Float),
//			AnyValue::Double(_) => Type::Primitive(PrimitiveType::Double),
//			AnyValue::Boolean(_) => Type::Primitive(PrimitiveType::Boolean),
//			AnyValue::Reference(_) => Reference::java_type_static(),
//		}
//	}
//}

macro_rules! impl_from {
	($TY:ty, $KIND:ident) => {
		impl Castable for $TY {
			fn cast_from(_: &Runtime, value: AnyValue) -> Self {
				value.try_into().unwrap()
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
	pub fn ty(&self, runtime: &Runtime) -> Type {
		match self {
			AnyValue::Byte(_) => Type::Primitive(PrimitiveType::Byte),
			AnyValue::Short(_) => Type::Primitive(PrimitiveType::Short),
			AnyValue::Int(_) => Type::Primitive(PrimitiveType::Int),
			AnyValue::Long(_) => Type::Primitive(PrimitiveType::Long),
			AnyValue::Char(_) => Type::Primitive(PrimitiveType::Char),
			AnyValue::Float(_) => Type::Primitive(PrimitiveType::Float),
			AnyValue::Double(_) => Type::Primitive(PrimitiveType::Double),
			AnyValue::Boolean(_) => Type::Primitive(PrimitiveType::Boolean),
			AnyValue::Reference(reference) => reference.ty(runtime),
		}
	}

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

	pub unsafe fn write(self, ptr: *mut UnionValue) {
		match self {
			AnyValue::Byte(v) => ptr.write(UnionValue { byte: v }),
			AnyValue::Short(v) => ptr.write(UnionValue { short: v }),
			AnyValue::Int(v) => ptr.write(UnionValue { int: v }),
			AnyValue::Long(v) => ptr.write(UnionValue { long: v }),
			AnyValue::Char(v) => ptr.write(UnionValue { char: v }),
			AnyValue::Float(v) => ptr.write(UnionValue { float: v }),
			AnyValue::Double(v) => ptr.write(UnionValue { double: v }),
			AnyValue::Boolean(v) => ptr.write(UnionValue { boolean: v }),
			AnyValue::Reference(v) => ptr.write(UnionValue { reference: v }),
		}
	}
	pub unsafe fn read(ptr: UnionValue, kind: Kind) -> Self {
		match kind {
			Kind::Byte => AnyValue::Byte(ptr.byte),
			Kind::Short => AnyValue::Short(ptr.short),
			Kind::Int => AnyValue::Int(ptr.int),
			Kind::Long => AnyValue::Long(ptr.long),
			Kind::Char => AnyValue::Char(ptr.char),
			Kind::Float => AnyValue::Float(ptr.float),
			Kind::Double => AnyValue::Double(ptr.double),
			Kind::Boolean => AnyValue::Boolean(ptr.boolean),
			Kind::Reference => AnyValue::Reference(ptr.reference),
		}
	}
}

macro_rules! impl_value {
	($VAR:ident $FIELD:ident $TY:ty) => {
		impl Value for $TY {
			fn kind() -> Kind {
				Kind::$VAR
			}

			unsafe fn write(ptr: *mut UnionValue, value: Self) {
				(*ptr).$FIELD = value;
			}

			unsafe fn read(ptr: UnionValue) -> Self {
				ptr.$FIELD
			}
		}
	};
}
impl_value!(Boolean boolean bool);
impl_value!(Byte byte i8);
impl_value!(Short short i16);
impl_value!(Int int i32);
impl_value!(Long long i64);
impl_value!(Char char u16);
impl_value!(Float float f32);
impl_value!(Double double f64);
impl_value!(Reference reference Reference);

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
