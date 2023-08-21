use crate::{compile, launch};
use rvm_runtime::java_bind_method;

#[test]
fn main() {
	launch(32 * 1024 * 1024, |runtime| {
		runtime
			.cl
			.load_jar(include_bytes!("../../../../../rt.zip"), |v| {
				v == "java/lang/Object.class"
			})
			.unwrap();

		compile(&runtime, &[("Main.java", include_str!("Main.java"))]).unwrap();

		runtime
			.linker
			.lock()
			.link("/home/alphasucks/CLionProjects/rvm/libs/rvm-tests/src/tests/jni/libnative.so");

		let java = java_bind_method!(runtime fn Main:main());
		let i = java();
	})
}
