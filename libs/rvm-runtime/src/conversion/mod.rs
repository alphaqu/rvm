mod multi;

use crate::{AnyValue, Reference, Vm};
pub use multi::*;
use rvm_core::{CastKindError, Kind, ObjectType, PrimitiveType, Type};
use std::sync::Arc;

//pub trait ToJavaType {
//	fn java_type(&self, runtime: &Runtime) -> Type;
//	fn java_type_static() -> Type;
//}
pub trait JavaTyped {
	fn java_type() -> Type;
}
pub trait ToJava: Sized {
	fn to_java(self, runtime: &Vm) -> eyre::Result<AnyValue>;
}

pub trait FromJava: Sized {
	fn from_java(value: AnyValue, runtime: &Vm) -> eyre::Result<Self>;
}

macro_rules! impl_simple {
	($KIND:ident $($JAVA_TY:block)? $TY:ty) => {
		impl ToJava for $TY {
			fn to_java(self, _: &Vm) -> eyre::Result<AnyValue> {
				Ok(AnyValue::$KIND(self))
			}
		}
		impl FromJava for $TY {
			fn from_java(value: AnyValue, _: &Vm) -> eyre::Result<Self> {
				match value {
					AnyValue::$KIND(v) => Ok(v),
					_ => Err(CastKindError {
						expected: Kind::$KIND,
						found: value.kind(),
					}
					.into()),
				}
			}
		}

		$(
			impl JavaTyped for $TY {
				//fn java_type(&self, _: &Runtime) -> Type {
				//	Self::java_type_static()
				//}

				fn java_type() -> Type {
					$JAVA_TY.into()
				}
			}
		)?
	};
}

impl_simple!(Byte {PrimitiveType::Byte} i8);
impl_simple!(Short {PrimitiveType::Short} i16);
impl_simple!(Int {PrimitiveType::Int} i32);
impl_simple!(Long {PrimitiveType::Long} i64);
impl_simple!(Char {PrimitiveType::Char} u16);
impl_simple!(Float {PrimitiveType::Float} f32);
impl_simple!(Double {PrimitiveType::Double} f64);
impl_simple!(Boolean {PrimitiveType::Boolean} bool);
impl_simple!(Reference Reference);
