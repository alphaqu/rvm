use std::fs::read;
use std::path::Path;
use std::thread::Builder;

use inkwell::context::Context;

use rvm_core::init;
use rvm_runtime::{java, Runtime};

fn main() {
	Builder::new()
		.name("hi".to_string())
		.stack_size(1024 * 1024 * 64)
		.spawn(|| {
			run();
		})
		.unwrap()
		.join()
		.unwrap();
}

fn run() {
	init();
	let context = Context::create();
	let runtime = Box::pin(Runtime::new(&context));

	// 	// bind
	// 	{
	// 		// bindhi(&mut runtime);
	// 		RUNTIME.cl.register_native(
	// 			"Main".to_string(),
	// 			MethodIdentifier {
	// 				name: "hi".to_string(),
	// 				descriptor: "(I)V".to_string(),
	// 			},
	// 			NativeCode {
	// 				func: |local_table, runtime| {
	// 					println!("{:?}", local_table.get_raw(0));
	// 					Ok(None)
	// 				},
	// 				max_locals: 1,
	// 			},
	// 		);
	// 		RUNTIME.cl.register_native(
	// 			"java/lang/Object".to_string(),
	// 			MethodIdentifier {
	// 				name: "registerNatives".to_string(),
	// 				descriptor: "()V".to_string(),
	// 			},
	// 			NativeCode {
	// 				func: |local_table, runtime| {
	// 					println!("Object registered natives");
	// 					Ok(None)
	// 				},
	// 				max_locals: 1,
	// 			},
	// 		);
	//
	// 		fn fake_define(runtime: &mut Runtime, class_name: &str, name: &str, desc: &str) {
	// 			runtime.cl.register_native(
	// 				class_name.to_string(),
	// 				MethodIdentifier {
	// 					name: name.to_string(),
	// 					descriptor: desc.to_string(),
	// 				},
	// 				NativeCode {
	// 					func: |local_table, runtime| Ok(None),
	// 					max_locals: 1,
	// 				},
	// 			);
	// 		}
	// 		fake_define(&mut RUNTIME, "java/lang/Object", "hashCode", "()I");
	// 		fake_define(
	// 			&mut RUNTIME,
	// 			"java/lang/Object",
	// 			"getClass",
	// 			"()Ljava/lang/Class;",
	// 		);
	// 		fake_define(
	// 			&mut RUNTIME,
	// 			"java/lang/Object",
	// 			"clone",
	// 			"()Ljava/lang/Object;",
	// 		);
	// 		fake_define(&mut runtime, "java/lang/Object", "notify", "()V");
	// 		fake_define(&mut runtime, "java/lang/Object", "notifyAll", "()V");
	// 		fake_define(&mut runtime, "java/lang/Object", "wait", "(J)V");
	// 	}

	runtime
		.cl
		.load_jar(include_bytes!("../rt.jar"), |v| v == "java/lang/Object.class")
		.unwrap();

	for jar in std::env::args().skip(1) {
		let path = Path::new(&jar);

		match path.extension().and_then(|x| x.to_str()) {
			Some("jar") | Some("zip") => {
				runtime.cl.load_jar(&read(path).unwrap(), |_| true).unwrap();
			}
			Some("class") => {
				runtime.cl.load_class(&read(path).unwrap()).unwrap();
			}
			other => {
				panic!("Unrecognised extension {other:?}");
			}
		}
	}

	// TODO: ARRAYS PLEASE
	unsafe { java!(compile &runtime.as_ref(), fn Main.main() -> ())() };
}
