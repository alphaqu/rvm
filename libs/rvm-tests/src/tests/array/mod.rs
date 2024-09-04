use rvm_core::PrimitiveType;
use rvm_runtime::{Array, Reference};

use crate::bindings::tests::array::ArrayTest;
use crate::{launch, load_sdk};

#[test]
fn primitive() {
	let runtime = launch(1024);
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
fn creation() -> eyre::Result<()> {
	let runtime = launch(1024);
	load_sdk(&runtime);

	let array = ArrayTest::singleArray(&runtime, 420)?;
	assert_eq!(array.length(), 1);
	assert_eq!(array.get(0), Some(420));
	Ok(())
}

#[test]
fn setter() -> eyre::Result<()> {
	let runtime = launch(1024);
	load_sdk(&runtime);

	let short = runtime.gc.lock().allocate_array(PrimitiveType::Int, 3)?;

	let array = Array::new(short);
	assert_eq!(array.get(0), Some(0));

	ArrayTest::setValue(&runtime, array, 0, 69)?;
	assert_eq!(array.get(0), Some(69));

	ArrayTest::setValue(&runtime, array, 0, 420)?;
	assert_eq!(array.get(0), Some(420));

	ArrayTest::setValue(&runtime, array, 2, 420)?;
	assert_eq!(array.get(2), Some(420));
	Ok(())
}

#[test]
fn getter() -> eyre::Result<()> {
	let runtime = launch(1024);
	load_sdk(&runtime);

	let short = runtime.gc.lock().allocate_array(PrimitiveType::Int, 3)?;

	let mut array = Array::new(short);

	assert_eq!(ArrayTest::getValue(&runtime, array, 0)?, 0);
	array.set(0, 342);
	assert_eq!(ArrayTest::getValue(&runtime, array, 0)?, 342);
	array.set(0, 69);
	assert_eq!(ArrayTest::getValue(&runtime, array, 0)?, 69);
	Ok(())
}

#[test]
fn ref_arrays() -> eyre::Result<()> {
	let runtime = launch(1024);
	load_sdk(&runtime);

	let array = ArrayTest::singleRefArray(&runtime)?;

	let reference = **array;

	assert_eq!(array.length(), 2);
	let v0 = array.get(0).unwrap();
	assert_ne!(v0, Reference::NULL);
	assert_ne!(ArrayTest::getValueRef(&runtime, array, 0)?, Reference::NULL);
	let v1 = array.get(1).unwrap();
	assert_eq!(v1, Reference::NULL);
	assert_eq!(ArrayTest::getValueRef(&runtime, array, 1)?, Reference::NULL);

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

	ArrayTest::setValueRef(&runtime, array, 1, v0)?;
	assert_eq!(ArrayTest::getValueRef(&runtime, array, 1)?, v0);

	ArrayTest::setValueRef(&runtime, array, 0, Reference::NULL)?;
	assert_eq!(ArrayTest::getValueRef(&runtime, array, 0)?, Reference::NULL);
	assert_eq!(array.length(), 2);

	Ok(())
}
