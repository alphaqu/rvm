use crate::executor::{ StackCast, StackValue};
use crate::object::ValueType;
use crate::{JError, JResult, Ref};
use std::any::type_name;
use std::array::try_from_fn;
use std::fmt::{Display, Formatter};
use std::mem::transmute;

#[derive(Debug)]
pub struct LocalVariables {
    data: Vec<LocalVar>,
}

impl LocalVariables {
    pub fn new(size: u16) -> LocalVariables {
        // TODO zero initialization
        LocalVariables {
            data: vec![LocalVar::Int(0); size as usize],
        }
    }

    pub fn get_raw(&self, local: u16) -> JResult<LocalVar> {
        self.data
            .get(local as usize)
            .cloned()
            .ok_or_else(|| JError::new("Could not find local"))
    }

    pub fn set_raw(&mut self, local: u16, value: LocalVar) -> JResult<()> {
        let var = self
            .data
            .get_mut(local as usize)
            .ok_or_else(|| JError::new("Could not find local"))?;
        *var = value;
        Ok(())
    }

    pub fn set_stack(&mut self, local: u16, stack: StackValue) -> JResult<()> {
        match stack {
            StackValue::Int(v) => self.set(local, v),
            StackValue::Float(v) => self.set(local, v),
            StackValue::Long(v) => self.set(local, v),
            StackValue::Double(v) => self.set(local, v),
            StackValue::Reference(v) => self.set(local, v),
        }
    }

    pub fn get_stack(&mut self, local: u16, ty: ValueType) -> JResult<StackValue> {
        Ok(match ty {
            ValueType::Boolean
            | ValueType::Byte
            | ValueType::Short
            | ValueType::Int
            | ValueType::Char => <i32 as StackCast>::push(self.get::<1, i32>(local)?),
            ValueType::Long => <i64 as StackCast>::push(self.get::<2, i64>(local)?),
            ValueType::Float => <f32 as StackCast>::push(self.get::<1, f32>(local)?),
            ValueType::Double => <f64 as StackCast>::push(self.get::<2, f64>(local)?),
            ValueType::Reference => <Ref as StackCast>::push(self.get::<1, Ref>(local)?),
        })
    }

    pub fn set<const L: usize, V: LocalCast<L>>(&mut self, local: u16, value: V) -> JResult<()> {
        let output = value.push();
        for i in 0..L {
            self.set_raw(local + i as u16, output[i].clone())?;
        }
        Ok(())
    }

    pub fn get<const L: usize, R: LocalCast<L>>(&self, local: u16) -> JResult<R> {
        let from_fn = try_from_fn::<_, L, _>(|i| self.get_raw(local + i as u16))?;
        R::pop(from_fn)
    }


    pub fn iter(&self) -> &[LocalVar] {
        &self.data
    }
}

impl Display for LocalVariables {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let len = self.data.len();
        for (i, value) in self.data.iter().enumerate() {
            write!(f, "{value}")?;
            if i != len - 1 {
                write!(f, " ")?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum LocalVar {
    Int(i32),
    Float(f32),
    Reference(Ref),
    ReturnAddress(u32),
}

impl Display for LocalVar {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LocalVar::Int(v) => write!(f, "{v}"),
            LocalVar::Float(v) => write!(f, "{v:?}"),
            LocalVar::Reference(v) => write!(f, "{}", v),
            LocalVar::ReturnAddress(v) => {
                write!(f, "{v}_ret")
            }
        }
    }
}

pub trait LocalCast<const L: usize>: Sized {
    fn pop(value: [LocalVar; L]) -> Result<Self, JError>;
    fn push(self) -> [LocalVar; L];
}

impl LocalCast<2> for i64 {
    fn pop(value: [LocalVar; 2]) -> Result<Self, JError> {
        match value {
            [LocalVar::Int(v0), LocalVar::Int(v1)] => {
                Ok(unsafe { transmute::<[i32; 2], i64>([v0, v1]) })
            }
            _ => Err(JError::new(format!(
                "Expected {} but found {value:?}",
                type_name::<Self>()
            ))),
        }
    }

    fn push(self) -> [LocalVar; 2] {
        let [low, high] = unsafe { transmute::<i64, [i32; 2]>(self) };
        [LocalVar::Int(low), LocalVar::Int(high)]
    }
}

impl LocalCast<2> for f64 {
    fn pop(value: [LocalVar; 2]) -> Result<Self, JError> {
        match value {
            [LocalVar::Float(v0), LocalVar::Float(v1)] => {
                Ok(unsafe { transmute::<[f32; 2], f64>([v0, v1]) })
            }
            _ => Err(JError::new(format!(
                "Expected {} but found {value:?}",
                type_name::<Self>()
            ))),
        }
    }

    fn push(self) -> [LocalVar; 2] {
        let [low, high] = unsafe { transmute::<f64, [f32; 2]>(self) };
        [LocalVar::Float(low), LocalVar::Float(high)]
    }
}

macro_rules! into_cast {
    ($VAR:ident $TY:ty) => {
        impl LocalCast<1> for $TY {
            fn pop([value]: [LocalVar; 1]) -> Result<Self, JError> {
                if let LocalVar::$VAR(v0) = value {
                    Ok(v0)
                } else {
                    Err(JError::new(format!(
                        "Expected {} but found {value:?}",
                        type_name::<Self>()
                    )))
                }
            }

            fn push(self) -> [LocalVar; 1] {
                [LocalVar::$VAR(self)]
            }
        }
    };
}

into_cast!(Int i32);
into_cast!(Float f32);
into_cast!(ReturnAddress u32);
into_cast!(Reference Ref);
