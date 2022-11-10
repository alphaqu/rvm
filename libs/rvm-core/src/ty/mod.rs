use std::fmt::{Display, Formatter, Write};
pub use kind::*;
pub use value::*;
pub use desc::*;
pub use flags::*;
pub use op::*;
pub(crate) use value::read_arr;
pub(crate) use value::write_arr;
mod kind;
mod value;
mod desc;
mod op;
mod flags;

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum Type {
	Primitive(PrimitiveType),
	Object(ObjectType),
	Array(Box<ArrayType>),
}

impl Type {
	pub fn parse(desc: &str) -> Option<Type> {
		Self::parse_len(desc).map(|(v, _)| v)
	}

	pub fn parse_len(desc: &str) -> Option<(Type, usize)> {
		match desc.as_bytes()[0] {
			b'L' => ObjectType::parse_len(desc).map(|(ty, l)| (Type::Object(ty), l)),
			b'[' => ArrayType::parse_len(desc).map(|(ty, l)| (Type::Array(Box::new(ty)), l)),
			_ => PrimitiveType::parse(desc).map(|v| (Type::Primitive(v), 1)),
		}
	}

	pub fn kind(&self) -> Kind {
		match self {
			Type::Primitive(prim) => prim.kind(),
			Type::Array(_) | Type::Object(_) => Kind::Reference,
		}
	}
}

impl Display for Type {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Type::Primitive(v) => v.fmt(f),
			Type::Object(v) => v.fmt(f),
			Type::Array(v) => v.fmt(f),
		}
	}
}

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum PrimitiveType {
	Boolean,
	Byte,
	Short,
	Int,
	Long,
	Char,
	Float,
	Double,
}

impl PrimitiveType {
	pub fn parse(desc: &str) -> Option<PrimitiveType> {
		Some(match desc.as_bytes()[0] {
			b'Z' => PrimitiveType::Boolean,
			b'B' => PrimitiveType::Byte,
			b'C' => PrimitiveType::Char,
			b'D' => PrimitiveType::Double,
			b'F' => PrimitiveType::Float,
			b'I' => PrimitiveType::Int,
			b'J' => PrimitiveType::Long,
			b'S' => PrimitiveType::Short,
			_ => {
				return None;
			}
		})
	}

	pub fn char(&self) -> char {
		match self {
			PrimitiveType::Boolean => 'C',
			PrimitiveType::Byte => 'B',
			PrimitiveType::Short => 'S',
			PrimitiveType::Int => 'I',
			PrimitiveType::Long => 'J',
			PrimitiveType::Char => 'C',
			PrimitiveType::Float => 'F',
			PrimitiveType::Double => 'D',
		}
	}

	pub fn kind(&self) -> Kind {
		match self {
			PrimitiveType::Boolean => Kind::Boolean,
			PrimitiveType::Byte => Kind::Byte,
			PrimitiveType::Short => Kind::Short,
			PrimitiveType::Int => Kind::Int,
			PrimitiveType::Long => Kind::Long,
			PrimitiveType::Char => Kind::Char,
			PrimitiveType::Float => Kind::Float,
			PrimitiveType::Double => Kind::Double,
		}
	}
}

impl Display for PrimitiveType {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.write_char(self.char())
	}
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct ObjectType {
	pub name: String,
}

impl ObjectType {
	pub fn parse_len(string: &str) -> Option<(ObjectType, usize)> {
		if string.as_bytes()[0] != b'L' {
			return None;
		}

		let end = string.find(';')?;
		Some((
			ObjectType {
				name: string[1..end].to_string(),
			},
			end + 1,
		))
	}

	pub fn kind(&self) -> Kind {
		Kind::Reference
	}
}

impl Display for ObjectType {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.write_char('L')?;
		f.write_str(&self.name)?;
		f.write_char(';')
	}
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct ArrayType {
	pub component: Type,
}

impl ArrayType {
	pub fn parse_len(string: &str) -> Option<(ArrayType, usize)> {
		let bytes = string.as_bytes();
		if bytes[0] != b'[' {
			return None;
		}

		let (component, length) = Type::parse_len(&string[1..])?;
		Some((ArrayType { component }, length + 1))
	}

	pub fn kind(&self) -> Kind {
		Kind::Reference
	}
}

impl Display for ArrayType {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.write_char('[')?;
		self.component.fmt(f)
	}
}
