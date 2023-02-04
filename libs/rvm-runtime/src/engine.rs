use std::ffi::c_void;
use std::pin::Pin;
use std::sync::Arc;
use rvm_object::MethodData;
use rvm_reader::ConstantPool;
use crate::Runtime;

pub trait Engine {
    fn compile_method(
        &self,
        runtime: &Pin<&Runtime>,
        method: &MethodData,
        cp: &Arc<ConstantPool>,
    ) -> *const c_void;
}