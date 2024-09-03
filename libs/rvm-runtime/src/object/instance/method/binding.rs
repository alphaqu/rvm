use crate::{
	AnyValue, Class, FromJava, FromJavaMulti, JavaTyped, JavaTypedMulti, MethodIdentifier,
	Reference, Runtime, ToJava, ToJavaMulti,
};
use ahash::HashMap;
use parking_lot::RwLock;
use rvm_core::{Id, Kind, MethodDescriptor, Type};
use std::collections::hash_map::Entry;
use std::fmt::Write;
use std::mem::transmute;
use std::sync::Arc;
use tracing::warn;

#[macro_export]
macro_rules! java_binding {
		(fn $NAME:ident($($P_NAME:ident: $P_TY:ty),*) $(->  $RET:ty)? $BLOCK:block) => {
			unsafe {
				extern "C" fn $NAME($($P_NAME: $P_TY),*) $(-> $RET)?
					$BLOCK


				let desc = java_desc!(fn($($P_TY),*) $(-> $RET)?);
				($crate::MethodBinding::new(
					std::mem::transmute($NAME as usize),
					MethodDescriptor::parse(desc).unwrap(),
				), MethodIdentifier {
					name: stringify!($NAME).to_string(),
					descriptor: desc.to_string(),
				})
			}
		};
	}
#[cfg(test)]
mod tests {
	use rvm_macro::java_desc;

	use super::*;

	#[test]
	fn basic() {
		unsafe {
			let binding = java_binding!(|hi: i32, another: i32| -> i32 {
				println!("{hi} {another}");
				69420
			});

			// from
			//fn(hi: i32, another: i32) -> i32 {
			//	println!("{hi} {another_hi}");
			//	69420
			//}

			// to
			//let binding = unsafe {
			//	#[no_mangle]
			//	extern "C" fn binding(hi: i32, another_hi: i32) -> i32 {
			//		println!("{hi} {another_hi}");
			//		69420
			//	}
			//
			//	MethodBinding::new(
			//		transmute(binding as usize),
			//		MethodDesc::parse(java_desc!(fn(i32, i32) -> i32)).unwrap(),
			//	)
			//};

			let i = 0;
			let value = binding
				.call(&[AnyValue::Int(32), AnyValue::Int(420)])
				.unwrap();
			let i2 = 0;
			println!("{i} {i2} {value:?}")
		}
	}
}
