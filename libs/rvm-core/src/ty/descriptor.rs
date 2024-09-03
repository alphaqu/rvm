use std::fmt::{Debug, Display, Formatter};

use crate::{PrimitiveType, Type};

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct MethodDescriptor {
	pub parameters: Vec<Type>,
	pub returns: Option<Type>,
}

impl MethodDescriptor {
	pub fn jni_short_name(class_name: &str, method_name: &str) -> String {
		Self::jni_name(class_name, method_name, None)
	}

	pub fn jni_long_name(class_name: &str, method_name: &str, arguments: &[Type]) -> String {
		Self::jni_name(class_name, method_name, Some(arguments))
	}
	fn jni_name(class_name: &str, method_name: &str, signature: Option<&[Type]>) -> String {
		let mut out = String::new();
		out.push_str("Java_");

		let push_mangled = |out: &mut String, string: &str| {
			for char in string.chars() {
				if char == '/' {
					out.push('_')
				} else if char == '_' {
					out.push_str("_1")
				} else if char == ';' {
					out.push_str("_2")
				} else if char == '[' {
					out.push_str("_3")
				} else if char == '<' || char == '>' {
					// THIS IS NOT JNI SPECC!!!
					out.push_str("_");
				} else if char.is_ascii_alphanumeric() {
					out.push(char);
				} else {
					panic!("Unsupported rn {char}!!")
				}
			}
		};

		push_mangled(&mut out, class_name);
		out.push('_');
		push_mangled(&mut out, method_name);

		if let Some(arguments) = signature {
			out.push_str("__");
			for argument in arguments {
				push_mangled(&mut out, &format!("{argument}"));
			}
		}
		out
	}

	pub fn parse(desc: &str) -> Option<MethodDescriptor> {
		let end = desc.find(')')?;
		let mut remaining = &desc[1..end];
		let mut parameters = Vec::new();

		let ret_desc = &desc[end + 1..];
		let ret = if ret_desc.as_bytes()[0] == b'V' {
			None
		} else {
			Some(Type::parse(ret_desc)?)
		};

		while !remaining.is_empty() {
			let (parameter, size) = Type::parse_len(remaining)?;
			parameters.push(parameter);
			remaining = &remaining[size..];
		}

		Some(MethodDescriptor {
			parameters,
			returns: ret,
		})
	}
}
impl Display for MethodDescriptor {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "(")?;
		for ty in &self.parameters {
			write!(f, "{ty}")?;
		}
		write!(f, ")")?;
		match &self.returns {
			None => {
				write!(f, "V")
			}
			Some(ty) => {
				write!(f, "{ty}")
			}
		}
	}
}
impl Debug for MethodDescriptor {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "(")?;
		for ty in &self.parameters {
			write!(f, "{ty:?}")?;
		}
		write!(f, ")")?;
		match &self.returns {
			None => {
				write!(f, "")
			}
			Some(ty) => {
				write!(f, " -> {ty:?}")
			}
		}
	}
}

pub trait Typed {
	fn ty() -> Type;
}

macro_rules! impl_typed_prim {
	($TY:ty => $VARIANT:ident) => {
		impl Typed for $TY {
			fn ty() -> Type {
				PrimitiveType::$VARIANT.into()
			}
		}
	};
}

impl_typed_prim!(bool => Boolean);
impl_typed_prim!(i8 => Byte);
impl_typed_prim!(i16 => Short);
impl_typed_prim!(i32 => Int);
impl_typed_prim!(i64 => Long);
impl_typed_prim!(f32 => Float);
impl_typed_prim!(f64 => Double);
