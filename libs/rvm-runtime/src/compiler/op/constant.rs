use crate::compiler::compiler::BlockCompiler;
use inkwell::values::BasicValue;
use std::fmt::{Display, Formatter};
use std::mem::transmute;
use rvm_core::Kind;
use rvm_reader::{ConstantInfo, ConstInst};

use crate::compiler::resolver::BlockResolver;

/// Loads a constant to the stack.
/// # Stack
/// -> value
#[derive(Debug, Clone)]
pub enum ConstTask {
	I32(i32),
	I64(i64),
	F32(f32),
	F64(f64),
	Null,
	// TODO
	String,
}

impl ConstTask {
	pub fn resolve(inst: &ConstInst, resolver: &mut BlockResolver) -> ConstTask {
		let ldc = |value: u16| -> ConstTask {
			let constant = resolver.cp().get_raw(value).unwrap();
			match constant {
				ConstantInfo::Integer(int) => ConstTask::I32(int.bytes),
				ConstantInfo::Float(float) => ConstTask::F32(float.bytes),
				ConstantInfo::String(string) => {
					//let string = string.string.get(&class.cp);
					//let id = runtime
					//	.cl
					//	.get_class_id(&BinaryName::Object("java.lang.String".to_string()));
					todo!("string constants")
				}
				ConstantInfo::Class(class) => {
					todo!("Class objects")
				}
				ConstantInfo::MethodHandle(_) => {
					todo!("method handle")
				}
				ConstantInfo::MethodType(_) => {
					todo!("method type")
				}
				_ => {
					todo!("maybe unsupported")
				}
			}
		};
		match inst {
			ConstInst::Null => ConstTask::Null,
			ConstInst::Int(v) => ConstTask::I32(*v),
			ConstInst::Long(v) => ConstTask::I64(*v),
			ConstInst::Float(v) =>  ConstTask::F32(*v),
			ConstInst::Double(v) => ConstTask::F64(*v),
			ConstInst::Ldc { id, cat2 } => {
				let constant = resolver.cp().get_raw(*id).unwrap();
				match constant {
					ConstantInfo::Integer(v) => ConstTask::I32(v.bytes),
					ConstantInfo::Float(v) => ConstTask::F32(v.bytes),
					ConstantInfo::Long(v) => ConstTask::I64(v.bytes),
					ConstantInfo::Double(v) => ConstTask::F64(v.bytes),
					ConstantInfo::String(string) => {
						//let string = string.string.get(&class.cp);
						//let id = runtime
						//	.cl
						//	.get_class_id(&BinaryName::Object("java.lang.String".to_string()));
						todo!("string constants")
					}
					ConstantInfo::Class(class) => {
						todo!("Class objects")
					}
					ConstantInfo::MethodHandle(_) => {
						todo!("method handle")
					}
					ConstantInfo::MethodType(_) => {
						todo!("method type")
					}
					_ => {
						todo!("maybe unsupported")
					}
				}
			}
		}
	}

	pub fn compile<'b, 'a>(&self, bc: &mut BlockCompiler<'b, 'a>) {
		let output = match self {
			ConstTask::I32(v) => bc
				.int()
				.const_int(unsafe { transmute::<_, u32>(*v) } as u64, false)
				.as_basic_value_enum(),
			ConstTask::I64(v) => bc
				.long()
				.const_int(unsafe { transmute::<_, u64>(*v) }, false)
				.as_basic_value_enum(),
			ConstTask::F32(v) => bc.float().const_float(*v as f64).as_basic_value_enum(),
			ConstTask::F64(v) => bc.double().const_float(*v).as_basic_value_enum(),
			ConstTask::Null => {
				todo!("ref")
			}
			ConstTask::String => {
				todo!("string")
			}
		};
		bc.push(output);
	}

	pub fn get_type(&self) -> Kind {
		match self {
			ConstTask::I32(_) => Kind::Int,
			ConstTask::I64(_) => Kind::Long,
			ConstTask::F32(_) => Kind::Float,
			ConstTask::F64(_) => Kind::Double,
			ConstTask::Null | ConstTask::String => Kind::Reference,
		}
	}
}

impl Display for ConstTask {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			ConstTask::I32(v) => write!(f, "const {v}i32"),
			ConstTask::I64(v) => write!(f, "const {v}i64"),
			ConstTask::F32(v) => write!(f, "const {v}f32"),
			ConstTask::F64(v) => write!(f, "const {v}f64"),
			ConstTask::Null => write!(f, "const null"),
			ConstTask::String => write!(f, "const str"),
		}
	}
}
