#![feature(try_blocks)]
#![feature(exit_status_error)]

use std::fs::{metadata, read, read_dir, File};
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
	eprintln!("HI!");
	let current_dir = std::env::current_dir().unwrap();
	let mut cache_dir = current_dir.clone();
	cache_dir.push("cache");

	let mut paths = vec![];
	walk_dir(PathBuf::from("src/testing"), &mut paths);

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
			"cache/{}.class",
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
								println!("cargo:warning=Skipping {path:?} because {class_path:?} exists.");
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
	println!("cargo:warning=needs_recompile: {needs_recompile}");

	if !needs_recompile {
		return;
	}

	std::fs::create_dir_all(&cache_dir).expect("Could not create cache folder");

	let mut process = Command::new(match std::env::var("JAVA_HOME") {
		Ok(java_home) => format!("{}/bin/javac", java_home),
		_ => "javac".to_string(),
	});
	process.current_dir(&current_dir.join("src")).arg("-Xlint");
	process.arg("-XDignore.symbol.file=true");
	process.args(["-d", "../cache"]);
	//process.args(["--patch-module", "java.base=java/lang"]);
	//process.args(["--system", "none"]);

	for path in paths {
		let path: PathBuf = path.components().skip(1).collect();
		println!("file: {path:?}");
		process.arg(path);
	}

	let status = process.status().unwrap();

	let string = String::from_utf8(process.output().unwrap().stderr).unwrap();
	if !string.trim().is_empty() {
		panic!("ERROR: {}", string);
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
