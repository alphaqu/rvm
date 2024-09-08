use crate::bindings::tests::string::Java;
use crate::launch;

#[test]
fn ldc() {
	let runtime = launch(128);
	Java::ldc(&runtime).unwrap();
}
