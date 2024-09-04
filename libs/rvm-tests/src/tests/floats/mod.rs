use rvm_runtime::java_bind_method;

use crate::{compile, launch};

#[test]
fn test() -> Result<(), std::io::Error> {
	let runtime = launch(1024);
	compile(
		&runtime,
		&[(
			"Main.java",
			"public class Main {
    public statics double get() {
        double i = Math.PI;
        i += 0 * i;
        i += 1 * i;
        i += 2 * i;
        i += 3 * i;
        i += 0 + i;
        i += 1 + i;
        i += 2 + i;
        i += 3 + i;
    	i += 0 - i;
        i += 1 - i;
        i += 2 - i;
        i += 3 - i;
		i += 0 % i;
        i += 1 % i;
        i += 2 % i;
        i += 3 % i;
        i += 4 % i;
        i += 2 / i;
        i += 1 / i;
        i += 0 / i;
        return i;
    }
}",
		)],
	)?;

	let rust = {
		let mut i = 3.14159265358979323846f64;
		i += 0.0 * i;
		i += 1.0 * i;
		i += 2.0 * i;
		i += 3.0 * i;
		i += 0.0 + i;
		i += 1.0 + i;
		i += 2.0 + i;
		i += 3.0 + i;
		i += 0.0 - i;
		i += 1.0 - i;
		i += 2.0 - i;
		i += 3.0 - i;
		i += 0.0 % i;
		i += 1.0 % i;
		i += 2.0 % i;
		i += 3.0 % i;
		i += 4.0 % i;
		i += 2.0 / i;
		i += 1.0 / i;
		i += 0.0 / i;
		i
	};
	let java = java_bind_method!(runtime fn Main:get() -> f64);
	assert_eq!(rust, java());

	Ok(())
}
