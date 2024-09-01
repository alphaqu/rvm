pub use descriptor::*;
pub use flags::*;
pub use kind::*;
pub use op::*;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Write};
use thiserror::Error;

mod descriptor;
mod flags;
mod kind;
mod op;

/// A Type holds concrete information about the type we are handling.
/// [Kind] is similar, but has no idea what the concrete implementation is.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Type {
	Primitive(PrimitiveType),
	Object(ObjectType),
	Array(ArrayType),
}

impl Type {
	pub fn parse(desc: &str) -> Option<Type> {
		Self::parse_len(desc).map(|(v, _)| v)
	}

	pub fn parse_len(desc: &str) -> Option<(Type, usize)> {
		match desc.as_bytes()[0] {
			b'L' => ObjectType::parse_len(desc).map(|(ty, l)| (Type::Object(ty), l)),
			b'[' => ArrayType::parse_len(desc).map(|(ty, l)| (Type::Array(ty), l)),
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

impl Debug for Type {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Type::Primitive(v) => Debug::fmt(v, f),
			Type::Object(v) => Debug::fmt(v, f),
			Type::Array(v) => Debug::fmt(v, f),
		}
	}
}

impl From<PrimitiveType> for Type {
	fn from(value: PrimitiveType) -> Self {
		Type::Primitive(value)
	}
}

impl From<ObjectType> for Type {
	fn from(value: ObjectType) -> Self {
		Type::Object(value)
	}
}

impl From<ArrayType> for Type {
	fn from(value: ArrayType) -> Self {
		Type::Array(value)
	}
}

impl Display for Type {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Type::Primitive(v) => Display::fmt(v, f),
			Type::Object(v) => Display::fmt(v, f),
			Type::Array(v) => Display::fmt(v, f),
		}
	}
}

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
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
			PrimitiveType::Boolean => 'Z',
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
impl Debug for PrimitiveType {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			PrimitiveType::Boolean => f.write_str("boolean"),
			PrimitiveType::Byte => f.write_str("byte"),
			PrimitiveType::Short => f.write_str("short"),
			PrimitiveType::Int => f.write_str("int"),
			PrimitiveType::Long => f.write_str("long"),
			PrimitiveType::Char => f.write_str("char"),
			PrimitiveType::Float => f.write_str("float"),
			PrimitiveType::Double => f.write_str("double"),
		}
	}
}

impl Display for PrimitiveType {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.write_char(self.char())
	}
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct ObjectType(pub String);

impl ObjectType {
	#[allow(non_snake_case)]
	pub fn Object() -> ObjectType {
		ObjectType("java/lang/Object".to_string())
	}

	pub fn parse(string: &str) -> Option<ObjectType> {
		Self::parse_len(string).map(|(v, _)| v)
	}
	pub fn parse_len(string: &str) -> Option<(ObjectType, usize)> {
		if string.as_bytes()[0] != b'L' {
			return None;
		}

		let end = string.find(';')?;
		Some((ObjectType(string[1..end].to_string()), end + 1))
	}

	pub fn kind(&self) -> Kind {
		Kind::Reference
	}
}

impl Debug for ObjectType {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.write_str(&self.0)
	}
}

impl From<String> for ObjectType {
	fn from(value: String) -> Self {
		ObjectType(value)
	}
}

impl From<&'static str> for ObjectType {
	fn from(value: &'static str) -> Self {
		ObjectType(value.to_string())
	}
}

impl Display for ObjectType {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.write_char('L')?;
		f.write_str(&self.0)?;
		f.write_char(';')
	}
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct ArrayType {
	pub component: Box<Type>,
}

impl ArrayType {
	pub fn from_component(ty: Type) -> ArrayType {
		ArrayType {
			component: Box::new(ty),
		}
	}

	pub fn parse_len(string: &str) -> Option<(ArrayType, usize)> {
		let bytes = string.as_bytes();
		if bytes[0] != b'[' {
			return None;
		}

		let (component, length) = Type::parse_len(&string[1..])?;
		Some((
			ArrayType {
				component: Box::new(component),
			},
			length + 1,
		))
	}

	pub fn component(&self) -> &Type {
		&self.component
	}

	pub fn kind(&self) -> Kind {
		Kind::Reference
	}
}
impl Debug for ArrayType {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		Debug::fmt(&self.component, f)?;
		f.write_str("[]")
	}
}
impl Display for ArrayType {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.write_char('[')?;
		Display::fmt(&self.component, f)
	}
}

#[derive(Debug, Copy, Clone)]
pub struct CastKindError {
	pub expected: Kind,
	pub found: Kind,
}

impl CastKindError {}
impl Error for CastKindError {}

impl Display for CastKindError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"Cast error! Expected {} but found {}",
			self.expected, self.found
		)
	}
}

#[derive(Debug, Clone)]
pub struct CastTypeError {
	pub expected: Type,
	pub found: Type,
}

impl CastTypeError {}
impl Error for CastTypeError {}

impl Display for CastTypeError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"Cast error! Expected {:?} but found {:?}",
			self.expected, self.found
		)
	}
}
