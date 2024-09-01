use crate::{launch, load_sdk};
use rvm_core::{ObjectType, Type};
use rvm_runtime::{
	bind, AnyInstance, Array, Instance, InstanceBinding, InstanceReference, Reference, Runtime,
	TypedField, Value,
};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

pub struct JavaField<V: Value> {
	value: *mut V,
}
#[derive(Clone, Copy)]
pub struct Animal;

impl InstanceBinding for Animal {
	fn ty() -> ObjectType {
		ObjectType("testing/object/Animal".to_string())
	}

	fn bind(instance: &AnyInstance) -> Self {
		Animal {}
	}
}

#[derive(Clone, Copy)]
pub struct SimpleObject {
	value: TypedField<i32>,
}

impl InstanceBinding for SimpleObject {
	fn ty() -> ObjectType {
		ObjectType("testing/object/SimpleObject".to_string())
	}

	fn bind(instance: &AnyInstance) -> Self {
		SimpleObject {
			value: instance.field_named("value").unwrap().typed(),
		}
	}
}

#[derive(Clone, Copy)]
pub struct ExtendedObject {
	base: SimpleObject,
	another_field: TypedField<i64>,
}

impl InstanceBinding for ExtendedObject {
	fn ty() -> ObjectType {
		ObjectType("testing/object/ExtendedObject".to_string())
	}

	fn bind(instance: &AnyInstance) -> Self {
		ExtendedObject {
			base: SimpleObject::bind(instance),
			another_field: instance.field_named("anotherField").unwrap().typed(),
		}
	}
}
impl Deref for ExtendedObject {
	type Target = SimpleObject;

	fn deref(&self) -> &Self::Target {
		&self.base
	}
}

impl DerefMut for ExtendedObject {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.base
	}
}

pub struct ObjectTests;
bind!("testing/object" {
	ObjectTests {
		createSimple() -> Instance<SimpleObject>,
		createSimpleNumbered(number: i32) -> Instance<SimpleObject>,
		createExtended() -> Instance<ExtendedObject>,
		casting(object: Instance<ExtendedObject>) -> Instance<SimpleObject>,
		getSimpleField(instance: Instance<SimpleObject>) -> i32,
		setSimpleField(instance: Instance<SimpleObject>, number: i32),
		simpleInvocation(instance: Instance<SimpleObject>) -> i32,
		interfaceCall(instance: Instance<Animal>) -> i32
	}
});

fn runtime() -> Arc<Runtime> {
	let runtime = launch(
		1024,
		vec![
			"testing/object/Animal.class",
			"testing/object/ObjectTests.class",
			"testing/object/SimpleObject.class",
			"testing/object/ExtendedObject.class",
			"testing/object/Dog.class",
		],
	);
	load_sdk(&runtime);
	runtime
}

#[test]
pub fn create() {
	let runtime = runtime();
	let id = runtime.cl.resolve_class(&SimpleObject::ty().into());

	let create = ObjectTests::createSimple(&runtime);

	let instance = create();
	assert_eq!(instance.untyped().class_id(), id)
}

#[test]
pub fn create_numbered() {
	let runtime = runtime();

	let create = ObjectTests::createSimpleNumbered(&runtime);

	let instance = create(69);
	assert_eq!(*instance.value, 69);
}

#[test]
pub fn get_field() {
	let runtime = runtime();
	let id = runtime.cl.resolve_class(&SimpleObject::ty().into());

	let mut instance = runtime.alloc_object(id).typed::<SimpleObject>();
	*instance.value = 420;

	let get_field = ObjectTests::getSimpleField(&runtime);

	let output = get_field(instance.clone());
	assert_eq!(output, 420);
}

#[test]
pub fn set_field() {
	let runtime = runtime();
	let id = runtime.cl.resolve_class(&SimpleObject::ty().into());

	let instance = runtime.alloc_object(id).typed::<SimpleObject>();
	assert_eq!(*instance.value, 0);

	let set_field = ObjectTests::setSimpleField(&runtime);
	set_field(instance.clone(), 420);
	assert_eq!(*instance.value, 420);
}

#[test]
pub fn basic_instance_method() {
	let runtime = runtime();
	let id = runtime.cl.resolve_class(&SimpleObject::ty().into());

	let instance = runtime.alloc_object(id).typed::<SimpleObject>();
	assert_eq!(*instance.value, 0);

	let invocation = ObjectTests::simpleInvocation(&runtime);
	let i = invocation(instance);
	assert_eq!(i, 640);
}

#[test]
pub fn create_extended() {
	let runtime = runtime();
	let id = runtime.cl.resolve_class(&ExtendedObject::ty().into());

	let create_extended = ObjectTests::createExtended(&runtime);

	let instance = create_extended();
	assert_eq!(instance.class_id(), id);
	assert_eq!(*instance.another_field, 500);
	assert_eq!(*instance.value, 400);

	let instance = instance.cast_to::<SimpleObject>();
	assert_eq!(*instance.value, 400);
}

#[test]
pub fn basic_override() {
	let runtime = runtime();

	let create_extended = ObjectTests::createExtended(&runtime);
	let simple_invocation = ObjectTests::simpleInvocation(&runtime);

	let instance = create_extended();

	let i = simple_invocation(instance.cast_to::<SimpleObject>());

	assert_eq!(i, 640 + 400);
}

#[test]
pub fn casting() {
	let runtime = runtime();
	let id = runtime.cl.resolve_class(&ExtendedObject::ty().into());
	let mut instance = runtime.alloc_object(id).typed::<ExtendedObject>();
	*instance.value = 500;

	let cast = ObjectTests::casting(&runtime);

	let instance = cast(instance);

	assert_eq!(*instance.value, 500);
}

#[test]
pub fn interface_call() {
	let runtime = runtime();

	let id = runtime.cl.resolve_class(&ExtendedObject::ty().into());
	let mut instance = runtime.alloc_object(id).typed::<ExtendedObject>();

	let cast = ObjectTests::interfaceCall(&runtime);

	let instance = cast(instance.cast_to::<Animal>());

	assert_eq!(instance, 49);
}
