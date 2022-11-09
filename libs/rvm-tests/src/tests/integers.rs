use rvm_runtime::java;

use crate::{compile, launch};

#[test]
fn test() -> Result<(), std::io::Error> {
	launch(|runtime| {
		compile(
			&*runtime,
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

		unsafe {
			// v == 0
			let eq = java!(compile runtime, fn Main.testZeroEq(i32) -> bool);
			assert!(eq(0));
			assert!(!eq(1));
			assert!(!eq(-1));
			assert!(!eq(i32::MIN));
			assert!(!eq(i32::MAX));

			// v != 0
			let neq = java!(compile runtime, fn Main.testZeroNeq(i32) -> bool);
			assert!(!neq(0));
			assert!(neq(1));
			assert!(neq(-1));
			assert!(neq(i32::MIN));
			assert!(neq(i32::MAX));

			// v > 0
			let gt = java!(compile runtime, fn Main.testZeroGt(i32) -> bool);
			assert!(!gt(0));
			assert!(!gt(-1));
			assert!(!gt(i32::MIN));
			assert!(gt(1));
			assert!(gt(i32::MAX));

			// v >= 0
			let ge = java!(compile runtime, fn Main.testZeroGe(i32) -> bool);
			assert!(!ge(-1));
			assert!(!ge(i32::MIN));
			assert!(ge(0));
			assert!(ge(1));
			assert!(ge(i32::MAX));

			// v < 0
			let lt = java!(compile runtime, fn Main.testZeroLt(i32) -> bool);
			assert!(!lt(0));
			assert!(!lt(1));
			assert!(!lt(i32::MAX));
			assert!(lt(-1));
			assert!(lt(i32::MIN));

			// v <= 0
			let le = java!(compile runtime, fn Main.testZeroLe(i32) -> bool);
			assert!(!le(1));
			assert!(!le(i32::MAX));
			assert!(le(0));
			assert!(le(-1));
			assert!(le(i32::MIN));
		}

		Ok(())
	})
}
