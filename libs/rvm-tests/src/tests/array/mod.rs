use std::sync::Arc;

use rvm_core::PrimitiveType;
use rvm_runtime::{bind, Array, Reference};

use crate::{launch, load_sdk};

pub struct ArrayTest;

bind!("testing/array" {
	ArrayTest {
		singleArray(value: i32) -> Array<i32>,
		singleRefArray() -> Array<Reference>,
		setValue(array: Array<i32>, index: i32, value: i32),
		getValue(array: Array<i32>, index: i32) -> i32,
		setValueRef(array: Array<Reference>, index: i32, value: Reference),
		getValueRef(array: Array<Reference>, index: i32) -> Reference
	}
});

#[test]
fn primitive() {
	let runtime = launch(1024, vec![]);
	let short = runtime
		.gc
		.lock()
		.allocate_array(PrimitiveType::Int, 3)
		.unwrap();
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
}

#[test]
fn creation() {
	let runtime = launch(1024, vec!["testing/array/ArrayTest.class"]);
	load_sdk(&runtime);

	let java_set = ArrayTest::singleArray(&runtime);

	let array = java_set(420);
	assert_eq!(array.length(), 1);
	assert_eq!(array.get(0), Some(420));
}

#[test]
fn setter() {
	let runtime = launch(1024, vec!["testing/array/ArrayTest.class"]);
	load_sdk(&runtime);

	let java_set = ArrayTest::setValue(&runtime);

	let short = runtime
		.gc
		.lock()
		.allocate_array(PrimitiveType::Int, 3)
		.unwrap();

	let array = Array::new(short);
	assert_eq!(array.get(0), Some(0));

	java_set(array, 0, 69);
	assert_eq!(array.get(0), Some(69));

	java_set(array, 0, 420);
	assert_eq!(array.get(0), Some(420));

	java_set(array, 2, 420);
	assert_eq!(array.get(2), Some(420));
}

#[test]
fn getter() {
	let runtime = launch(1024, vec!["testing/array/ArrayTest.class"]);
	load_sdk(&runtime);

	let short = runtime
		.gc
		.lock()
		.allocate_array(PrimitiveType::Int, 3)
		.unwrap();

	let mut array = Array::new(short);

	let get_value = ArrayTest::getValue(&runtime);
	assert_eq!(get_value(array, 0), 0);
	array.set(0, 342);
	assert_eq!(get_value(array, 0), 342);
	array.set(0, 69);
	assert_eq!(get_value(array, 0), 69);
}

#[test]
fn ref_arrays() {
	let runtime = launch(1024, vec!["testing/array/ArrayTest.class"]);
	load_sdk(&runtime);

	let java_create = ArrayTest::singleRefArray(&runtime);
	let java_get = ArrayTest::getValueRef(&runtime);
	let java_set = ArrayTest::setValueRef(&runtime);

	let array = java_create();

	let reference = **array;

	assert_eq!(array.length(), 2);
	let v0 = array.get(0).unwrap();
	assert_ne!(v0, Reference::NULL);
	assert_ne!(java_get(array, 0), Reference::NULL);
	let v1 = array.get(1).unwrap();
	assert_eq!(v1, Reference::NULL);
	assert_eq!(java_get(array, 1), Reference::NULL);

	let mut visited_0 = false;
	reference.visit_refs(|v| {
		if !visited_0 {
			visited_0 = true;
			assert_eq!(v, v0);
		} else {
			assert_eq!(v, v1);
		}
	});

	assert!(visited_0);

	java_set(array, 1, v0);
	assert_eq!(java_get(array, 1), v0);

	java_set(array, 0, Reference::NULL);
	assert_eq!(java_get(array, 0), Reference::NULL);
	assert_eq!(array.length(), 2);
}
