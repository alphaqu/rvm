#![feature(exit_status_error)]
#![feature(try_blocks)]

use std::borrow::Borrow;
use std::fs::read;
use std::io::Result;
use std::sync::Arc;
use std::time::Instant;

use crate::core::load_test_core;
use rvm_core::{MethodDescriptor, ObjectType, Type};
use rvm_engine_ben::BenBinding;
use rvm_runtime::{AnyValue, MethodIdentifier, Runtime};
use walkdir::WalkDir;

mod core;
#[cfg(test)]
mod tests;

pub fn load_sdk(runtime: &Runtime) {
	let vec = read("../../rt.zip").unwrap();
	runtime
		.classes
		.load_jar(&vec, |v| v == "java/lang/Object.class")
		.unwrap();
}

pub struct SimpleClassTest {
	pub name: String,
	pub methods: Vec<SimpleMethodTest>,
}

pub struct SimpleMethodTest {
	pub name: String,
	pub parameters: Vec<(Type, AnyValue)>,
	pub returns: Option<(Type, AnyValue)>,
}

impl SimpleMethodTest {
	pub fn void(name: impl ToString) -> SimpleMethodTest {
		SimpleMethodTest {
			name: name.to_string(),
			parameters: vec![],
			returns: None,
		}
	}

	pub fn parameters_simple(name: impl ToString, parameters: Vec<AnyValue>) -> SimpleMethodTest {
		SimpleMethodTest {
			name: name.to_string(),
			parameters: parameters
				.into_iter()
				.map(|v| (v.kind().weak_ty(), v))
				.collect(),
			returns: None,
		}
	}
}

pub fn simple_launch(tests: Vec<SimpleClassTest>) {
	let mut classes: Vec<String> = tests.iter().map(|v| format!("{}.class", v.name)).collect();

	// Core
	classes.push("core/Assert.class".to_string());

	let runtime = launch(1024, classes.iter().map(|v| v.as_str()).collect());
	load_test_core(&runtime);

	for test in tests {
		let ty = ObjectType::new(test.name);
		for method in test.methods {
			let _ = runtime.simple_run(
				ty.clone(),
				MethodIdentifier {
					name: method.name.into(),
					descriptor: MethodDescriptor {
						parameters: method.parameters.iter().map(|(v, _)| v.clone()).collect(),
						returns: method.returns.as_ref().map(|(v, _)| v.clone()),
					}
					.to_string()
					.into(),
				},
				method.parameters.iter().map(|(_, v)| *v).collect(),
			);
		}
	}
}

pub fn launch(heap_size: usize, files: Vec<&str>) -> Arc<Runtime> {
	rvm_core::init();
	let runtime = Arc::new(Runtime::new(heap_size, Box::new(BenBinding::new())));

	load_sdk(&runtime);
	for x in files {
		let x1 = x.borrow();
		runtime
			.classes
			.load_class(&read(format!("bytecode/{x1}")).unwrap())
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
					.classes
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
	F: Fn() -> R,
{
	let mut nanos = 0;
	let mut results = Vec::with_capacity(times);

	for i in 0..times {
		let start = Instant::now();
		results.push(std::hint::black_box(f()));
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
