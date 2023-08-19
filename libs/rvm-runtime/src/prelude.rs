#[macro_export]
macro_rules! java_descriptor {
	(()) => {"V"};
	(bool) => {"Z"};
	(i8) => {"B"};
	(i16) => {"S"};
	(i32) => {"I"};
	(f32) => {"F"};
	(i64) => {"J"};
	(f64) => {"D"};
	(Array<$param:tt>) => {
		::core::concat!("[", $crate::java_descriptor!($param))
	};
	(fn($($param:tt),*)) => {
		$crate::java_descriptor!(fn($($param),*) -> ())
	};
	(fn($($param:tt),*) -> $ret:tt) => {
		::core::concat!("(", $($crate::java_descriptor!($param),)* ")", $crate::java_descriptor!($ret))
	};
}

#[macro_export]
macro_rules! java_bind_method {
    ($runtime:ident fn $class:ident.$method:ident($($name:ident: $pty:ty),*) $(-> $ret:ty)?) => {
		 {
			 let value = |$($name: $pty),*| $(-> $ret)? {
			// just so jetbrains ides have colors
			struct $class {}
			fn $method() {}
			let thread = $runtime.engine.create_thread($runtime.clone(), rvm_runtime::engine::ThreadConfig {
				name: "run".to_string(),
			});

			thread.run(
				rvm_core::ObjectType(::core::stringify!($class).to_string()),
				rvm_object::MethodIdentifier {
					name: ::core::stringify!($method).to_string(),
					descriptor: rvm_macro::java_desc!(fn($($pty),*) $(-> $ret)?).to_string(),
				},
				vec![
					$(
						<rvm_object::DynValue as TryFrom<$pty>>::try_from($name).unwrap()
					),*
				]
			);
			let value = thread.join();

			$(
				let dyn_value = value.expect("Thread failed to run").expect("Void return");
				<rvm_object::DynValue as TryInto<$ret>>::try_into(dyn_value).expect("failed to convert")
			)?
		};
			 value
		 }
	};
}

// /// A multi-purpose macro to lookup Rust types into Java equivalents or mirrors
// ///
// /// To convert Rust primitives to Java descriptors:
// /// ```
// /// use rvm_runtime::java;
// ///
// /// assert_eq!(java!(()), "V");
// /// assert_eq!(java!(i32), "I");
// /// assert_eq!(java!(f64), "D");
// /// ```
// ///
// /// To create method descriptors
// /// ```
// /// use rvm_runtime::java;
// ///
// /// assert_eq!(java!(descriptor ()), "()V");
// /// assert_eq!(java!(descriptor i32, i32, i32), "(II)I");
// /// ```
// ///
// /// To lookup and compile constant methods within a [`Runtime`]
// /// ```
// /// use std::pin::Pin;
// /// use rvm_runtime::{java, Runtime};
// ///
// /// fn create_adder(runtime: &Pin<&Runtime>) -> impl Fn(i32, i32) -> i32 {
// /// 	let compiled = unsafe { java!(compile runtime, fn Math.sum(i32, i32) -> i32) };
// ///
// /// 	move |a, b| {
// /// 		unsafe { compiled(a, b) }
// /// 	}
// /// }
// /// ```
// ///
// /// [`Runtime`]: crate::Runtime
// #[macro_export]
// macro_rules! java {
// 	(()) => {"V"};
// 	(bool) => {"Z"};
// 	(i8) => {"B"};
// 	(i16) => {"S"};
// 	(i32) => {"I"};
// 	(f32) => {"F"};
// 	(i64) => {"J"};
// 	(f64) => {"D"};
//
// 	(descriptor $return:tt) => {
// 		::core::concat!("()", $crate::java!($return))
// 	};
// 	(descriptor $return:tt, $($param:tt),+) => {
// 		::core::concat!("(", $($crate::java!($param)),+, ")", $crate::java!($return))
// 	};
// 	(run fn $class:ident.$method:ident($($p:expr: $pty:tt),+) -> $return:tt) => {
// 			let thread = runtime.engine.create_thread(ThreadConfig {
// 				name: "run".to_string(),
// 			});
//
// 			thread.run(
// 				ObjectType {
// 					name: ::core::stringify!($class).to_string(),
// 				},
// 				MethodIdentifier {
// 					name: ::core::stringify!($method).to_string(),
// 					descriptor: $crate::java!(descriptor $return, $($pty),+).to_string(),
// 				},
// 				vec![
// 					$(
//
// 					),*
// 				]
// 			);
// 			let value = thread.join().unwrap();
// 			match value {
// 				DynValue::Int(o) => o,
// 				_ =>  {
// 					panic!("invalid")
// 				}
// 			}
// 	}
//
// 	(compile $runtime:expr, fn $class:ident.$name:ident() -> $return:tt) => {
// 		::core::mem::transmute::<_, unsafe extern "C" fn() -> $return>($crate::Runtime::compile_method($runtime, ::core::stringify!($class), ::core::stringify!($name), $crate::java!(descriptor $return)))
// 	};
//
// 	(compile $runtime:expr, fn $class:ident.$name:ident($($param:tt),+) -> $return:tt) => {
// 		::core::mem::transmute::<_, unsafe extern "C" fn($($param),+) -> $return>($crate::Runtime::compile_method($runtime, ::core::stringify!($class), ::core::stringify!($name), $crate::java!(descriptor $return, $($param),+)))
// 	};
// }
