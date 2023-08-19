use crate::{compile, launch};
use rvm_core::Kind;
use rvm_object::Array;
use rvm_runtime::java_bind_method;

#[test]
fn primitive() {
	launch(32 * 1024 * 1024, |runtime| {
		let short = runtime.gc.lock().allocate_array(Kind::Int, 3).unwrap();
		let mut array = Array::new(short);

		assert_eq!(array.length(), 3);
		assert_eq!(array.get(0), Some(0));
		array.set(0, 69);
		assert_eq!(array.length(), 3);
		assert_eq!(array.get(0), Some(69));
		assert_eq!(array.get(3), None);
		array.set(0, 420);
		assert_eq!(array.get(0), Some(420));

		assert_eq!(array.get(1), Some(0));
		array.set(1, 420);
		assert_eq!(array.get(1), Some(420));
	})
}
#[test]
fn setter() {
	launch(32 * 1024 * 1024, |runtime| {
		compile(
			&runtime,
			&[("ArrayTest.java", include_str!("ArrayTest.java"))],
		)
		.unwrap();

		runtime
			.class_loader
			.load_jar(include_bytes!("../../../../../rt.zip"), |v| {
				v == "java/lang/Object.class"
			})
			.unwrap();

		let java_set = java_bind_method!(runtime fn ArrayTest.setValue(array: Array<i32>, index: i32, value: i32));
		let java_get =
			java_bind_method!(runtime fn ArrayTest.getValue(array: Array<i32>, index: i32) -> i32);

		let short = runtime.gc.lock().allocate_array(Kind::Int, 3).unwrap();
		let mut array = Array::new(short);

		assert_eq!(array.get(0), Some(0));

		java_set(array, 0, 69);
		assert_eq!(array.get(0), Some(69));

		java_set(array, 0, 420);
		assert_eq!(array.get(0), Some(420));

		java_set(array, 2, 420);
		assert_eq!(array.get(2), Some(420));
	})
}

#[test]
fn getter() {
	launch(32 * 1024 * 1024, |runtime| {
		compile(
			&runtime,
			&[("ArrayTest.java", include_str!("ArrayTest.java"))],
		)
		.unwrap();

		runtime
			.class_loader
			.load_jar(include_bytes!("../../../../../rt.zip"), |v| {
				v == "java/lang/Object.class"
			})
			.unwrap();

		let java_get =
			java_bind_method!(runtime fn ArrayTest.getValue(array: Array<i32>, index: i32) -> i32);

		let short = runtime.gc.lock().allocate_array(Kind::Int, 3).unwrap();
		let mut array = Array::new(short);

		assert_eq!(java_get(array, 0), 0);
		array.set(0, 342);
		assert_eq!(java_get(array, 0), 342);
		array.set(0, 69);
		assert_eq!(java_get(array, 0), 69);
	})
}
