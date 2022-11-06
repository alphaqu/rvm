/// A multi-purpose macro to lookup Rust types into Java equivalents or mirrors
///
/// To convert Rust primitives to Java descriptors:
/// ```
/// use rvm_runtime::java;
///
/// assert_eq!(java!(()), "V");
/// assert_eq!(java!(i32), "I");
/// assert_eq!(java!(f64), "D");
/// ```
///
/// To create method descriptors
/// ```
/// use rvm_runtime::java;
///
/// assert_eq!(java!(descriptor ()), "()V");
/// assert_eq!(java!(descriptor i32, i32, i32), "(II)I");
/// ```
///
/// To lookup and compile constant methods within a [`Runtime`]
/// ```
/// use std::pin::Pin;
/// use rvm_runtime::{java, Runtime};
///
/// |runtime: &Pin<&Runtime>, a: i32, b: i32| {
/// 	unsafe { java!(compile runtime, fn Math.sum(i32, i32) -> i32)(a, b) }
/// };
/// ```
///
/// [`Runtime`]: crate::Runtime
#[macro_export]
macro_rules! java {
	(()) => {"V"};
    (bool) => {"Z"};
    (i8) => {"B"};
    (i16) => {"S"};
    (i32) => {"I"};
    (f32) => {"F"};
    (i64) => {"J"};
    (f64) => {"D"};

	(descriptor $return:tt) => {
		::std::concat!("()", $crate::java!($return))
	};
	(descriptor $return:tt, $($param:tt),+) => {
		::std::concat!("(", $($crate::java!($param)),+, ")", $crate::java!($return))
	};

	(compile $runtime:expr, fn $class:ident.$name:ident() -> $return:tt) => {
		::std::mem::transmute::<_, unsafe extern "C" fn() -> $return>($crate::Runtime::compile_method($runtime, ::std::stringify!($class), ::std::stringify!($name), $crate::java!(descriptor $return)))
	};

	(compile $runtime:expr, fn $class:ident.$name:ident($($param:tt),+) -> $return:tt) => {
		::std::mem::transmute::<_, unsafe extern "C" fn($($param),+) -> $return>($crate::Runtime::compile_method($runtime, ::std::stringify!($class), ::std::stringify!($name), $crate::java!(descriptor $return, $($param),+)))
	};
}
