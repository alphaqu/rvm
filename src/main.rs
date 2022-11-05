use std::fs::read;
use std::thread::Builder;

use inkwell::context::Context;

use rvm_core::init;
use rvm_runtime::Runtime;

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

	for jar in std::env::args().skip(1) {
		runtime.cl.load_jar(read(jar).unwrap(), |_| true).unwrap();
	}

	runtime
		.cl
		.load_jar(read("./rt.jar").unwrap(), |v| v == "java/lang/Object.class")
		.unwrap();

	for jar in std::env::args().skip(1) {
		runtime.cl.load_jar(read(jar).unwrap(), |_| true).unwrap();
	}
	//
	// 	let class_id = runtime
	// 		.cl
	// 		.get_class_id(&BinaryName::Object("Main".to_string()));
	//
	// 	let class_guard = runtime.cl.get(class_id);
	// 	if let ClassKind::Object(class) = &class_guard.kind {
	// 		let method_id = class
	// 			.methods
	// 			.get_id(&MethodIdentifier {
	// 				name: "test".to_string(),
	// 				descriptor: "()I".to_string(),
	// 			})
	// 			.unwrap();
	// 		drop(class_guard);
	//
	// 		let mut stack = Stack::new(1);
	// 		let mut frame = Frame::raw_frame(class_id, stack);
	// 		// args
	// 		let executor = Frame::new(class_id, method_id, &runtime, &mut frame).unwrap();
	// 		println!("{:?}", executor.execute(&runtime, method_id));
	//
	// 		let method = compile_method(
	// 			CString::new("Main").unwrap(),
	// 			CString::new("Main").unwrap(),
	// 			CString::new("Main").unwrap(),
	// 		);
	// 		let function = runtime.compile_method::<unsafe extern "C" fn() -> i32>(class_id, method_id);
	// 		println!("{:?}", unsafe {
	// 			function.call()
	// 		});
	// 		// match executor.run(&runtime) {
	// 		//             Ok(v) => {
	// 		//
	// 		//             }
	// 		//             Err(err) => {}
	// 		//         }
	// 		//         executor.run(&runtime).map_err(|e| {
	// 		//             let mut out = String::new();
	// 		//             e.fmt(&mut out, &runtime).unwrap();
	// 		//             out
	// 		//         }).unwrap();
	// 	}
}

// pub fn value_bind(runtime: &mut Runtime) {
//     runtime.load_native(
//         "ClassName".to_string(),
//         "value".to_string(),
//         "(I)L".to_string(),
//         NativeCode {
//             func: |local_table, runtime| {
//                 Ok(Some(StackValue::from(
//                     value(local_table.get(0)?)?.to_java(runtime)?,
//                 )))
//             },
//             max_locals: 1,
//         },
//     );
// }

// #[rvm_bind::method(ClassName, (I)V)]
// pub fn hi(hi: i8) -> Result<()> {
//     Ok(hi as i64)
// }
