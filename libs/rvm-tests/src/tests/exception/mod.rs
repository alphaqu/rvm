use crate::bindings::tests::exception::Java;
use crate::launch;

#[test]
fn basic() {
	let mut runtime = launch(128);
	let basic1 = Java::basic(&mut runtime, true);
}
