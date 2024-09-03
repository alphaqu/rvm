use rvm_runtime::{InstanceBinding, Runtime};

use crate::bindings::tests::object::{Animal, ExtendedObject, ObjectTests, SimpleObject};
use crate::launch2;

fn runtime() -> Runtime {
	launch2(1024)
}

#[test]
pub fn create() {
	let runtime = runtime();
	let id = runtime.classes.resolve(&SimpleObject::ty().into());

	let instance = ObjectTests::createSimple(&runtime).unwrap();

	assert_eq!(instance.untyped().class_id(), id)
}

#[test]
pub fn create_numbered() {
	for i in 0..100 {
		let runtime = runtime();

		let instance = ObjectTests::createSimpleNumbered(&runtime, 69).unwrap();

		assert_eq!(*instance.value, 69);
	}
}

#[test]
pub fn get_field() {
	let runtime = runtime();
	let id = runtime.classes.resolve(&SimpleObject::ty().into());

	let mut instance = runtime.alloc_object(id).typed::<SimpleObject>();
	*instance.value = 420;

	let output = ObjectTests::getSimpleField(&runtime, instance.clone()).unwrap();
	assert_eq!(output, 420);
}

#[test]
pub fn set_field() {
	let runtime = runtime();
	let id = runtime.classes.resolve(&SimpleObject::ty().into());

	let instance = runtime.alloc_object(id).typed::<SimpleObject>();
	assert_eq!(*instance.value, 0);

	ObjectTests::setSimpleField(&runtime, instance.clone(), 420).unwrap();
	assert_eq!(*instance.value, 420);
}

#[test]
pub fn basic_instance_method() {
	let runtime = runtime();
	let id = runtime.classes.resolve(&SimpleObject::ty().into());

	let instance = runtime.alloc_object(id).typed::<SimpleObject>();
	assert_eq!(*instance.value, 0);

	let i = ObjectTests::simpleInvocation(&runtime, instance).unwrap();
	assert_eq!(i, 640);
}

#[test]
pub fn create_extended() {
	let runtime = runtime();
	let id = runtime.classes.resolve(&ExtendedObject::ty().into());

	let instance = ObjectTests::createExtended(&runtime).unwrap();
	assert_eq!(instance.class_id(), id);
	assert_eq!(*instance.anotherField, 500);
	assert_eq!(*instance.value, 400);

	let instance = instance.cast_to::<SimpleObject>();
	assert_eq!(*instance.value, 400);
}

#[test]
pub fn basic_override() {
	let runtime = launch2(1024);
	let reference = ObjectTests::createExtended(&runtime).unwrap();
	let i = ObjectTests::simpleInvocation(&runtime, reference.cast_to::<SimpleObject>()).unwrap();

	assert_eq!(i, 640 + 400);
}

#[test]
pub fn casting() {
	let runtime = runtime();
	let id = runtime.classes.resolve(&ExtendedObject::ty().into());
	let mut instance = runtime.alloc_object(id).typed::<ExtendedObject>();
	*instance.value = 500;

	let instance = ObjectTests::casting(&runtime, instance).unwrap();

	assert_eq!(*instance.value, 500);
}

#[test]
pub fn interface_call() {
	let runtime = runtime();

	let id = runtime.classes.resolve(&ExtendedObject::ty().into());
	let instance = runtime.alloc_object(id).typed::<ExtendedObject>();

	let instance = ObjectTests::interfaceCall(&runtime, instance.cast_to::<Animal>()).unwrap();

	assert_eq!(instance, 49);
}
