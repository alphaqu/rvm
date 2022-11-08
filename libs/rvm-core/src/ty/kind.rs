use std::mem::size_of;
use crate::ty::{Value, ValueEnum};

/// A kind represents a type category without any deeper information about the types.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum Kind {
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

impl Kind {
    pub unsafe fn read(&self, ptr: *mut u8) -> ValueEnum {
        match self {
            Kind::Boolean => ValueEnum::Boolean(Value::read(ptr)),
            Kind::Byte => ValueEnum::Byte(Value::read(ptr)),
            Kind::Short => ValueEnum::Short(Value::read(ptr)),
            Kind::Int => ValueEnum::Int(Value::read(ptr)),
            Kind::Long => ValueEnum::Long(Value::read(ptr)),
            Kind::Char => ValueEnum::Char(Value::read(ptr)),
            Kind::Float => ValueEnum::Float(Value::read(ptr)),
            Kind::Double => ValueEnum::Double(Value::read(ptr)),
            Kind::Reference => ValueEnum::Reference(Value::read(ptr)),
        }
    }

    pub unsafe fn write(&self, ptr: *mut u8, value: ValueEnum) {
        match (self, value) {
            (Kind::Boolean, ValueEnum::Boolean(boolean)) => Value::write(ptr, boolean),
            (Kind::Byte, ValueEnum::Byte(value)) => Value::write(ptr, value),
            (Kind::Short, ValueEnum::Short(value)) => Value::write(ptr, value),
            (Kind::Int, ValueEnum::Int(value)) => Value::write(ptr, value),
            (Kind::Long, ValueEnum::Long(value)) => Value::write(ptr, value),
            (Kind::Char, ValueEnum::Char(value)) => Value::write(ptr, value),
            (Kind::Float, ValueEnum::Float(value)) => Value::write(ptr, value),
            (Kind::Double, ValueEnum::Double(value)) => Value::write(ptr, value),
            (Kind::Reference, ValueEnum::Reference(value)) => Value::write(ptr, value),
            _ => {
                panic!("Value type missmatch")
            }
        }
    }
    
    pub fn size(&self) -> usize {
        match self {
            Kind::Boolean => size_of::<bool>(),
            Kind::Byte => size_of::<i8>(),
            Kind::Short => size_of::<i16>(),
            Kind::Int => size_of::<i32>(),
            Kind::Long => size_of::<i64>(),
            Kind::Char => size_of::<u16>(),
            Kind::Float => size_of::<f32>(),
            Kind::Double => size_of::<f64>(),
            Kind::Reference => size_of::<*mut u8>(),
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