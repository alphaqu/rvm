#![feature(exit_status_error)]
#![feature(io_error_other)]
#![feature(try_blocks)]
#![feature(pin_macro)]
#![feature(unchecked_math)]

use std::fs::read;
use std::io::Result;
use std::sync::Arc;
use std::time::Instant;

use walkdir::WalkDir;

use rvm_engine_ben::BenBinding;
use rvm_runtime::Runtime;

mod testing;
#[cfg(test)]
mod tests;

pub fn load_sdk(runtime: &Runtime) {
	let vec = read("../../rt.zip").unwrap();
	runtime
		.cl
		.load_jar(&vec, |v| v == "java/lang/Object.class")
		.unwrap();
}

pub fn launch(heap_size: usize, files: Vec<&str>) -> Arc<Runtime> {
	rvm_core::init();
	let runtime = Arc::new(Runtime::new(heap_size, Box::new(BenBinding::new())));

	load_sdk(&runtime);
	for x in files {
		runtime
			.cl
			.load_class(&read(format!("cache/{x}")).unwrap())
			.unwrap();
	}
	runtime
}

pub fn compile(runtime: &Runtime, sources: &[(&str, &str)]) -> Result<()> {
	let mut root = std::env::current_dir().unwrap();
	root.push("temp");
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

		let status = process.status()?;

		println!(
			"ERROR: {}",
			String::from_utf8(process.output().unwrap().stderr).unwrap()
		);
		status.exit_ok().expect("javac not successful");

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
		nanos as f32 / 1000_000.0,
		nanos
	);
	results
}
