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
    public static int pow(int base, int power) {
        int result = 1;

        while (power > 0) {
            if (power % 2 == 1) {
                result = result * base;
            }

            base = base * base;
            power >>= 1;
        }

        return result;
    }
}",
			)],
		)?;

		let jack = unsafe { java!(compile runtime, fn Main.pow(i32, i32) -> i32) };

		const SAMPLES: usize = 4;

		for i in 0..8i32 {
			for j in 0..8i32 {
				assert_eq!(unsafe { jack(i, j) }, i.pow(j as u32));
			}
		}

		Ok(())
	})
}
