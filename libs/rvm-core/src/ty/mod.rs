pub use descriptor::*;
pub use flags::*;
pub use kind::*;
use std::borrow::Borrow;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Write};
use std::ops::Deref;
use std::sync::Arc;

mod descriptor;
mod flags;
mod kind;

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

	pub fn to_java(&self) -> String {
		match self {
			Type::Primitive(ty) => ty.char().to_string(),
			Type::Object(ty) => ty.to_java(),
			Type::Array(ty) => ty.to_java(),
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
pub struct ObjectType(Arc<str>);

impl ObjectType {
	#[allow(non_snake_case)]
	pub fn Object() -> ObjectType {
		ObjectType::new("java/lang/Object")
	}

	#[allow(non_snake_case)]
	pub fn String() -> ObjectType {
		ObjectType::new("java/lang/String")
	}

	#[allow(non_snake_case)]
	pub fn Class() -> ObjectType {
		ObjectType::new("java/lang/Class")
	}
	pub fn new<V>(value: V) -> ObjectType
	where
		Arc<str>: From<V>,
	{
		ObjectType(Arc::from(value))
	}

	pub fn parse(string: &str) -> Option<ObjectType> {
		Self::parse_len(string).map(|(v, _)| v)
	}

	pub fn parse_len(string: &str) -> Option<(ObjectType, usize)> {
		if string.as_bytes()[0] != b'L' {
			return None;
		}

		let end = string.find(';')?;
		Some((ObjectType::new(&string[1..end]), end + 1))
	}

	pub fn kind(&self) -> Kind {
		Kind::Reference
	}

	pub fn name(&self) -> String {
		if let Some((_, name)) = self.0.rsplit_once("/") {
			name.to_string()
		} else {
			self.0.to_string()
		}
	}

	// String may be empty
	pub fn package(&self) -> String {
		if let Some((package, _)) = self.0.rsplit_once("/") {
			package.to_string()
		} else {
			// packageless
			String::new()
		}
	}

	pub fn package_path(&self) -> Vec<String> {
		self.package().split("/").map(|v| v.to_string()).collect()
	}

	pub fn to_java(&self) -> String {
		let mut output = String::new();
		output.push('L');
		output.push_str(&self.0);
		output.push(';');
		output
	}
}
impl Deref for ObjectType {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl Debug for ObjectType {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.write_str(&self.0)
	}
}

impl From<String> for ObjectType {
	fn from(value: String) -> Self {
		ObjectType::new(value)
	}
}

impl From<&'static str> for ObjectType {
	fn from(value: &'static str) -> Self {
		ObjectType::new(value.to_string())
	}
}

/// TODO not do this!!!
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
	#[allow(non_snake_case)]
	pub fn ObjectArray() -> ArrayType {
		ArrayType::from_component(Type::Object(ObjectType::Object()))
	}
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

	pub fn to_java(&self) -> String {
		format!("[{}", self.component.to_java())
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
