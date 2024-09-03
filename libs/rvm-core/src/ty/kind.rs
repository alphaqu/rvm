use crate::{ObjectType, PrimitiveType, Type};
use std::fmt::{Display, Formatter};
use std::mem::size_of;

/// A kind represents a type category without any deeper information about the types.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
#[repr(u8)]
pub enum Kind {
	Reference = 3,
	Boolean = 4,
	Char = 5,
	Float = 6,
	Double = 7,
	Byte = 8,
	Short = 9,
	Int = 10,
	Long = 11,
}

impl Kind {
	pub fn weak_ty(&self) -> Type {
		match self {
			Kind::Reference => Type::Object(ObjectType::Object()),
			Kind::Boolean => Type::Primitive(PrimitiveType::Boolean),
			Kind::Char => Type::Primitive(PrimitiveType::Char),
			Kind::Float => Type::Primitive(PrimitiveType::Float),
			Kind::Double => Type::Primitive(PrimitiveType::Double),
			Kind::Byte => Type::Primitive(PrimitiveType::Byte),
			Kind::Short => Type::Primitive(PrimitiveType::Short),
			Kind::Int => Type::Primitive(PrimitiveType::Int),
			Kind::Long => Type::Primitive(PrimitiveType::Long),
		}
	}

	pub fn size(&self) -> usize {
		match self {
			Kind::Reference => size_of::<*mut u8>(),
			Kind::Boolean => size_of::<bool>(),
			Kind::Char => size_of::<u16>(),
			Kind::Float => size_of::<f32>(),
			Kind::Double => size_of::<f64>(),
			Kind::Byte => size_of::<i8>(),
			Kind::Short => size_of::<i16>(),
			Kind::Int => size_of::<i32>(),
			Kind::Long => size_of::<i64>(),
		}
	}

	pub fn local_size(&self) -> u8 {
		match self {
			Kind::Reference => 1,
			Kind::Boolean => 1,
			Kind::Char => 1,
			Kind::Float => 1,
			Kind::Byte => 1,
			Kind::Short => 1,
			Kind::Int => 1,
			Kind::Double => 2,
			Kind::Long => 2,
		}
	}
	pub fn is_category_2(&self) -> bool {
		matches!(self, Kind::Double | Kind::Long)
	}

	pub fn is_ref(&self) -> bool {
		matches!(self, Kind::Reference)
	}

	pub fn is_floating(&self) -> bool {
		matches!(self, Kind::Float | Kind::Double)
	}

	pub fn is_integer(&self) -> bool {
		matches!(
			self,
			Kind::Int | Kind::Long | Kind::Short | Kind::Byte | Kind::Boolean | Kind::Char
		)
	}
}

impl Display for Kind {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Kind::Boolean => f.write_str("boolean"),
			Kind::Byte => f.write_str("byte"),
			Kind::Short => f.write_str("short"),
			Kind::Int => f.write_str("int"),
			Kind::Long => f.write_str("long"),
			Kind::Char => f.write_str("char"),
			Kind::Float => f.write_str("float"),
			Kind::Double => f.write_str("double"),
			Kind::Reference => f.write_str("Object"),
		}
	}
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum StackKind {
	Int,
	Long,
	Float,
	Double,
	Reference,
}

impl Display for StackKind {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		self.kind().fmt(f)
	}
}

impl StackKind {
	pub fn kind(&self) -> Kind {
		match self {
			StackKind::Int => Kind::Int,
			StackKind::Long => Kind::Long,
			StackKind::Float => Kind::Float,
			StackKind::Double => Kind::Double,
			StackKind::Reference => Kind::Reference,
		}
	}
}
