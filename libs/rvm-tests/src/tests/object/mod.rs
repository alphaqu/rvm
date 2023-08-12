use crate::{compile, launch};
use rvm_runtime::java_bind_method;

#[test]
fn new_test() {
	launch(|runtime| {
		compile(
			&runtime,
			&[("ObjectTest.java", include_str!("ObjectTest.java"))],
		)
		.unwrap();

		runtime
			.class_loader
			.load_jar(include_bytes!("../../../../../rt.zip"), |v| {
				v == "java/lang/Object.class"
			})
			.unwrap();

		let java = java_bind_method!(runtime fn ObjectTest.simpleTest(value: i32) -> i32);
		let i = java(69);
		assert_eq!(i, 69)
	})
}
