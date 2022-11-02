use std::fmt::{Display, Formatter, Write};
use crate::executor::StackValue;
use crate::Ref;
use std::mem::size_of;
use std::ptr::{read, write};
use inkwell::context::Context;
use inkwell::types::{BasicType, BasicTypeEnum};
use inkwell::values::BasicValueEnum;

#[derive(Copy, Clone, PartialOrd, PartialEq, Debug)]
pub enum Value {
	Boolean(bool),
	Byte(i8),
	Short(i16),
	Int(i32),
	Long(i64),
	Char(u16),
	Float(f32),
	Double(f64),
	Reference(Ref),
}

impl Value {
	pub fn ty(&self) -> ValueType {
		match self {
			Value::Boolean(_) => ValueType::Boolean,
			Value::Byte(_) => ValueType::Byte,
			Value::Short(_) => ValueType::Short,
			Value::Int(_) => ValueType::Int,
			Value::Long(_) => ValueType::Long,
			Value::Char(_) => ValueType::Char,
			Value::Float(_) => ValueType::Float,
			Value::Double(_) => ValueType::Double,
			Value::Reference(_) => ValueType::Reference,
		}
	}
}

#[derive(Copy, Clone, PartialOrd, PartialEq, Debug)]
pub enum ValueType {
	Boolean,
	Byte,
	Short,
	Int,
	Long,
	Char,
	Float,
	Double,
	Reference,
}

impl ValueType {
	pub fn is_category_2(&self) -> bool {
		match self {
			ValueType::Long => true,
			ValueType::Double => true,
			_ => false,
		}
	}
	
	pub fn ir<'a>(&self, ctx: &'a Context) -> BasicTypeEnum<'a> {
		match self {
			ValueType::Boolean => ctx.bool_type().as_basic_type_enum(),
			ValueType::Byte => ctx.i8_type().as_basic_type_enum(),
			ValueType::Short => ctx.i16_type().as_basic_type_enum(),
			ValueType::Int => ctx.i32_type().as_basic_type_enum(),
			ValueType::Long => ctx.i64_type().as_basic_type_enum(),
			ValueType::Char => ctx.i16_type().as_basic_type_enum(),
			ValueType::Float => ctx.f32_type().as_basic_type_enum(),
			ValueType::Double => ctx.f64_type().as_basic_type_enum(),
			ValueType::Reference => ctx.i32_type().as_basic_type_enum(),
		}
	}
	
	pub fn new_val(&self, stack: StackValue) -> Value {
		match (self, stack) {
			(ValueType::Boolean, StackValue::Int(v)) => Value::Boolean(v != 0),
			(ValueType::Byte, StackValue::Int(v)) => Value::Byte(v as i8),
			(ValueType::Short, StackValue::Int(v)) => Value::Short(v as i16),
			(ValueType::Int, StackValue::Int(v)) => Value::Int(v),
			(ValueType::Long, StackValue::Long(v)) => Value::Long(v),
			(ValueType::Char, StackValue::Int(v)) => Value::Char(v as u16),
			(ValueType::Float, StackValue::Float(v)) => Value::Float(v),
			(ValueType::Double, StackValue::Double(v)) => Value::Double(v),
			(ValueType::Reference, StackValue::Reference(v)) => {
				// TODO instanceof
				Value::Reference(v)
			}
			_ => {
				// TODO result
				panic!("Invalid conversion")
			}
		}
	}

	pub unsafe fn read(&self, ptr: *mut u8) -> Value {
		match self {
			ValueType::Boolean => Value::Boolean(Type::read(ptr)),
			ValueType::Byte => Value::Byte(Type::read(ptr)),
			ValueType::Short => Value::Short(Type::read(ptr)),
			ValueType::Int => Value::Int(Type::read(ptr)),
			ValueType::Long => Value::Long(Type::read(ptr)),
			ValueType::Char => Value::Char(Type::read(ptr)),
			ValueType::Float => Value::Float(Type::read(ptr)),
			ValueType::Double => Value::Double(Type::read(ptr)),
			ValueType::Reference => Value::Reference(Type::read(ptr)),
		}
	}

	pub unsafe fn write(&self, ptr: *mut u8, value: Value) {
		match (self, value) {
			(ValueType::Boolean, Value::Boolean(boolean)) => Type::write(ptr, boolean),
			(ValueType::Byte, Value::Byte(value)) => Type::write(ptr, value),
			(ValueType::Short, Value::Short(value)) => Type::write(ptr, value),
			(ValueType::Int, Value::Int(value)) => Type::write(ptr, value),
			(ValueType::Long, Value::Long(value)) => Type::write(ptr, value),
			(ValueType::Char, Value::Char(value)) => Type::write(ptr, value),
			(ValueType::Float, Value::Float(value)) => Type::write(ptr, value),
			(ValueType::Double, Value::Double(value)) => Type::write(ptr, value),
			(ValueType::Reference, Value::Reference(value)) => Type::write(ptr, value),
			_ => {
				panic!("Value type missmatch")
			}
		}
	}

	pub fn size(&self) -> usize {
		match self {
			ValueType::Boolean => size_of::<bool>(),
			ValueType::Byte => size_of::<i8>(),
			ValueType::Short => size_of::<i16>(),
			ValueType::Int => size_of::<i32>(),
			ValueType::Long => size_of::<i64>(),
			ValueType::Char => size_of::<u16>(),
			ValueType::Float => size_of::<f32>(),
			ValueType::Double => size_of::<f64>(),
			ValueType::Reference => size_of::<Ref>(),
		}
	}
}

