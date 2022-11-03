use std::fs::read;
use std::mem::transmute;
use std::thread::Builder;
use std::time::Instant;

use inkwell::context::Context;
use tracing::info;

use rvm_core::init;
use rvm_runtime::{CringeContext, Runtime};

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

#[inline(always)]
pub extern "C" fn ack(m: i32, n: i32) -> i32 {
	return if m == 0 {
		n + 1
	} else if m > 0 && n == 0 {
		ack(m - 1, 1)
	} else if m > 0 && n > 0 {
		ack(m - 1, ack(m, n - 1))
	} else {
		n + 1
	};
}

macro_rules! java {
	(()) => {"V"};
    (bool) => {"Z"};
    (i8) => {"B"};
    (i16) => {"S"};
    (i32) => {"I"};
    (f32) => {"F"};
    (i64) => {"J"};
    (f64) => {"D"};

	(descriptor $return:tt) => {
		concat!("()", java!($return))
	};
	(descriptor $return:tt, $($param:tt),+) => {
		concat!("(", $(java!($param)),+, ")", java!($return))
	};

	(compile $runtime:expr, $class:expr, fn $name:ident() -> $return:tt) => {
		transmute::<_, unsafe extern "C" fn() -> $return>($runtime.compile_method($class, stringify!($name), java!(descriptor $return)))
	};

	(compile $runtime:expr, $class:expr, fn $name:ident($($param:tt),+) -> $return:tt) => {
		transmute::<_, unsafe extern "C" fn($($param),+) -> $return>($runtime.compile_method($class, stringify!($name), java!(descriptor $return, $($param),+)))
	};
}

fn run() {
	init();
	let context = Box::pin(CringeContext(Context::create()));
	let runtime = Box::pin(Runtime::new(&context));

	{
		let start = Instant::now();
		let i = ack(3, 12);
		println!("{} in {}ms", i, start.elapsed().as_millis());
	}

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

	unsafe {
		// v == 0
		let eq = java!(compile runtime, "Main", fn testZeroEq(i32) -> bool);
		assert!(eq(0));
		assert!(!eq(1));
		assert!(!eq(-1));
		assert!(!eq(i32::MIN));
		assert!(!eq(i32::MAX));

		// v != 0
		let neq = java!(compile runtime, "Main", fn testZeroNeq(i32) -> bool);
		assert!(!neq(0));
		assert!(neq(1));
		assert!(neq(-1));
		assert!(neq(i32::MIN));
		assert!(neq(i32::MAX));

		// v > 0
		let gt = java!(compile runtime, "Main", fn testZeroGt(i32) -> bool);
		assert!(!gt(0));
		assert!(!gt(-1));
		assert!(!gt(i32::MIN));
		assert!(gt(1));
		assert!(gt(i32::MAX));

		// v >= 0
		let ge = java!(compile runtime, "Main", fn testZeroGe(i32) -> bool);
		assert!(!ge(-1));
		assert!(!ge(i32::MIN));
		assert!(ge(0));
		assert!(ge(1));
		assert!(ge(i32::MAX));

		// v < 0
		let lt = java!(compile runtime, "Main", fn testZeroLt(i32) -> bool);
		assert!(!lt(0));
		assert!(!lt(1));
		assert!(!lt(i32::MAX));
		assert!(lt(-1));
		assert!(lt(i32::MIN));

		// v <= 0
		let le = java!(compile runtime, "Main", fn testZeroLe(i32) -> bool);
		assert!(!le(1));
		assert!(!le(i32::MAX));
		assert!(le(0));
		assert!(le(-1));
		assert!(le(i32::MIN));

		info!("Invoking");
		let start = Instant::now();
		let func = java!(compile runtime, "Main", fn test() -> i32);
		let i = func();
		println!("{} in {}ms", i, start.elapsed().as_millis());

		let func = java!(compile runtime, "Main", fn test() -> i32);
		let start = Instant::now();
		let i = func();
		println!("{} in {}ms", i, start.elapsed().as_millis());
	}

	// runtime
	// 		.cl
	// 		.load_jar(read("./rt.jar").unwrap(), |v| v == "java/lang/Object.class")
	// 		.unwrap();
	//
	// 	for jar in std::env::args().skip(1) {
	// 		runtime.cl.load_jar(read(jar).unwrap(), |_| true).unwrap();
	// 	}
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
