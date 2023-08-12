use crate::{compile, launch};
use rvm_runtime::java_bind_method;

#[test]
fn newTest() {
	launch(|runtime| {
		compile(
			&runtime,
			&[("ObjectTest.java", include_str!("ObjectTest.java"))],
		)
		.unwrap();

		let java = java_bind_method!(runtime fn ObjectTest.simpleTest(value: i32) -> i32);

		let i = java(69);
	})
}
