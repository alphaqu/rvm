use crate::bindings::tests::string::Java;
use crate::launch;

#[test]
fn ldc() {
	let mut runtime = launch(1024);
	Java::ldc(&mut runtime).unwrap();
}
