use crate::{compile, launch};
use rvm_core::MethodDescriptor;
use rvm_macro::java_desc;
use rvm_runtime::{java_bind_method, java_binding, MethodIdentifier};

//#[test]
//fn basic_switch() {
//	let runtime = launch(1024, vec![]);
//
//	compile(
//		&runtime,
//		&[
//			("SwitchTest.java", include_str!("SwitchTest.java")),
//			("Assert.java", include_str!("../Assert.java")),
//		],
//	)
//	.unwrap();
//	let (binding, identifier) = java_binding!(
//		fn yes(value: bool) {
//			assert!(value);
//		}
//	);
//	println!("{identifier:?}");
//	runtime.bindings.write().insert(identifier, binding);
//
//	let java = java_bind_method!(runtime fn tests::switch_statement::SwitchTest:test());
//	let i = java();
//}
//
