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
		concat!("()", java!($return))
	};
	(descriptor $return:tt, $($param:tt),+) => {
		concat!("(", $(java!($param)),+, ")", java!($return))
	};

	(compile $runtime:expr, fn $class:ident.$name:ident() -> $return:tt) => {
		::std::mem::transmute::<_, unsafe extern "C" fn() -> $return>($runtime.compile_method(stringify!($class), stringify!($name), java!(descriptor $return)))
	};

	(compile $runtime:expr, fn $class:ident.$name:ident($($param:tt),+) -> $return:tt) => {
		::std::mem::transmute::<_, unsafe extern "C" fn($($param),+) -> $return>($runtime.compile_method(stringify!($class), stringify!($name), java!(descriptor $return, $($param),+)))
	};
}
