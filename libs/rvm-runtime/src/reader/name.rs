use crate::{StrParse, ValueDesc};
use std::fmt::{Display, Formatter};

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum BinaryName {
	Object(String),
	Array(ValueDesc),
}

impl BinaryName {
	pub fn parse(s: &str) -> BinaryName {
		if s.starts_with('[') {
			let component = ValueDesc::parse(&s[1..]).unwrap();
			BinaryName::Array(component)
		} else {
			if s.is_empty() {
				panic!("empty shit");
			}
			BinaryName::Object(s.to_string())
		}
	}

	pub fn to_component(self) -> BinaryName {
		match self {
			BinaryName::Object(object) => BinaryName::Array(ValueDesc::Object(object)),
			BinaryName::Array(array) => BinaryName::Array(ValueDesc::Array(Box::new(array))),
		}
	}
}

impl Display for BinaryName {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			BinaryName::Object(value) => value.fmt(f),
			BinaryName::Array(component) => write!(f, "[{component}"),
		}
	}
}
