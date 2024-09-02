#![feature(try_blocks)]
#![feature(exit_status_error)]

use std::env::vars;
use std::fs::{metadata, read_dir, File};
use std::path::PathBuf;
use std::process::Command;

fn main() {
	eprintln!("HI!");
	let current_dir = std::env::current_dir().unwrap();
	let mut bytecode_dir = current_dir.clone();
	bytecode_dir.push("bytecode");

	let mut paths = vec![];
	walk_dir(PathBuf::from("src"), &mut paths);

	// Check if it needs recompiling
	let mut needs_recompile = false;
	for path in &paths {
		let java_file = File::open(path)
			.unwrap()
			.metadata()
			.unwrap()
			.modified()
			.unwrap();
		let class_path = format!(
			"bytecode/{}.class",
			path.to_str()
				.unwrap()
				.trim_start_matches("src/")
				.trim_end_matches(".java")
		);
		let class_path = PathBuf::from(class_path);

		if class_path.exists() {
			match metadata(&class_path) {
				Ok(class_file) => {
					match class_file.modified() {
						Ok(class_modified) => {
							if java_file <= class_modified {
								//println!("cargo:warning=Skipping {path:?} because {class_path:?} exists.");
								continue;
							}
						}
						Err(_) => {
							println!("cargo:warning=Could not get modified time");
						}
					}
				}
				_ => {
					println!("cargo:warning=Could not find file at {class_path:?}");
				}
			}
		} else {
			println!("cargo:warning={class_path:?} does not exist");
		}

		println!("cargo:warning={path:?} needs recompiling");
		needs_recompile = true;
		break;
	}
	//println!("cargo:warning=needs_recompile: {needs_recompile}");

	if !needs_recompile {
		return;
	}

	std::fs::create_dir_all(&bytecode_dir).expect("Could not create bytecode dir");

	for (key, value) in vars() {
		println!("{key}: {value}")
	}
	let mut process = Command::new(match std::env::var("JAVA_HOME") {
		Ok(java_home) => {
			println!("Using JDK: \"{java_home}\"",);
			format!("{}/bin/javac", java_home)
		}
		_ => "javac".to_string(),
	});

	println!(
		"Using JAVAC: \"{}\"",
		process.get_program().to_str().unwrap()
	);
	process.current_dir(&current_dir.join("src")).arg("-Xlint");

	process.arg("-XDignore.symbol.file=true");
	process.args(["-d", "../bytecode"]);
	//process.args(["--patch-module", "java.base=java/lang"]);
	//process.args(["--system", "none"]);

	for path in paths {
		let path: PathBuf = path.components().collect();
		let canonic_path = path.canonicalize().unwrap();

		process.arg(canonic_path);
	}

	let status = process.status().expect("Could not start java compiler");

	let string = String::from_utf8(process.output().unwrap().stderr).unwrap();
	if !string.trim().is_empty() {
		for string in string.split("\n") {
			println!("cargo:warning=JAVAC: {string}",);
		}
	}
	status.exit_ok().expect("javac not successful");
}

fn walk_dir(path: PathBuf, paths: &mut Vec<PathBuf>) {
	for dir in read_dir(&path).unwrap() {
		let entry = dir.unwrap();
		if entry.path().extension().map(|v| v.to_str().unwrap()) == Some("java") {
			paths.push(entry.path());
		} else if entry.metadata().unwrap().is_dir() {
			walk_dir(path.join(entry.file_name()), paths);
		}
	}
}
