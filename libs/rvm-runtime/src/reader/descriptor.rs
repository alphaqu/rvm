use crate::descriptor::ReturnDescriptor::Field;
use std::mem::size_of;

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct FieldDescriptor(pub FieldType);

impl FieldDescriptor {
    pub fn parse(desc: &str) -> Option<FieldDescriptor> {
        Some(FieldDescriptor(FieldType::parse(desc)?))
    }
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct ParameterDescriptor(pub FieldType);

impl ParameterDescriptor {
    pub fn parse(desc: &str) -> Option<ParameterDescriptor> {
        Some(ParameterDescriptor(FieldType::parse(desc)?))
    }
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum ReturnDescriptor {
    Field(FieldType),
    Void,
}


impl ReturnDescriptor {
    pub fn is_void(&self) -> bool {
        match self {
            ReturnDescriptor::Void => true,
            _ => false
        }
    }
    
    pub fn parse(desc: &str) -> Option<ReturnDescriptor> {
        return if desc.as_bytes()[0] == b'V' {
            Some(ReturnDescriptor::Void)
        } else {
            Some(Field(FieldType::parse(desc)?))
        };
    }
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct MethodDescriptor {
    pub parameters: Vec<ParameterDescriptor>,
    pub ret: ReturnDescriptor,
}

impl MethodDescriptor {
    pub fn parse(desc: &str) -> Option<MethodDescriptor> {
        let end = desc.find(')')?;
        let mut remaining = &desc[1..end];
        let mut parameters = Vec::new();

        let desc1 = &desc[end + 1..];
        let ret = ReturnDescriptor::parse(desc1)?;
        while !remaining.is_empty() {
            let (parameter, size) = FieldType::parse_len(remaining)?;
            parameters.push(ParameterDescriptor(parameter));
            remaining = &remaining[size..];
        }

        Some(MethodDescriptor { parameters, ret })
    }
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum FieldType {
    Boolean,
    Byte,
    Short,
    Int,
    Long,
    Char,
    Float,
    Double,
    Object(String),
    Array(Box<FieldType>),
}

impl FieldType {
    pub fn parse(desc: &str) -> Option<FieldType> {
        Self::parse_len(desc).map(|(ty, _)| ty)
    }

    pub fn parse_len(desc: &str) -> Option<(FieldType, usize)> {
        Some(match desc.as_bytes()[0] {
            b'Z' => (FieldType::Boolean, 1),
            b'B' => (FieldType::Byte, 1),
            b'C' => (FieldType::Char, 1),
            b'D' => (FieldType::Double, 1),
            b'F' => (FieldType::Float, 1),
            b'I' => (FieldType::Int, 1),
            b'J' => (FieldType::Long, 1),
            b'S' => (FieldType::Short, 1),
            b'L' => {
                let end = desc.find(';')?;
                (FieldType::Object(desc[1..end].to_string()), end + 1)
            }
            b'[' => {
                let (component, len) = FieldType::parse_len(&desc[1..])?;
                (FieldType::Array(Box::new(component)), 1 + len)
            }
            _ => {
                return None;
            }
        })
    }

    pub fn get_size(&self) -> usize {
        match self {
            FieldType::Boolean => size_of::<bool>(),
            FieldType::Byte => size_of::<i8>(),
            FieldType::Short => size_of::<i16>(),
            FieldType::Int => size_of::<i32>(),
            FieldType::Long => size_of::<i64>(),
            FieldType::Char => size_of::<u16>(),
            FieldType::Float => size_of::<f32>(),
            FieldType::Double => size_of::<f64>(),
            FieldType::Array(_) | FieldType::Object(_) => size_of::<*mut u8>(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_method() {
        assert_eq!(
            MethodDescriptor::parse("(IDLjava/lang/Thread;)Ljava/lang/Object;"),
            Some(MethodDescriptor {
                parameters: vec![
                    ParameterDescriptor(FieldType::Int),
                    ParameterDescriptor(FieldType::Double),
                    ParameterDescriptor(FieldType::Object("java/lang/Thread".to_string()))
                ], ret: Field(FieldType::Object("java/lang/Object".to_string()))
            })
        );
    }

    #[test]
    fn parse_primitives() {
        assert_eq!(FieldType::parse("B"), Some(FieldType::Byte));
        assert_eq!(FieldType::parse("C"), Some(FieldType::Char));
        assert_eq!(FieldType::parse("D"), Some(FieldType::Double));
        assert_eq!(FieldType::parse("F"), Some(FieldType::Float));
        assert_eq!(FieldType::parse("I"), Some(FieldType::Int));
        assert_eq!(FieldType::parse("J"), Some(FieldType::Long));
        assert_eq!(FieldType::parse("S"), Some(FieldType::Short));
        assert_eq!(FieldType::parse("Z"), Some(FieldType::Boolean));
    }

    #[test]
    fn parse_object() {
        assert_eq!(
            FieldType::parse("Ljava/lang/Object;"),
            Some(FieldType::Object("java/lang/Object".to_string()))
        );
        assert_eq!(
            FieldType::parse("LHalo;"),
            Some(FieldType::Object("Halo".to_string()))
        );
        assert_eq!(
            FieldType::parse("L;"),
            Some(FieldType::Object("".to_string()))
        );
    }

    #[test]
    fn parse_array() {
        assert_eq!(
            FieldType::parse("[B"),
            Some(FieldType::Array(box FieldType::Byte))
        );
        assert_eq!(
            FieldType::parse("[[B"),
            Some(FieldType::Array(box FieldType::Array(box FieldType::Byte)))
        );
        assert_eq!(
            FieldType::parse("[Ljava/lang/Object;"),
            Some(FieldType::Array(Box::new(FieldType::Object(
                "java/lang/Object".to_string()
            ))))
        );
    }
}
