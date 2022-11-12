#![feature(exit_status_error)]
#![feature(io_error_other)]
#![feature(try_blocks)]
#![feature(pin_macro)]

use std::io::Result;
use std::pin::{pin, Pin};
use std::time::Instant;

use inkwell::context::Context;
use walkdir::WalkDir;

use rvm_runtime::Runtime;

#[cfg(test)]
mod tests;

pub fn launch<F, R>(f: F) -> R
where
	F: FnOnce(&Pin<&Runtime>) -> R + Send + 'static,
	R: Send + 'static,
{
	std::thread::Builder::new()
		.name(
			std::thread::current()
				.name()
				.unwrap_or("Runner")
				.to_string(),
		)
		.stack_size(1024 * 1024 * 64)
		.spawn(|| {
			rvm_core::init();
			let context = Context::create();
			let runtime = Runtime::new(&context);
			let x = f(&pin!(runtime).into_ref());
			x
		})
		.unwrap()
		.join()
		.unwrap()
}

pub fn compile(runtime: &Runtime, sources: &[(&str, &str)]) -> Result<()> {
	let mut root = std::env::temp_dir();
	root.push(&format!("rvm-{:p}", runtime));

	let result: Result<()> = try {
		std::fs::create_dir_all(&root)?;

		let mut process = std::process::Command::new(match std::env::var("JAVA_HOME") {
			Ok(java_home) => format!("{}/bin/javac", java_home),
			_ => "javac".to_string(),
		});
		process.current_dir(&root).arg("-Xlint");

		for &(name, source) in sources {
			let mut file = root.clone();
			file.push(name);

			if let Some(parent) = file.parent() {
				std::fs::create_dir_all(parent)?;
			}

			std::fs::write(file, source)?;
			process.arg(name);
		}

		process.status()?.exit_ok().expect("javac not successful");

		for entry in WalkDir::new(&root) {
			let entry = entry?;

			if entry.path().extension().and_then(|x| x.to_str()) == Some("class") {
				runtime
					.cl
					.load_class(&std::fs::read(entry.path())?)
					.unwrap();
			}
		}
	};

	if let Err(error) = std::fs::remove_dir_all(root) {
		eprintln!("Error cleaning up: {}", error);
	}

	result
}

pub fn sample<F, R>(message: &str, times: usize, f: F) -> Vec<R>
where
	F: Fn(usize) -> R,
{
	let mut nanos = 0;
	let mut results = Vec::with_capacity(times);

	for i in 0..times {
		let start = Instant::now();
		results.push(std::hint::black_box(f(i)));
		nanos += start.elapsed().as_nanos();
	}

	println!(
		"{} took {}ms ({} nanos) on average",
		message,
		nanos / 1000_000,
		nanos
	);
	results
}
