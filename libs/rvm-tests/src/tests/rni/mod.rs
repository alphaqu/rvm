use rvm_runtime::MethodBinding;

use crate::bindings::tests::rni::RniTests;
use crate::launch;

#[test]
pub fn basic() -> eyre::Result<()> {
	let runtime = launch(1024);

	runtime.bindings.bind(
		"tests/rni/RniTests",
		"testNative",
		MethodBinding::new(
			|_, (number_1, number_2, number_3): (i32, i64, i32)| -> i64 {
				(number_1 as i64) + number_2 * (number_3 as i64)
			},
		),
	);

	let i = RniTests::test(&runtime, 69, 50, 12)?;

	assert_eq!(i, 69 + 50 * 12);
	Ok(())
}

//pub struct RniTestsLinking;
//
// #[jni_binding(testing/rni/RniTests)]
// impl RniTestsLinking {
// 	#[jni_method]
// 	fn test_native(_: &Arc<Runtime>, number_1: i32, number_2: i64, number_3: i32) -> i64 {
// 		(number_1 as i64) + number_2 * (number_3 as i64)
// 	}
//
// 	fn hello() {}
//
// 	//fn link(runtime: &Runtime) {
// 	//	unsafe extern "C" fn test(
// 	//		runtime: *const Runtime,
// 	//		number_1: i32,
// 	//		number_2: i64,
// 	//		number_3: i32,
// 	//	) -> i64 {
// 	//		let runtime = Arc::from_raw(runtime);
// 	//		let returns = RniTestsLinking::test(&runtime, number_1, number_2, number_3);
// 	//		let _ = Arc::into_raw(runtime);
// 	//		returns
// 	//	}
// 	//
// 	//	unsafe {
// 	//		runtime.linker.lock().link(
// 	//			"Java_testing_rni_RniTests_testNative".to_string(),
// 	//			MethodBinding::new(
// 	//				transmute::<*const (), extern "C" fn()>(test as *const ()),
// 	//				MethodDescriptor {
// 	//					parameters: vec![i32::ty(), i64::ty(), i32::ty()],
// 	//					returns: Some(i64::ty()),
// 	//				},
// 	//			),
// 	//		);
// 	//	}
// 	//}
// }
