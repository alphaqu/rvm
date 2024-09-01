use std::env::temp_dir;
use std::fs::{create_dir_all, write, File};
use std::process::Command;

mod array;
mod consts;
mod math;

pub fn quick_compile(name: &str, data: &str) -> String {
	let buf = temp_dir().join("rvm-quick").join(name);
	create_dir_all(&buf).unwrap();

	write(buf.join("Main.java"), data).unwrap();

	let mut command = bin("javac");
	command.current_dir(&buf);
	command.arg("Main.java");
	let output1 = command.output().unwrap();
	if let Err(error) = output1.status.exit_ok() {
		panic!("{error}: {}", String::from_utf8(output1.stderr).unwrap());
	}

	let mut command = bin("java");
	command.current_dir(&buf);
	command.arg("Main");
	let output = command.output().unwrap();
	output.status.exit_ok().unwrap();

	String::from_utf8(output.stdout).unwrap()
}
pub fn bin(name: &str) -> Command {
	let mut process = Command::new(match std::env::var("JAVA_HOME") {
		Ok(java_home) => format!("{java_home}/bin/{name}"),
		_ => name.to_string(),
	});
	process
}
