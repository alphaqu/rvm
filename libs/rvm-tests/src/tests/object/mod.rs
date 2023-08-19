use rvm_core::MethodDesc;
use rvm_macro::java_desc;
use rvm_runtime::{java_bind_method, java_binding, MethodBinding, MethodIdentifier};

use crate::{compile, launch};

#[test]
fn interface() {
	launch(32 * 1024 * 1024, |runtime| {
		runtime
			.cl
			.load_jar(include_bytes!("../../../../../rt.zip"), |v| {
				v == "java/lang/Object.class"
			})
			.unwrap();

		compile(
			&runtime,
			&[
				("InterfaceTest.java", include_str!("InterfaceTest.java")),
				("Fruit.java", include_str!("Fruit.java")),
				("Assert.java", include_str!("../Assert.java")),
			],
		)
		.unwrap();

		let (binding, identifier) = java_binding!(
			fn yes(value: bool) {
				println!("{value}");
			}
		);
		runtime.bindings.write().insert(identifier, binding);
		let java = java_bind_method!(runtime fn tests::object::InterfaceTest:hi());
		let i = java();
	})
}

#[test]
fn extend_test() {
	launch(32 * 1024 * 1024, |runtime| {
		runtime
			.cl
			.load_jar(include_bytes!("../../../../../rt.zip"), |v| {
				v == "java/lang/Object.class"
			})
			.unwrap();

		compile(
			&runtime,
			&[
				("ObjectTest.java", include_str!("ObjectTest.java")),
				("ExtendTest.java", include_str!("ExtendTest.java")),
				("Assert.java", include_str!("../Assert.java")),
			],
		)
		.unwrap();

		let java = java_bind_method!(runtime fn tests::object::ExtendTest:create());
		let i = java();
	})
}

#[test]
fn new_test() {
	launch(32 * 1024 * 1024, |runtime| {
		compile(
			&runtime,
			&[("ObjectTest.java", include_str!("ObjectTest.java"))],
		)
		.unwrap();
		runtime
			.cl
			.load_jar(include_bytes!("../../../../../rt.zip"), |v| {
				v == "java/lang/Object.class"
			})
			.unwrap();

		let java =
			java_bind_method!(runtime fn tests::object::ObjectTest:simpleTest(value: i32) -> i32);
		let i = java(69);
		assert_eq!(i, 69)
	})
}

#[test]
fn gc_test() {
	launch(1024 * 4, |runtime| {
		compile(
			&runtime,
			&[("ObjectTest.java", include_str!("ObjectTest.java"))],
		)
		.unwrap();

		runtime
			.cl
			.load_jar(include_bytes!("../../../../../rt.zip"), |v| {
				v == "java/lang/Object.class"
			})
			.unwrap();

		let java = java_bind_method!(runtime fn ObjectTest:gcTest(value: i32) -> i32);
		let i = java(64);
		assert_eq!(i, 64)
	})
}
