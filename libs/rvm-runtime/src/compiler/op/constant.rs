use crate::compiler::compiler::BlockCompiler;
use inkwell::values::BasicValue;
use std::fmt::{Display, Formatter};
use std::mem::transmute;

use crate::compiler::resolver::BlockResolver;
use crate::executor::Inst;
use crate::object::ValueType;
use crate::reader::ConstantInfo;

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
	pub fn resolve(inst: &Inst, resolver: &mut BlockResolver) -> ConstTask {
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
			Inst::BIPUSH(v) => ConstTask::I32(*v as i32),
			Inst::SIPUSH(v) => ConstTask::I32(*v as i32),
			Inst::ACONST_NULL => ConstTask::Null,
			Inst::DCONST_0 => ConstTask::F64(0.0),
			Inst::DCONST_1 => ConstTask::F64(1.0),
			Inst::FCONST_0 => ConstTask::F32(0.0),
			Inst::FCONST_1 => ConstTask::F32(1.0),
			Inst::FCONST_2 => ConstTask::F32(2.0),
			Inst::ICONST_M1 => ConstTask::I32(-1),
			Inst::ICONST_0 => ConstTask::I32(0),
			Inst::ICONST_1 => ConstTask::I32(1),
			Inst::ICONST_2 => ConstTask::I32(2),
			Inst::ICONST_3 => ConstTask::I32(3),
			Inst::ICONST_4 => ConstTask::I32(4),
			Inst::ICONST_5 => ConstTask::I32(5),
			Inst::LCONST_0 => ConstTask::I64(0),
			Inst::LCONST_1 => ConstTask::I64(1),
			Inst::LDC(value) => ldc(*value as u16),
			Inst::LDC_W(value) => ldc(*value),
			Inst::LDC2_W(value) => {
				let constant = resolver.cp().get_raw(*value).unwrap();
				match constant {
					ConstantInfo::Float(float) => ConstTask::F32(float.bytes),
					ConstantInfo::Double(double) => ConstTask::F64(double.bytes),
					_ => {
						todo!("maybe unsupported")
					}
				}
			}
			_ => {
				panic!("wtf")
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

	pub fn get_type(&self) -> ValueType {
		match self {
			ConstTask::I32(_) => ValueType::Int,
			ConstTask::I64(_) => ValueType::Long,
			ConstTask::F32(_) => ValueType::Float,
			ConstTask::F64(_) => ValueType::Double,
			ConstTask::Null | ConstTask::String => ValueType::Reference,
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
