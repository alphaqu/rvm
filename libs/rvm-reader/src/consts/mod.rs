mod class;
mod field;
mod interface;
mod method;
mod name_and_type;
mod number;
mod string;
mod utf_8;

pub use crate::consts::class::ClassConst;
pub use crate::consts::field::FieldConst;
pub use crate::consts::interface::InterfaceConst;
pub use crate::consts::method::{MethodConst, MethodHandleConst, MethodTypeConst};
pub use crate::consts::name_and_type::NameAndTypeConst;
pub use crate::consts::number::{DoubleConst, FloatConst, IntegerConst, LongConst};
pub use crate::consts::string::StringConst;
pub use crate::consts::utf_8::UTF8Const;
use crate::IResult;
use nom::combinator::{map, map_res};
use nom::error::VerboseError;
use nom::multi::length_data;
use nom::number::complete::{be_f32, be_f64, be_i32, be_i64, be_u16, be_u8};
use nom::sequence::pair;
use nom::Needed;
use std::cell::Cell;
use std::fmt::{Debug, Formatter};
use std::io;
use std::io::ErrorKind;
use std::marker::PhantomData;
use tracing::{error, trace};

#[macro_export]
macro_rules! impl_constant {
	($VARIANT:ident $TY:ty) => {
		impl crate::Constant for $TY {
			fn get(value: &crate::ConstantInfo) -> &Self {
				if let crate::ConstantInfo::$VARIANT(v) = value {
					return v;
				}
				panic!("Wrong type")
			}

			fn get_mut(value: &mut crate::ConstantInfo) -> &mut Self {
				if let crate::ConstantInfo::$VARIANT(v) = value {
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

	pub fn get<'a>(&self, cp: &'a ConstantPool) -> Option<&'a V> {
		cp.get(*self)
	}
}
#[inline]
pub fn be_cp<V: Constant>(input: &[u8]) -> IResult<'_, ConstPtr<V>> {
	map(be_u16, |v| ConstPtr::new(v))(input)
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

	pub fn raw_get(&self, index: u16) -> Option<&ConstantInfo> {
		assert!(index >= 1);
		self.0.get(index as usize - 1)
	}

	pub fn get<V: Constant>(&self, ptr: ConstPtr<V>) -> Option<&V> {
		if ptr.0 >= 1 {
			let info = &self.0[ptr.0 as usize - 1];
			Some(V::get(info))
		} else {
			None
		}
	}
}

#[derive(Debug)]
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
	Unusable,
	Unknown,
}

impl ConstantInfo {
	pub fn parse<'a>(input: &'a [u8], skip: &mut usize) -> IResult<'a, Self> {
		if *skip > 0 {
			*skip -= 1;
			return Ok((input, ConstantInfo::Unknown));
		}
		let (input, variant) = be_u8(input)?;

		trace!("cp_info tag: {variant}");
		match variant {
			1 => map_res(
				length_data(be_u16),
				//FIXME(leocth): Java uses MUTF-8, which Rust does *not* expect. https://en.wikipedia.org/wiki/UTF-8#Modified_UTF-8
				|data: &[u8]| {
					let data = mutf8::mutf8_to_utf8(data).unwrap();
					let vec = data.to_vec();
					String::from_utf8(vec).map(|v| {
						trace!("cp_info UTF-8 {v}");
						ConstantInfo::UTF8(UTF8Const(v))
					})
				},
			)(input),
			3 => map(be_i32, |bytes| {
				ConstantInfo::Integer(IntegerConst { bytes })
			})(input),
			4 => map(be_f32, |bytes| ConstantInfo::Float(FloatConst { bytes }))(input),
			5 => {
				*skip += 1;
				map(be_i64, |bytes| ConstantInfo::Long(LongConst { bytes }))(input)
			}
			6 => {
				*skip += 1;
				map(be_f64, |bytes| ConstantInfo::Double(DoubleConst { bytes }))(input)
			}
			7 => map(be_u16, |name_index| {
				ConstantInfo::Class(ClassConst {
					name: ConstPtr::new(name_index),
				})
			})(input),
			8 => map(be_u16, |string_index| {
				ConstantInfo::String(StringConst {
					string: ConstPtr::new(string_index),
				})
			})(input),
			9 => map(
				pair(be_u16, be_u16),
				|(class_index, name_and_type_index)| {
					ConstantInfo::Field(FieldConst {
						class: ConstPtr::new(class_index),
						name_and_type: ConstPtr::new(name_and_type_index),
					})
				},
			)(input),
			10 => map(
				pair(be_u16, be_u16),
				|(class_index, name_and_type_index)| {
					ConstantInfo::Method(MethodConst {
						class: ConstPtr::new(class_index),
						name_and_type: ConstPtr::new(name_and_type_index),
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
			12 => map(pair(be_u16, be_u16), |(name_index, descriptor_index)| {
				ConstantInfo::NameAndType(NameAndTypeConst {
					name: ConstPtr::new(name_index),
					descriptor: ConstPtr::new(descriptor_index),
				})
			})(input),
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
			17 => panic!("Dynamic is not supported"),
			18 => panic!("InvokeDynamic is not supported"),
			19 => panic!("Module is not supported"),
			20 => panic!("Package is not supported"),
			opcode => {
				return Err(nom::Err::Incomplete(Needed::Unknown));
				panic!("Unknown {opcode}");
				return Ok((input, ConstantInfo::Unknown));
			}
		}
	}
}
