use crate::object::ValueType;
use crate::reader::descriptor::ReturnDescriptor::Field;
use inkwell::context::Context;
use inkwell::types::{AnyType, AnyTypeEnum, BasicMetadataTypeEnum, BasicType, FunctionType};
use std::fmt::{Display, Formatter, Write};

pub trait StrParse: Sized {
	fn parse(desc: &str) -> Option<Self>;
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct FieldDescriptor(pub ValueDesc);

impl StrParse for FieldDescriptor {
	fn parse(desc: &str) -> Option<FieldDescriptor> {
		Some(FieldDescriptor(ValueDesc::parse(desc)?))
	}
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct ParameterDescriptor(pub ValueDesc);

impl StrParse for ParameterDescriptor {
	fn parse(desc: &str) -> Option<ParameterDescriptor> {
		Some(ParameterDescriptor(ValueDesc::parse(desc)?))
	}
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum ReturnDescriptor {
	Field(ValueDesc),
	Void,
}

impl StrParse for ReturnDescriptor {
	fn parse(desc: &str) -> Option<ReturnDescriptor> {
		return if desc.as_bytes()[0] == b'V' {
			Some(ReturnDescriptor::Void)
		} else {
			Some(Field(ValueDesc::parse(desc)?))
		};
	}
}

impl ReturnDescriptor {
	pub fn func<'ctx>(
		&self,
		ctx: &'ctx Context,
		param_types: &[BasicMetadataTypeEnum<'ctx>],
	) -> FunctionType<'ctx> {
		match self {
			Field(value) => value.ty().ir(ctx).fn_type(param_types, false),
			ReturnDescriptor::Void => ctx.void_type().fn_type(param_types, false),
		}
	}

	pub fn is_void(&self) -> bool {
		match self {
			ReturnDescriptor::Void => true,
			_ => false,
		}
	}
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct MethodDescriptor {
	pub parameters: Vec<ParameterDescriptor>,
	pub ret: ReturnDescriptor,
}

impl MethodDescriptor {
	pub fn func<'ctx>(&self, ctx: &'ctx Context) -> FunctionType<'ctx> {
		let param_types: Vec<BasicMetadataTypeEnum> = self
			.parameters
			.iter()
			.map(|v| BasicMetadataTypeEnum::from(v.0.ty().ir(ctx)))
			.collect();
		
		match &self.ret {
			ReturnDescriptor::Field(ty) => ty.ty().ir(ctx).fn_type(&param_types, false),
			ReturnDescriptor::Void => ctx.void_type().fn_type(&param_types, false),
		}
	}
}

impl StrParse for MethodDescriptor {
	fn parse(desc: &str) -> Option<MethodDescriptor> {
		let end = desc.find(')')?;
		let mut remaining = &desc[1..end];
		let mut parameters = Vec::new();

		let desc1 = &desc[end + 1..];
		let ret = ReturnDescriptor::parse(desc1)?;
		while !remaining.is_empty() {
			let (parameter, size) = ValueDesc::parse_len(remaining)?;
			parameters.push(ParameterDescriptor(parameter));
			remaining = &remaining[size..];
		}

		Some(MethodDescriptor { parameters, ret })
	}
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum ValueDesc {
	Base(BaseDesc),
	Object(String),
	Array(Box<ValueDesc>),
}

impl StrParse for ValueDesc {
	fn parse(desc: &str) -> Option<Self> {
		Self::parse_len(desc).map(|(ty, _)| ty)
	}
}

impl ValueDesc {
	pub fn parse_len(desc: &str) -> Option<(ValueDesc, usize)> {
		Some(match desc.as_bytes()[0] {
			b'L' => {
				let end = desc.find(';')?;
				(ValueDesc::Object(desc[1..end].to_string()), end + 1)
			}
			b'[' => {
				let (component, len) = ValueDesc::parse_len(&desc[1..])?;
				(ValueDesc::Array(Box::new(component)), 1 + len)
			}
			_ => {
				if let Some(base) = BaseDesc::parse(desc).map(|v| (ValueDesc::Base(v), 1)) {
					return Some(base);
				} else {
					let end = desc.find(';')?;
					(ValueDesc::Object(desc[1..end].to_string()), end + 1)
				}
			}
		})
	}

