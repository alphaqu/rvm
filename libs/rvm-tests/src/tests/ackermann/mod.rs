use rvm_runtime::java_bind_method;

use crate::bindings::tests::ackermann::Ackermann;
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
	let runtime = launch(1024);

	const SAMPLES: usize = 4;
	let java_ack = |m, n| Ackermann::ack(&runtime, m, n).unwrap();
	let rust = sample("Rust ackermann", SAMPLES, || ack(3, 4));
	let java = sample("Java ackermann", SAMPLES, || java_ack(3, 4));

	for ((i, rust), java) in rust.into_iter().enumerate().zip(java.into_iter()) {
		assert_eq!(
			rust, java,
			"Ackermann({i}, 12) Rust ({rust}) vs Java ({java}) failed "
		);
	}

	println!("HLELO");
	Ok(())
}
