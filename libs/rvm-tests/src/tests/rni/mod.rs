use std::mem::transmute;
use std::sync::Arc;

use rvm_core::{MethodDescriptor, PrimitiveType, Type};
use rvm_runtime::{bind, MethodBinding, Runtime};

use crate::{launch, load_sdk};

pub struct RniTests;
bind!("testing/rni" {
	RniTests {
		test(number_1: i32, number_2: i64, number_3: i32) -> i64
	}
});

fn runtime() -> Arc<Runtime> {
	let runtime = launch(1024, vec!["testing/rni/RniTests.class"]);
	load_sdk(&runtime);
	runtime
}

pub struct RniTestsLinking;

impl RniTestsLinking {
	fn test(runtime: &Arc<Runtime>, number_1: i32, number_2: i64, number_3: i32) -> i64 {
		println!("{:?}", runtime.started.elapsed());
		(number_1 as i64) + number_2 * (number_3 as i64)
	}

	fn link(runtime: &Runtime) {
		unsafe extern "C" fn test(
			runtime: *const Runtime,
			number_1: i32,
			number_2: i64,
			number_3: i32,
		) -> i64 {
			let runtime = Arc::from_raw(runtime);
			let returns = RniTestsLinking::test(&runtime, number_1, number_2, number_3);
			let _ = Arc::into_raw(runtime);
			returns
		}

		unsafe {
			runtime.linker.lock().link(
				"Java_testing_rni_RniTests_testNative".to_string(),
				MethodBinding::new(
					transmute::<*const (), extern "C" fn()>(test as *const ()),
					MethodDescriptor {
						parameters: vec![
							Type::Primitive(PrimitiveType::Int),
							Type::Primitive(PrimitiveType::Long),
							Type::Primitive(PrimitiveType::Int),
						],
						returns: Some(Type::Primitive(PrimitiveType::Long)),
					},
				),
			);
		}
	}
}

#[test]
pub fn basic() {
	let runtime = runtime();

	RniTestsLinking::link(&runtime);

	let tests = RniTests::test(&runtime);

	let i = tests(69, 50, 12);

	assert_eq!(i, 69 + 50 * 12);
}
