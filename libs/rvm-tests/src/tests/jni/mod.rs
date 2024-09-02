use rvm_runtime::java_bind_method;

use crate::{compile, launch};

//#[test]
//fn main() {
//	let runtime = launch(1024, vec![]);
//	compile(&runtime, &[("Main.java", include_str!("Main.java"))]).unwrap();
//
//	runtime.linker.lock().link_library(
//		"/home/alphasucks/CLionProjects/rvm/libs/rvm-tests/src/tests/jni/libnative.so",
//	);
//
//	let java = java_bind_method!(runtime fn Main:main());
//	let _ = java();
//}
