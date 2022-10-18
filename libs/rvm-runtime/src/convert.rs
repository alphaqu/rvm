use crate::{Ref, Runtime};
use anyways::Result;
use crate::object::Value;

pub trait ToJava {
    fn to_java(&self, runtime: &Runtime) -> Result<Value>;
}

pub trait FromJava: Sized {
    fn from_java(value: Value, runtime: &Runtime) -> Result<Self>;
}

macro_rules! impl_through_deref {
    ($([$TYPE:ty, $VARIANT:ident]),*) => {
	    $(
		    impl ToJava for $TYPE {
				fn to_java(&self, _: &Runtime) -> Result<Value> {
					Ok(Value::$VARIANT(*self))
				}
			}

			impl FromJava for $TYPE {
				fn from_java(value: Value, _: &Runtime) -> Result<Self> {
					if let Value::$VARIANT(byte) = value {
						Ok(byte)
					} else {
						Err(anyways::audit::Audit::new(format!("Value type is not correct for {}", std::any::type_name::<Self>())))
					}
				}
			}
	    )*
    };
}

impl_through_deref!(
    [bool, Boolean],
    [i8, Byte],
    [i16, Short],
    [i32, Int],
    [i64, Long],
    [u16, Char],
    [f32, Float],
    [f64, Double],
    [Ref, Reference]
);
