use std::fs::read;
use std::path::Path;
use std::sync::Arc;
use std::thread::Builder;

use inkwell::context::Context;

use rvm_core::{init, ObjectType, Type};
use rvm_engine_ben::BenBinding;
use rvm_object::{Class, DynValue, MethodIdentifier};
use rvm_runtime::engine::ThreadConfig;
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
	let engine = Box::new(BenBinding::new());

	let runtime = Arc::new(Runtime::new(1024 * 1024, engine));
	runtime
		.cl
		.load_jar(include_bytes!("../rt.zip"), |v| {
			v == "java/lang/Object.class"
		})
		.unwrap();
	runtime
		.cl
		.load_jar(include_bytes!("../unnamed.jar"), |v| true)
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

	let handle = runtime.engine.create_thread(
		runtime.clone(),
		ThreadConfig {
			name: "Hi".to_string(),
		},
	);

	handle.run(
		ObjectType("Main".to_string()),
		MethodIdentifier {
			name: "main".to_string(),
			descriptor: "()I".to_string(),
		},
		vec![],
	);

	println!("{:?}", handle.join().unwrap());
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

	//let id = runtime.cl.get_class_id(&Type::Object(ObjectType {
	//	name: "Main".to_string(),
	//}));
	//let id_child = runtime.cl.get_class_id(&Type::Object(ObjectType {
	//	name: "Child".to_string(),
	//}));
	//
	//let child = Object::new(&runtime, id_child, [
	//	("haha", DynValue::Int(69))
	//]);
	//let object = Object::new(&runtime, id, [
	//	("thing", DynValue::Int(2)),
	//	("child", DynValue::Ref(child.reference))
	//]);
	//
	//println!("{:?}", object.get_dyn_field("thing"));
	//println!("{:?}", object.get_dyn_field("child"));
	//println!("{:?}", child.get_dyn_field("haha"));
	//
	//
	//runtime.arena.gc();
	//println!("{:?}", object.get_dyn_field("thing"));
	//println!("{:?}", object.get_dyn_field("child"));
	//println!("{:?}", child.get_dyn_field("haha"));
	//// TODO: ARRAYS PLEASE
	//let value = unsafe { java!(compile &runtime.as_ref(), fn Main.main() -> i32)() };
	//println!("{value}");
}
