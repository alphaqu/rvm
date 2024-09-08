use rvm_macro::{jni_binding, jni_method};
use rvm_runtime::{
	ClassSource, DirectoryClassSource, JarClassSource, MethodBinding, MethodIdentifier, Vm,
};
use std::path::PathBuf;
use std::sync::Arc;

pub fn load_test_sdk(runtime: &Vm) {
	runtime.classes.add_source(Box::new(
		DirectoryClassSource::new(PathBuf::from("bytecode")).unwrap(),
	));
	runtime.bindings.bind(
		"core/Assert",
		"yes",
		MethodBinding::new(|_, value: bool| {
			assert!(value);
		}),
	);

	macro_rules! assert_bind {
		($TY:ty) => {
			runtime.bindings.bind(
				"core/Assert",
				"eq",
				MethodBinding::new(|_, (left, right): ($TY, $TY)| {
					assert_eq!(left, right);
				}),
			);
		};
	}

	assert_bind!(i32);
	assert_bind!(i64);
	assert_bind!(f32);
	assert_bind!(f64);
}

pub struct AssertBindings {}

impl AssertBindings {}

#[jni_binding(core/Assert)]
impl AssertBindings {
	#[jni_method]
	pub fn yes(_: &Arc<Vm>, value: bool) {
		assert!(value);
	}

	#[jni_method(eq)]
	pub fn eq_i32(_: &Arc<Vm>, v0: i32, v1: i32) {
		assert_eq!(v0, v1);
	}

	#[jni_method(eq)]
	pub fn eq_i64(_: &Arc<Vm>, v0: i64, v1: i64) {
		assert_eq!(v0, v1);
	}
	#[jni_method(eq)]
	pub fn eq_f32(_: &Arc<Vm>, v0: f32, v1: f32) {
		assert_eq!(v0, v1);
	}

	#[jni_method(eq)]
	pub fn eq_f64(_: &Arc<Vm>, v0: f64, v1: f64) {
		assert_eq!(v0, v1);
	}
}
