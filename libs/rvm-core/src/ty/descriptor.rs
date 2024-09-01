use std::fmt::{Display, Formatter};

use crate::Type;

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
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
