use crate::bindings::tests::constants::ConstantTests;
use crate::launch;
use rvm_core::ObjectType;
use rvm_runtime::{AnyInstance, Instance, InstanceBinding};
use std::mem::transmute;
use std::ops::Deref;

pub struct Hello {
	field: f32,
	// base: Dog
}

pub mod test {
	use crate::java::lang::Object;

	impl Object {}
}

pub mod java {
	pub mod test {
		use super::super::java::lang;
	}
	pub mod lang {
		pub struct Object;
	}
}
impl Hello {
	pub fn run(self: &Instance<Hello>) {}
}

pub struct HelloStatic {
	static_field: f32,
}

impl InstanceBinding for Hello {
	fn ty() -> ObjectType {
		todo!()
	}

	fn bind(instance: &AnyInstance) -> Self {
		todo!()
	}
}

macro_rules! test_const {
	($TEST_NAME:ident $EXPECTED:literal) => {
		#[test]
		fn $TEST_NAME() {
			let runtime = launch(128);

			assert_eq!(ConstantTests::$TEST_NAME(&runtime).unwrap(), $EXPECTED);
		}
	};
}

test_const!(iconst_m1 - 1);
test_const!(iconst_0 0);
test_const!(iconst_1 1);
test_const!(iconst_2 2);
test_const!(iconst_3 3);
test_const!(iconst_4 4);
test_const!(iconst_5 5);
test_const!(lconst_0 0);
test_const!(lconst_1 1);
test_const!(fconst_0 0.0);
test_const!(fconst_1 1.0);
test_const!(fconst_2 2.0);
test_const!(dconst_0 0.0);
test_const!(dconst_1 1.0);
test_const!(bipush 12);
test_const!(sipush 244);
test_const!(ldc 696969);
test_const!(ldc_2 6969695232535242342i64);

//macro_rules! java_bind {
// 	(package $PACKAGE:path; pub struct $TY:ident {
// 		$(
// 			pub fn $FUNC_NAME:ident($($PARAM_NAME:ident: $PARAM_TYPE: ty),*) $(-> $RETURNS:ty)? $FUNC_BLOCK:block
// 		)*
// 	}) => {
//
// 		pub struct $TY;
//
// 		#[allow(unused)]
// 		impl $TY {
// 			$(
//
// 				pub fn $FUNC_NAME($($PARAM_NAME: $PARAM_TYPE),*) $(-> $RETURNS)? {
// 					$FUNC_BLOCK
// 				}
// 			)*
// 		}
// 	};
// }
//
// java_bind! {
// 	package tests::constants;
//
// 	pub struct ConstantTests {
// 		pub fn native_function(input: i32) -> i32 {
// 			input
// 		}
// 	}
// }
