use crate::bindings::tests::statics::Java;
use crate::launch;

#[test]
fn get_static() {
	let runtime = launch(1024);

	assert_eq!(Java::getStatic(&runtime).unwrap(), 3);
}
