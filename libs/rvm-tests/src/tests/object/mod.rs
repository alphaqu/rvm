use crate::bindings::tests::object::{Animal, ExtendedObject, ObjectTests, SimpleObject};
use crate::launch;
use rvm_runtime::gc::AllocationError;
use rvm_runtime::{AnyInstance, InstanceBinding, Runtime, Vm};
use tracing::debug;

fn runtime() -> Runtime<'static> {
	launch(1024)
}

#[test]
pub fn create() {
	let mut runtime = runtime();
	let id = runtime.resolve_class(&SimpleObject::ty().into()).unwrap();

	let instance = ObjectTests::createSimple(&mut runtime).unwrap();

	assert_eq!(instance.untyped().class_id(), id)
}

#[test]
pub fn create_numbered() {
	for i in 0..100 {
		let mut runtime = runtime();

		let instance = ObjectTests::createSimpleNumbered(&mut runtime, 69).unwrap();

		assert_eq!(*instance.value, 69);
	}
}

#[test]
pub fn get_field() {
	let mut runtime = runtime();
	let id = runtime.resolve_class(&SimpleObject::ty().into()).unwrap();
	let class = runtime.vm.classes.get(id);
	let class = class.to_instance();

	let mut instance = runtime.alloc_object(class).unwrap().typed::<SimpleObject>();
	*instance.value = 420;

	let output = ObjectTests::getSimpleField(&mut runtime, instance.clone()).unwrap();
	assert_eq!(output, 420);
}

#[test]
pub fn set_field() {
	let mut runtime = runtime();
	let id = runtime.resolve_class(&SimpleObject::ty().into()).unwrap();

	let class = runtime.vm.classes.get(id);
	let class = class.to_instance();

	let instance = runtime.alloc_object(class).unwrap().typed::<SimpleObject>();
	assert_eq!(*instance.value, 0);

	ObjectTests::setSimpleField(&mut runtime, instance.clone(), 420).unwrap();
	assert_eq!(*instance.value, 420);
}

#[test]
pub fn basic_instance_method() {
	let mut runtime = runtime();
	let id = runtime.resolve_class(&SimpleObject::ty().into()).unwrap();

	let class = runtime.vm.classes.get(id);
	let class = class.to_instance();

	let instance = runtime.alloc_object(class).unwrap().typed::<SimpleObject>();
	assert_eq!(*instance.value, 0);

	let i = ObjectTests::simpleInvocation(&mut runtime, instance).unwrap();
	assert_eq!(i, 640);
}

#[test]
pub fn create_extended() {
	let mut runtime = runtime();
	let id = runtime.resolve_class(&ExtendedObject::ty().into()).unwrap();

	let instance = ObjectTests::createExtended(&mut runtime).unwrap();
	assert_eq!(instance.class_id(), id);
	assert_eq!(*instance.anotherField, 500);
	assert_eq!(*instance.value, 400);

	let instance = instance.cast_to::<SimpleObject>();
	assert_eq!(*instance.value, 400);
}

#[test]
pub fn basic_override() {
	let mut runtime = launch(1024);
	let reference = ObjectTests::createExtended(&mut runtime).unwrap();
	let i =
		ObjectTests::simpleInvocation(&mut runtime, reference.cast_to::<SimpleObject>()).unwrap();

	assert_eq!(i, 640 + 400);
}

#[test]
pub fn casting() {
	let mut runtime = runtime();
	let id = runtime.resolve_class(&ExtendedObject::ty().into()).unwrap();

	let class = runtime.vm.classes.get(id);
	let class = class.to_instance();

	let mut instance = runtime
		.alloc_object(class)
		.unwrap()
		.typed::<ExtendedObject>();
	*instance.value = 500;

	let instance = ObjectTests::casting(&mut runtime, instance).unwrap();

	assert_eq!(*instance.value, 500);
}

#[test]
pub fn interface_call() {
	let mut runtime = runtime();

	let id = runtime.resolve_class(&ExtendedObject::ty().into()).unwrap();

	let class = runtime.vm.classes.get(id);
	let class = class.to_instance();

	let instance = runtime
		.alloc_object(class)
		.unwrap()
		.typed::<ExtendedObject>();

	let instance = ObjectTests::interfaceCall(&mut runtime, instance.cast_to::<Animal>()).unwrap();

	assert_eq!(instance, 49);
}

#[test]
pub fn gc() {
	let mut runtime = launch(1024);

	let id = runtime.resolve_class(&ExtendedObject::ty().into()).unwrap();

	let class = runtime.vm.classes.get(id);
	let class = class.to_instance();

	let mut frozen = 0;
	let mut ran_gc = false;
	for i in 0..16 {
		match runtime.gc.alloc_instance(class) {
			Err(AllocationError::OutOfHeap) => {
				runtime.gc();
				ran_gc = true;
			}
			Ok(value) => {
				if i > 4 {
					runtime.vm.gc.add_frozen(*value);
					frozen += 1;
					debug!("Now frozen {frozen}");
				}
			}
			_ => {}
		}
	}
	assert!(ran_gc);
}
