use rvm_runtime::java_bind_method;

use crate::bindings::tests::integers::Java;
use crate::{compile, launch};

#[test]
fn test() -> Result<(), std::io::Error> {
	let runtime = launch(1024);

	compile(
		&runtime,
		&[(
			"Main.java",
			"public class Main {
    public static boolean testZeroEq(int v) {
        return v == 0;
    }

    public static boolean testZeroNeq(int v) {
        return v != 0;
    }

    public static boolean testZeroGt(int v) {
        return v > 0;
    }

    public static boolean testZeroGe(int v) {
        return v >= 0;
    }

    public static boolean testZeroLt(int v) {
        return v < 0;
    }

    public static boolean testZeroLe(int v) {
        return v <= 0;
    }
}",
		)],
	)?;

	// v == 0
	let func = java_bind_method!(runtime fn Main:testZeroEq(v: i32) -> bool);
	assert!(func(0));
	assert!(!func(1));
	assert!(!func(-1));
	assert!(!func(i32::MIN));
	assert!(!func(i32::MAX));

	// v != 0
	let func = java_bind_method!(runtime fn Main:testZeroNeq(v: i32) -> bool);
	assert!(!func(0));
	assert!(func(1));
	assert!(func(-1));
	assert!(func(i32::MIN));
	assert!(func(i32::MAX));

	// v > 0
	let func = java_bind_method!(runtime fn Main:testZeroGt(v: i32) -> bool);
	assert!(!func(0));
	assert!(!func(-1));
	assert!(!func(i32::MIN));
	assert!(func(1));
	assert!(func(i32::MAX));

	// v >= 0
	let func = java_bind_method!(runtime fn Main:testZeroGe(v: i32) -> bool);
	assert!(!func(-1));
	assert!(!func(i32::MIN));
	assert!(func(0));
	assert!(func(1));
	assert!(func(i32::MAX));

	// v < 0
	let func = java_bind_method!(runtime fn Main:testZeroLt(v: i32) -> bool);
	assert!(!func(0));
	assert!(!func(1));
	assert!(!func(i32::MAX));
	assert!(func(-1));
	assert!(func(i32::MIN));

	// v <= 0
	let func = java_bind_method!(runtime fn Main:testZeroLe(v: i32) -> bool);
	assert!(!func(1));
	assert!(!func(i32::MAX));
	assert!(func(0));
	assert!(func(-1));
	assert!(func(i32::MIN));

	Ok(())
}
