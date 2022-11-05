use std::collections::HashMap;
use std::ffi::{c_void, CStr};
use std::ptr::null_mut;
use inkwell::context::Context;

use jni_sys::{
	jint, JNIEnv, JNIInvokeInterface_, JNINativeInterface_, JavaVM, JavaVMInitArgs, JNI_ERR,
	JNI_EVERSION, JNI_OK, JNI_TRUE, JNI_VERSION_1_8,
};

use crate::Runtime;

#[no_mangle]
pub unsafe extern "system" fn JNI_GetDefaultJavaVMInitArgs(args: *mut c_void) -> jint {
	let args = &*(args as *mut JavaVMInitArgs);

	if args.version > JNI_VERSION_1_8 {
		JNI_EVERSION
	} else {
		JNI_OK
	}
}

#[no_mangle]
pub unsafe extern "system" fn JNI_CreateJavaVM(
	pvm: *mut *mut JavaVM,
	penv: *mut *mut c_void,
	args: *mut c_void,
) -> jint {
	let args = &*(args as *mut JavaVMInitArgs);

	if args.version > JNI_VERSION_1_8 {
		return JNI_EVERSION;
	}

	todo!();

	let mut context = Box::leak(Box::new(Context::create()));
	let mut runtime = Box::pin(Runtime::new(&context));
	let mut properties = HashMap::new();

	for arg in 0..usize::try_from(args.nOptions).unwrap() {
		let arg = &*(args.options.add(arg));

		match CStr::from_ptr(arg.optionString).to_str() {
			Ok("-verbose") => {}
			Ok("-verbose:class") => {}
			Ok("-verbose:gc") => {}
			Ok("-verbose:jni") => {}
			Ok("vfprintf") => {}
			Ok("exit") => {}
			Ok("abort") => {}
			Ok(x) if x.starts_with("-D") => {
				let (k, v) = x[2..].split_once('=').unwrap_or((x, ""));
				properties.insert(k.to_string(), v.to_string());
			}
			Ok(_) if args.ignoreUnrecognized == JNI_TRUE => {}
			other => {
				panic!("Unrecognised option {:?}", other);
			}
		}
	}

	// We probably want to Arc Mutex this instead
	let leak = Box::leak(Box::new(runtime));
	let jvm = JNIInvokeInterface_ {
		reserved0: leak as *mut _ as *mut c_void,
		reserved1: null_mut(),
		reserved2: null_mut(),
		DestroyJavaVM: Some(destroy_java_vm),
		AttachCurrentThread: Some(attach_current_thread),
		DetachCurrentThread: Some(detach_current_thread),
		GetEnv: Some(get_env),
		AttachCurrentThreadAsDaemon: Some(attach_current_thread_as_daemon),
	};
	*pvm = &mut (Box::leak(Box::new(jvm)) as *const JNIInvokeInterface_);

	JNI_OK
}

pub unsafe extern "system" fn destroy_java_vm(vm: *mut JavaVM) -> jint {
	JNI_ERR
}

pub unsafe extern "system" fn attach_current_thread(
	vm: *mut JavaVM,
	penv: *mut *mut c_void,
	args: *mut c_void,
) -> jint {
	JNI_ERR
}

pub unsafe extern "system" fn detach_current_thread(vm: *mut JavaVM) -> jint {
	JNI_ERR
}

pub unsafe extern "system" fn get_env(
	vm: *mut JavaVM,
	penv: *mut *mut c_void,
	version: jint,
) -> jint {
	JNI_ERR
}

pub unsafe extern "system" fn attach_current_thread_as_daemon(
	vm: *mut JavaVM,
	penv: *mut *mut c_void,
	args: *mut c_void,
) -> jint {
	JNI_ERR
}
