use rvm_macro::{jni_binding, jni_method};
use rvm_runtime::{MethodBinding, MethodIdentifier, Runtime};
use std::sync::Arc;

pub fn load_test_core(runtime: &Runtime) {
	runtime.bindings.bind(
		"core/Assert",
		"yes",
		MethodBinding::new(|runtime, value: bool| {
			assert!(value);
		}),
	);
}

pub struct AssertBindings {}

impl AssertBindings {}

#[jni_binding(core/Assert)]
impl AssertBindings {
	#[jni_method]
	pub fn yes(_: &Arc<Runtime>, value: bool) {
		assert!(value);
	}

	#[jni_method(eq)]
	pub fn eq_i32(_: &Arc<Runtime>, v0: i32, v1: i32) {
		assert_eq!(v0, v1);
	}

	#[jni_method(eq)]
	pub fn eq_i64(_: &Arc<Runtime>, v0: i64, v1: i64) {
		assert_eq!(v0, v1);
	}
	#[jni_method(eq)]
	pub fn eq_f32(_: &Arc<Runtime>, v0: f32, v1: f32) {
		assert_eq!(v0, v1);
	}

	#[jni_method(eq)]
	pub fn eq_f64(_: &Arc<Runtime>, v0: f64, v1: f64) {
		assert_eq!(v0, v1);
	}
}
