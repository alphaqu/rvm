use crate::engine::ThreadConfig;
use crate::{AnyValue, MethodIdentifier, Returnable, Runtime};
use rvm_core::ObjectType;
use std::marker::PhantomData;
use std::sync::Arc;

pub struct TestMethodCall<R: Returnable> {
	pub ty: ObjectType,
	pub desc: MethodIdentifier,
	pub parameters: Vec<AnyValue>,
	pub _returns: PhantomData<R>,
}

impl<R: Returnable> TestMethodCall<R> {
	pub fn call(self, runtime: &Arc<Runtime>) -> R {
		let thread = runtime.engine.create_thread(
			runtime.clone(),
			ThreadConfig {
				name: "run".to_string(),
			},
		);
		thread.run(self.ty, self.desc, self.parameters);
		let value = thread.join();
		let dyn_value = value.expect("Thread failed to run");
		R::from_value(runtime, dyn_value)
	}
}

#[macro_export]
macro_rules! java_bind_method {
    ($runtime:ident fn $class:path:$method:ident($($name:ident: $pty:ty),*) $(-> $ret:ty)?) => {
		 {
			 let value = |$($name: $pty),*| $(-> $ret)? {
			// just so jetbrains ides have colors
			fn $method() {}
			let thread = $runtime.engine.create_thread($runtime.clone(), rvm_runtime::engine::ThreadConfig {
				name: "run".to_string(),
			});


			thread.run(
				rvm_core::ObjectType(::core::stringify!($class).to_string().replace("::", "/")),
				rvm_runtime::MethodIdentifier {
					name: ::core::stringify!($method).to_string(),
					descriptor: rvm_macro::java_desc!(fn($($pty),*) $(-> $ret)?).to_string(),
				},
				vec![
					$(
						<rvm_runtime::AnyValue as TryFrom<$pty>>::try_from($name).unwrap()
					),*
				]
			);
			let value = thread.join();
			let dyn_value = value.expect("Thread failed to run");
			$(
				let dyn_value = dyn_value.expect("Void return");
				<rvm_runtime::AnyValue as TryInto<$ret>>::try_into(dyn_value).expect("failed to convert")
			)?
		};
			 value
		 }
	};
}

#[macro_export]
macro_rules! bind_return {
	() => {
		()
	};
	($RETURNS:ty) => {
		$RETURNS
	};
}

#[macro_export]
macro_rules! bind {
    ($PACKAGE:literal { $($CLASS:ident {
		$($METHOD:ident($($PARAM_NAME:ident: $PARAM:ty),*) $(-> $RETURNS:ty)?),*
	}),* }) => {
		$(
			#[allow(unused, non_snake_case)]
			impl $CLASS {
				$(
				pub fn $METHOD(runtime: &Arc<rvm_runtime::Runtime>) -> impl Fn($($PARAM),*) $(-> $RETURNS)? {
					let runtime = runtime.clone();
					let mut class = $PACKAGE.to_string();
					class.push('/');
					class.push_str(stringify!($CLASS));

					move |$($PARAM_NAME: $PARAM),*| {
						let call = rvm_runtime::prelude::TestMethodCall::<rvm_runtime::bind_return!($($RETURNS)?)> {
							ty: rvm_core::ObjectType(class.clone()),
							desc: rvm_runtime::MethodIdentifier {
								name: stringify!($METHOD).to_string(),
								descriptor: rvm_macro::java_desc!(fn($($PARAM),*) $(-> $RETURNS)?).to_string(),
							},
							parameters: vec![$($PARAM_NAME.into()),*],
							_returns: Default::default(),
						};
						call.call(&runtime)
					}
				}
				)*
			}
		)*

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
