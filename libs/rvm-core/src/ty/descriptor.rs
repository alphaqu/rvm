use std::fmt::{Debug, Display, Formatter};

use crate::{PrimitiveType, Type};

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct MethodDescriptor {
	pub parameters: Vec<Type>,
	pub returns: Option<Type>,
}

impl MethodDescriptor {
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
