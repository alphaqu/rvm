use rvm_runtime::java_bind_method;

use crate::bindings::tests::floats::Java;
use crate::{compile, launch};

#[test]
fn test() -> Result<(), std::io::Error> {
	let mut runtime = launch(128);

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

	assert_eq!(rust, Java::get(&mut runtime).unwrap());

	Ok(())
}
