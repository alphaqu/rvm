use crate::code::Executor;
use crate::thread::{BenFrameMut, ThreadFrame};
use crate::value::StackValue;
use rvm_core::{ObjectType, PrimitiveType};
use rvm_reader::{ConstInst, ConstantInfo};
use rvm_runtime::{
	AnyValue, CallType, InstanceClass, MethodIdentifier, Reference, ReferenceKind, ThreadContext,
};
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use tracing::info;

#[derive(Debug)]
pub enum ConstTask {
	Null,
	Int(i32),
	Long(i64),
	Float(f32),
	Double(f64),
	String(String),
	Class(ObjectType),
}

impl Display for ConstTask {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"CONST {}",
			match self {
				ConstTask::Null => "null".to_string(),
				ConstTask::Int(v) => v.to_string(),
				ConstTask::Long(v) => v.to_string(),
				ConstTask::Float(v) => v.to_string(),
				ConstTask::Double(v) => v.to_string(),
				ConstTask::String(v) => format!("\"{v:?}\""),
				ConstTask::Class(v) => format!("class:{v:?}"),
			}
		)
	}
}

impl ConstTask {
	pub fn new(inst: &ConstInst, class: &InstanceClass) -> ConstTask {
		match inst {
			ConstInst::Null => ConstTask::Null,
			ConstInst::Int(v) => ConstTask::Int(*v),
			ConstInst::Long(v) => ConstTask::Long(*v),
			ConstInst::Float(v) => ConstTask::Float(*v),
			ConstInst::Double(v) => ConstTask::Double(*v),
			ConstInst::Ldc { id, cat2: _ } => {
				let info = class.cp.raw_get(*id).unwrap();
				match info {
					ConstantInfo::Integer(value) => ConstTask::Int(value.bytes),
					ConstantInfo::Float(value) => ConstTask::Float(value.bytes),
					ConstantInfo::Long(value) => ConstTask::Long(value.bytes),
					ConstantInfo::Double(value) => ConstTask::Double(value.bytes),
					ConstantInfo::String(value) => {
						ConstTask::String(class.cp[value.string].to_string())
					}
					ConstantInfo::Class(value) => {
						let string = class.cp[value.name].to_string();
						ConstTask::Class(ObjectType::new(string))
					}
					v => {
						panic!("{v:?}");
					}
				}
			}
		}
	}

	#[inline(always)]
	pub fn exec(&self, executor: &mut Executor) -> eyre::Result<()> {
		let mut frame = executor.current_frame();
		match self {
			ConstTask::Null => {
				frame.push(StackValue::Reference(Reference::NULL));
			}
			ConstTask::Int(v) => frame.push(StackValue::Int(*v)),
			ConstTask::Long(v) => frame.push(StackValue::Long(*v)),
			ConstTask::Float(v) => frame.push(StackValue::Float(*v)),
			ConstTask::Double(v) => frame.push(StackValue::Double(*v)),
			ConstTask::String(v) => {
				info!("CREATING STYRING");

				// TODO not create string instances every time ldc gets hit
				let mut runtime = executor.runtime();

				let array =
					runtime.alloc_array(&PrimitiveType::Char.into(), v.chars().count() as u32)?;

				executor.frozen_references.push(*array);
				let i = executor.frozen_references.len();

				let mut runtime = executor.runtime();

				assert_eq!((*array).reference_kind(), Some(ReferenceKind::Array));
				let id = runtime.resolve_class(&ObjectType::String().into())?;
				let class = runtime.classes.get(id);
				let class = class.to_instance();

				let string = runtime.alloc_object(class)?;

				assert_eq!(i, executor.frozen_references.len());
				executor.frozen_references.pop();
				let mut runtime = executor.runtime();

				assert_eq!((*array).reference_kind(), Some(ReferenceKind::Array));
				let _ = runtime.run(
					CallType::Special,
					&ObjectType::String(),
					&MethodIdentifier {
						name: Arc::from("<init>"),
						descriptor: Arc::from("([C)V"),
					},
					vec![AnyValue::Reference(**string), AnyValue::Reference(*array)],
				)?;

				let mut frame = executor.current_frame();
				frame.push(StackValue::Reference(**string));
			}
			ConstTask::Class(_) => {
				frame.push(StackValue::Reference(Reference::NULL));
			}
		}

		Ok(())
	}
}
