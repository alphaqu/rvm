use crate::object::{Ref, Value};
use crate::JError;
use std::any::type_name;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct Stack {
	data: Vec<StackValue>,
}

impl Stack {
	pub fn new(size: u16) -> Stack {
		Stack {
			data: Vec::with_capacity(size as usize),
		}
	}

	#[inline(always)]
	pub fn push<V: StackCast>(&mut self, value: V) {
		self.push_raw(value.push());
	}

	#[inline(always)]
	pub fn pop<V: StackCast>(&mut self) -> Result<V, JError> {
		V::pop(self.pop_raw()?)
	}

	#[inline(always)]
	pub fn push_raw(&mut self, value: StackValue) {
		self.data.push(value);
	}

	#[inline(always)]
	pub fn pop_raw(&mut self) -> Result<StackValue, JError> {
		self.data
			.pop()
			.ok_or_else(|| JError::new("Stack is empty on pop"))
	}

	pub fn iter(&self) -> &[StackValue] {
		&self.data
	}
}

impl Display for Stack {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		for value in &self.data {
			write!(f, " {value}")?;
		}
		Ok(())
	}
}
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum StackValueType {
	Int,
	Long,
	Float,
	Double,
	Object,
}

#[derive(Clone, Debug)]
pub enum StackValue {
	Int(i32),
	Float(f32),
	Long(i64),
	Double(f64),
	Reference(Ref),
}

impl StackValue {
	pub fn is_category_2(&self) -> bool {
		matches!(self, StackValue::Long(_) | StackValue::Double(_))
	}

	pub fn from_value(value: Value) -> Self {
		match value {
			Value::Boolean(v) => StackValue::Int(v as u8 as i32),
			Value::Byte(v) => StackValue::Int(v as i32),
			Value::Short(v) => StackValue::Int(v as i32),
			Value::Int(v) => StackValue::Int(v),
			Value::Long(v) => StackValue::Long(v),
			Value::Char(v) => StackValue::Int(v as i32),
			Value::Float(v) => StackValue::Float(v),
			Value::Double(v) => StackValue::Double(v),
			Value::Reference(v) => StackValue::Reference(v),
		}
	}
}

impl Display for StackValue {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			StackValue::Int(v) => write!(f, "{v}"),
			StackValue::Float(v) => write!(f, "{v:?}"),
			StackValue::Long(v) => write!(f, "{v}"),
			StackValue::Double(v) => write!(f, "{v:?}"),
			StackValue::Reference(v) => write!(f, "{}", v),
		}
	}
}

pub trait StackCast: Sized {
	fn pop(value: StackValue) -> Result<Self, JError>;
	fn push(self) -> StackValue;
}

macro_rules! cast_cast {
	($TY:ty => $CAST:ty) => {
		impl StackCast for $TY {
			fn pop(value: StackValue) -> Result<Self, JError> {
				Ok(<$CAST>::pop(value)? as $TY)
			}

			fn push(self) -> StackValue {
				<$CAST>::push(self as $CAST)
			}
		}
	};
}

cast_cast!(i8 => i32);
cast_cast!(i16 => i32);
cast_cast!(u16 => i32);

impl StackCast for bool {
	fn pop(value: StackValue) -> Result<Self, JError> {
		Ok(i32::pop(value)? != 0)
	}

	fn push(self) -> StackValue {
		i32::push(self as u8 as i32)
	}
}

macro_rules! into_cast {
	($VAR:ident $TY:ty) => {
		impl StackCast for $TY {
			#[inline(always)]
			fn pop(value: StackValue) -> Result<Self, JError> {
				match value {
					StackValue::$VAR(v) => Ok(v),
					_ => Err(JError::new(format!(
						"Expected {} but found {value:?}",
						type_name::<Self>()
					))),
				}
			}

			#[inline(always)]
			fn push(self) -> StackValue {
				StackValue::$VAR(self)
			}
		}
	};
}
into_cast!(Int i32);
into_cast!(Float f32);
into_cast!(Long i64);
into_cast!(Double f64);
into_cast!(Reference Ref);

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn casting() {
		assert_eq!(69.0, f64::pop(f64::push(69.0)).unwrap());
		assert_eq!(69.0f32, f32::pop(f32::push(69.0)).unwrap());
	}
}
