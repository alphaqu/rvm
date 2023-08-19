use crate::{compile, launch};
use rvm_runtime::java_bind_method;

#[test]
fn test() -> Result<(), std::io::Error> {
	launch(32 * 1024 * 1024, |runtime| {
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

		const SAMPLES: usize = 4;

		for i in 0..8i32 {
			for j in 0..8i32 {
				let pow = java_bind_method!(runtime fn Main:pow(base: i32, power: i32) -> i32);
				assert_eq!(pow(i, j), i.pow(j as u32));
			}
		}

		Ok(())
	})
}