impl Display for ValueType {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			ValueType::Boolean => f.write_str("boolean"),
			ValueType::Byte => f.write_str("byte"),
			ValueType::Short => f.write_str("short"),
			ValueType::Int => f.write_str("int"),
			ValueType::Long => f.write_str("long"),
			ValueType::Char => f.write_str("char"),
			ValueType::Float => f.write_str("float"),
			ValueType::Double => f.write_str("double"),
			ValueType::Reference => f.write_str("ref"),
		}
	}
}
pub trait Type: Sized {
	fn ty() -> ValueType;
	unsafe fn write(ptr: *mut u8, value: Self);
	unsafe fn read(ptr: *mut u8) -> Self;
}

macro_rules! impl_direct {
	($VAR:ident $TY:ty) => {
		impl Type for $TY {
			fn ty() -> ValueType {
				ValueType::$VAR
			}

			unsafe fn write(ptr: *mut u8, value: Self) {
				write_arr(ptr, value.to_le_bytes())
			}

			unsafe fn read(ptr: *mut u8) -> Self {
				<$TY>::from_le_bytes(read_arr(ptr))
			}
		}
	};
}
impl_direct!(Byte i8);
impl_direct!(Short i16);
impl_direct!(Int i32);
impl_direct!(Long i64);
impl_direct!(Char u16);
impl_direct!(Float f32);
impl_direct!(Double f64);

impl Type for bool {
	fn ty() -> ValueType {
		ValueType::Boolean
	}

	unsafe fn write(ptr: *mut u8, value: Self) {
		write(ptr, value as u8)
	}

	unsafe fn read(ptr: *mut u8) -> Self {
		read(ptr) != 0
	}
}

impl Type for Ref {
	fn ty() -> ValueType {
		ValueType::Reference
	}

	unsafe fn write(ptr: *mut u8, value: Self) {
		write_arr(ptr, {
			let i = value.0 .0 as usize;
			i.to_le_bytes()
		})
	}

	unsafe fn read(ptr: *mut u8) -> Self {
		Ref::new_ptr(usize::from_le_bytes(read_arr(ptr)) as *mut u8)
	}
}

#[inline(always)]
unsafe fn read_arr<const C: usize>(ptr: *mut u8) -> [u8; C] {
	let mut out = [0; C];
	for i in 0..C {
		*out.get_unchecked_mut(i) = read(ptr.add(i));
	}
	out
}

#[inline(always)]
unsafe fn write_arr<const C: usize>(ptr: *mut u8, value: [u8; C]) {
	for i in 0..C {
		write(ptr.add(i), *value.get_unchecked(i));
	}
}
