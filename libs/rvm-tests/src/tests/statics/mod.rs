use crate::bindings::tests::statics::Java;
use crate::launch;
use rvm_runtime::JavaTyped;

#[test]
fn get_static() {
	let runtime = launch(1024);

	let id = runtime.resolve_class(&Java::java_type()).unwrap();
	let class = runtime.classes.get(id);
	let class = class.to_instance();

	let mut field = class
		.static_fields()
		.by_name_typed::<i32>("number")
		.unwrap();

	*field = 69;

	assert_eq!(Java::getStatic(&runtime).unwrap(), 69);
}

#[test]
fn set_static() {
	let runtime = launch(1024);

	let id = runtime.resolve_class(&Java::java_type()).unwrap();
	let class = runtime.classes.get(id);
	let class = class.to_instance();

	let field = class
		.static_fields()
		.by_name_typed::<i32>("number")
		.unwrap();

	Java::setStatic(&runtime, 420).unwrap();
	assert_eq!(*field, 420);
}

#[test]
fn static_class_init() {
	let runtime = launch(1024);

	let id = runtime.resolve_class(&Java::java_type()).unwrap();
	let class = runtime.classes.get(id);
	let class = class.to_instance();

	let field = class
		.static_fields()
		.by_name_typed::<i32>("number")
		.unwrap();

	// This is set by the class initialiation
	assert_eq!(*field, 3);
}
