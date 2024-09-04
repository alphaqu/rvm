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
#[test]
fn test() {
	let runtime = launch(1024);

	ConstantTests::test(&runtime).unwrap();
	//println!("{:?}", current_dir());
	//let result = JarClassSource::new(read("../../rt.zip").unwrap()).unwrap();
	//
	//let vec = result.try_load(&ObjectType::Object()).unwrap().unwrap();
	//
}

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
