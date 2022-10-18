mod class;
mod field;
mod interface;
mod method;
mod name_and_type;
mod number;
mod string;
mod utf_8;

pub use crate::reader::consts::class::ClassConst;
pub use crate::reader::consts::field::FieldConst;
pub use crate::reader::consts::interface::InterfaceConst;
pub use crate::reader::consts::method::{MethodConst, MethodHandleConst, MethodTypeConst};
pub use crate::reader::consts::name_and_type::NameAndTypeConst;
pub use crate::reader::consts::number::{DoubleConst, FloatConst, IntegerConst, LongConst};
pub use crate::reader::consts::string::StringConst;
pub use crate::reader::consts::utf_8::UTF8Const;
use crate::reader::IResult;
use nom::combinator::{map, map_res};
use nom::multi::length_data;
use nom::number::complete::{be_f32, be_f64, be_i32, be_i64, be_u16, be_u32, be_u64, be_u8, le_u16};
use nom::sequence::pair;
use std::cell::Cell;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::ops::RangeFrom;
use nom::error::ParseError;
use nom::{InputIter, InputLength, Slice};
use tracing::trace;

#[macro_export]
macro_rules! impl_constant {
    ($VARIANT:ident $TY:ty) => {
        impl crate::reader::Constant for $TY {
            fn get(value: &crate::reader::ConstantInfo) -> &Self {
                if let crate::reader::ConstantInfo::$VARIANT(v) = value {
                    return v;
                }
                panic!("Wrong type")
            }

            fn get_mut(value: &mut crate::reader::ConstantInfo) -> &mut Self {
                if let crate::reader::ConstantInfo::$VARIANT(v) = value {
                    return v;
                }
                panic!("Wrong type")
            }
        }
    };
}
pub trait Constant {
    fn get(value: &ConstantInfo) -> &Self;
    fn get_mut(value: &mut ConstantInfo) -> &mut Self;
}

pub struct ConstPtr<V: Constant>(u16, PhantomData<V>);

impl<V: Constant> ConstPtr<V> {
    pub fn new(id: u16) -> ConstPtr<V> {
        ConstPtr(id, PhantomData::default())
    }

    pub fn get<'a>(&self, cp: &'a ConstantPool) -> &'a V {
        cp.get(*self)
    }
}
#[inline]
pub fn be_cp<V: Constant>(input: &[u8]) -> IResult<'_, ConstPtr<V>>
{
    map(be_u16, |v| {
        ConstPtr::new(v)
    })(input)
}

impl<V: Constant> Clone for ConstPtr<V> {
    fn clone(&self) -> Self {
        ConstPtr(self.0, PhantomData::default())
    }
}

impl<V: Constant> Debug for ConstPtr<V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<V: Constant> Copy for ConstPtr<V> {}

pub struct ConstantPool(Vec<ConstantInfo>);

impl ConstantPool {
    pub fn new(values: Vec<ConstantInfo>) -> ConstantPool {
        ConstantPool(values)
    }

    pub fn get_raw(&self, index: u16) -> Option<&ConstantInfo> {
        assert!(index >= 1);

        self.0.get(index as usize - 1)
    }

    pub fn get<V: Constant>(&self, ptr: ConstPtr<V>) -> &V {
        assert!(ptr.0 >= 1);
        let info = &self.0[ptr.0 as usize - 1];
        V::get(info)
    }
}

pub enum ConstantInfo {
    Class(ClassConst),
    Field(FieldConst),
    Method(MethodConst),
    Interface(InterfaceConst),
    String(StringConst),
    Integer(IntegerConst),
    Float(FloatConst),
    Long(LongConst),
    Double(DoubleConst),
    NameAndType(NameAndTypeConst),
    UTF8(UTF8Const),
    MethodHandle(MethodHandleConst),
    MethodType(MethodTypeConst),
    Unknown,
}

impl ConstantInfo {
    pub fn parse_method_descriptor(text: &str) {
        let chars = text.chars();
        for c in chars {
            if c == ')' {
                break;
            }
        }
    }

    pub fn parse(input: &[u8]) -> IResult<Self> {
        trace!("constant info");
        let (input, variant) = be_u8(input)?;
        match variant {
            7 => map(be_u16, |name_index| {
                ConstantInfo::Class(ClassConst {
                    name: ConstPtr::new(name_index),
                    link: Cell::new(None),
                })
            })(input),
            9 => map(
                pair(be_u16, be_u16),
                |(class_index, name_and_type_index)| {
                    ConstantInfo::Field(FieldConst {
                        class: ConstPtr::new(class_index),
                        name_and_type: ConstPtr::new(name_and_type_index),
                        link: Cell::new(None),
                    })
                },
            )(input),
            10 => map(
                pair(be_u16, be_u16),
                |(class_index, name_and_type_index)| {
                    ConstantInfo::Method(MethodConst {
                        class: ConstPtr::new(class_index),
                        name_and_type: ConstPtr::new(name_and_type_index),
                        link: Cell::new(None),
                    })
                },
            )(input),
            11 => map(
                pair(be_u16, be_u16),
                |(class_index, name_and_type_index)| {
                    ConstantInfo::Interface(InterfaceConst {
                        class: ConstPtr::new(class_index),
                        name_and_type: ConstPtr::new(name_and_type_index),
                    })
                },
            )(input),
            8 => map(be_u16, |string_index| {
                ConstantInfo::String(StringConst {
                    string: ConstPtr::new(string_index),
                })
            })(input),
            3 => map(be_i32, |bytes| {
                ConstantInfo::Integer(IntegerConst { bytes })
            })(input),
            4 => map(be_f32, |bytes| ConstantInfo::Float(FloatConst { bytes }))(input),
            5 => map(be_i64, |bytes| ConstantInfo::Long(LongConst { bytes }))(input),
            6 => map(be_f64, |bytes| ConstantInfo::Double(DoubleConst { bytes }))(input),
            12 => map(pair(be_u16, be_u16), |(name_index, descriptor_index)| {
                ConstantInfo::NameAndType(NameAndTypeConst {
                    name: ConstPtr::new(name_index),
                    descriptor: ConstPtr::new(descriptor_index),
                })
            })(input),
            1 => map_res(
                length_data(be_u16),
                //FIXME(leocth): Java uses MUTF-8, which Rust does *not* expect. https://en.wikipedia.org/wiki/UTF-8#Modified_UTF-8
                |data: &[u8]| {
                    String::from_utf8(data.into()).map(|text| ConstantInfo::UTF8(UTF8Const(text)))
                },
            )(input),
            15 => map(pair(be_u8, be_u16), |(reference_kind, reference_index)| {
                ConstantInfo::MethodHandle(MethodHandleConst {
                    reference_kind,
                    reference_index,
                })
            })(input),
            16 => map(be_u16, |descriptor_index| {
                ConstantInfo::MethodType(MethodTypeConst {
                    descriptor: ConstPtr::new(descriptor_index),
                })
            })(input),
            //18 => map(
            //	pair(be_u16, be_u16),
            //	|(bootstrap_method_attr_index, name_and_type_index)| ConstantInfo::InvokeDynamic {
            //		bootstrap_method_attr_index,
            //		name_and_type_index,
            //	},
            //)(input),
            _ => {
                println!("unknown");
                return Ok((input, ConstantInfo::Unknown));
            }
        }
    }
}
