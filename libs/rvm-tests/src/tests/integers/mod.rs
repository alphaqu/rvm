use crate::bindings::tests::integers::Java;
use crate::launch;

#[test]
fn test() -> Result<(), std::io::Error> {
	let runtime = launch(128);

	// v == 0
	let func = |v| Java::testZeroEq(&runtime, v).unwrap();
	assert!(func(0));
	assert!(!func(1));
	assert!(!func(-1));
	assert!(!func(i32::MIN));
	assert!(!func(i32::MAX));

	// v != 0
	let func = |v| Java::testZeroNeq(&runtime, v).unwrap();
	assert!(!func(0));
	assert!(func(1));
	assert!(func(-1));
	assert!(func(i32::MIN));
	assert!(func(i32::MAX));

	// v > 0
	let func = |v| Java::testZeroGt(&runtime, v).unwrap();
	assert!(!func(0));
	assert!(!func(-1));
	assert!(!func(i32::MIN));
	assert!(func(1));
	assert!(func(i32::MAX));

	// v >= 0
	let func = |v| Java::testZeroGe(&runtime, v).unwrap();
	assert!(!func(-1));
	assert!(!func(i32::MIN));
	assert!(func(0));
	assert!(func(1));
	assert!(func(i32::MAX));

	// v < 0
	let func = |v| Java::testZeroLt(&runtime, v).unwrap();
	assert!(!func(0));
	assert!(!func(1));
	assert!(!func(i32::MAX));
	assert!(func(-1));
	assert!(func(i32::MIN));

	// v <= 0
	let func = |v| Java::testZeroLe(&runtime, v).unwrap();
	assert!(!func(1));
	assert!(!func(i32::MAX));
	assert!(func(0));
	assert!(func(-1));
	assert!(func(i32::MIN));

	Ok(())
}
