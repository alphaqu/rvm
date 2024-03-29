use rvm_runtime::java_bind_method;

use crate::{compile, launch, sample};

#[inline(always)]
fn ack(m: i32, n: i32) -> i32 {
	return if m == 0 {
		n + 1
	} else if m > 0 && n == 0 {
		ack(m - 1, 1)
	} else if m > 0 && n > 0 {
		ack(m - 1, ack(m, n - 1))
	} else {
		n + 1
	};
}

#[test]
fn test() -> Result<(), std::io::Error> {
	launch(32 * 1024 * 1024, |runtime| {
		compile(
			&*runtime,
			&[(
				"Main.java",
				"public class Main {
    static int ack(int m, int n) {
        if (m == 0) {
            return n + 1;
        } else if (m > 0 && n == 0) {
            return ack(m - 1, 1);
        } else if (m > 0 && n > 0) {
            return ack(m - 1, ack(m, n - 1));
        } else {
            return n + 1;
        }
    }
}",
			)],
		)?;

		const SAMPLES: usize = 3;
		let java_ack = java_bind_method!(runtime fn Main:ack(m: i32, n: i32) -> i32);
		let rust = sample("Rust ackermann", SAMPLES, |i| ack(i as i32, 10));
		let java = sample("Java ackermann", SAMPLES, |i| java_ack(i as i32, 10));

		for ((i, rust), java) in rust.into_iter().enumerate().zip(java.into_iter()) {
			assert_eq!(
				rust, java,
				"Ackermann({i}, 12) Rust ({rust}) vs Java ({java}) failed "
			);
		}

		Ok(())
	})
}
