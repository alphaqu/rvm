use crate::bindings::tests::integers::Java;
use crate::launch;

#[test]
fn test() -> Result<(), std::io::Error> {
	let mut runtime = launch(128);

	// v == 0
	let mut func = |v| Java::testZeroEq(&mut runtime, v).unwrap();
	assert!(func(0));
	assert!(!func(1));
	assert!(!func(-1));
	assert!(!func(i32::MIN));
	assert!(!func(i32::MAX));

	// v != 0
	let mut func = |v| Java::testZeroNeq(&mut runtime, v).unwrap();
	assert!(!func(0));
	assert!(func(1));
	assert!(func(-1));
	assert!(func(i32::MIN));
	assert!(func(i32::MAX));

	// v > 0
	let mut func = |v| Java::testZeroGt(&mut runtime, v).unwrap();
	assert!(!func(0));
	assert!(!func(-1));
	assert!(!func(i32::MIN));
	assert!(func(1));
	assert!(func(i32::MAX));

	// v >= 0
	let mut func = |v| Java::testZeroGe(&mut runtime, v).unwrap();
	assert!(!func(-1));
	assert!(!func(i32::MIN));
	assert!(func(0));
	assert!(func(1));
	assert!(func(i32::MAX));

	// v < 0
	let mut func = |v| Java::testZeroLt(&mut runtime, v).unwrap();
	assert!(!func(0));
	assert!(!func(1));
	assert!(!func(i32::MAX));
	assert!(func(-1));
	assert!(func(i32::MIN));

	// v <= 0
	let mut func = |v| Java::testZeroLe(&mut runtime, v).unwrap();
	assert!(!func(1));
	assert!(!func(i32::MAX));
	assert!(func(0));
	assert!(func(-1));
	assert!(func(i32::MIN));

	Ok(())
}