	pub fn ty(&self) -> ValueType {
		match self {
			ValueDesc::Base(base) => base.ty(),
			ValueDesc::Array(_) | ValueDesc::Object(_) => ValueType::Reference,
		}
	}
}

impl Display for ValueDesc {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			ValueDesc::Base(base) => base.fmt(f),
			ValueDesc::Object(object) => {
				f.write_char('L')?;
				f.write_str(object)?;
				f.write_char(';')
			}
			ValueDesc::Array(component) => {
				f.write_char('[')?;
				component.fmt(f)
			}
		}
	}
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum BaseDesc {
	Boolean,
	Byte,
	Short,
	Int,
	Long,
	Char,
	Float,
	Double,
}

impl BaseDesc {
	pub fn parse(desc: &str) -> Option<BaseDesc> {
		Some(match desc.as_bytes()[0] {
			b'Z' => BaseDesc::Boolean,
			b'B' => BaseDesc::Byte,
			b'C' => BaseDesc::Char,
			b'D' => BaseDesc::Double,
			b'F' => BaseDesc::Float,
			b'I' => BaseDesc::Int,
			b'J' => BaseDesc::Long,
			b'S' => BaseDesc::Short,
			_ => {
				return None;
			}
		})
	}

	pub fn char(&self) -> char {
		match self {
			BaseDesc::Boolean => 'C',
			BaseDesc::Byte => 'B',
			BaseDesc::Short => 'S',
			BaseDesc::Int => 'I',
			BaseDesc::Long => 'J',
			BaseDesc::Char => 'C',
			BaseDesc::Float => 'F',
			BaseDesc::Double => 'D',
		}
	}

	pub fn ty(&self) -> ValueType {
		match self {
			BaseDesc::Boolean => ValueType::Boolean,
			BaseDesc::Byte => ValueType::Byte,
			BaseDesc::Short => ValueType::Short,
			BaseDesc::Int => ValueType::Int,
			BaseDesc::Long => ValueType::Long,
			BaseDesc::Char => ValueType::Char,
			BaseDesc::Float => ValueType::Float,
			BaseDesc::Double => ValueType::Double,
		}
	}
}

impl Display for BaseDesc {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.write_char(self.char())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parse_method() {
		assert_eq!(
			MethodDescriptor::parse("(IDLjava/lang/Thread;)Ljava/lang/Object;"),
			Some(MethodDescriptor {
				parameters: vec![
					ParameterDescriptor(ValueDesc::Base(BaseDesc::Int)),
					ParameterDescriptor(ValueDesc::Base(BaseDesc::Double)),
					ParameterDescriptor(ValueDesc::Object("java/lang/Thread".to_string()))
				],
				ret: Field(ValueDesc::Object("java/lang/Object".to_string()))
			})
		);
	}

	#[test]
	fn parse_primitives() {
		assert_eq!(ValueDesc::parse("B"), Some(ValueDesc::Base(BaseDesc::Byte)));
		assert_eq!(ValueDesc::parse("C"), Some(ValueDesc::Base(BaseDesc::Char)));
		assert_eq!(
			ValueDesc::parse("D"),
			Some(ValueDesc::Base(BaseDesc::Double))
		);
		assert_eq!(
			ValueDesc::parse("F"),
			Some(ValueDesc::Base(BaseDesc::Float))
		);
		assert_eq!(ValueDesc::parse("I"), Some(ValueDesc::Base(BaseDesc::Int)));
		assert_eq!(ValueDesc::parse("J"), Some(ValueDesc::Base(BaseDesc::Long)));
		assert_eq!(
			ValueDesc::parse("S"),
			Some(ValueDesc::Base(BaseDesc::Short))
		);
		assert_eq!(
			ValueDesc::parse("Z"),
			Some(ValueDesc::Base(BaseDesc::Boolean))
		);
	}

	#[test]
	fn parse_object() {
		assert_eq!(
			ValueDesc::parse("Ljava/lang/Object;"),
			Some(ValueDesc::Object("java/lang/Object".to_string()))
		);
		assert_eq!(
			ValueDesc::parse("LHalo;"),
			Some(ValueDesc::Object("Halo".to_string()))
		);
		assert_eq!(
			ValueDesc::parse("L;"),
			Some(ValueDesc::Object("".to_string()))
		);
	}

	#[test]
	fn parse_array() {
		assert_eq!(
			ValueDesc::parse("[B"),
			Some(ValueDesc::Array(Box::new(ValueDesc::Base(BaseDesc::Byte))))
		);
		assert_eq!(
			ValueDesc::parse("[[B"),
			Some(ValueDesc::Array(Box::new(ValueDesc::Array(Box::new(
				ValueDesc::Base(BaseDesc::Byte)
			)))))
		);
		assert_eq!(
			ValueDesc::parse("[Ljava/lang/Object;"),
			Some(ValueDesc::Array(Box::new(ValueDesc::Object(
				"java/lang/Object".to_string()
			))))
		);
	}
}
