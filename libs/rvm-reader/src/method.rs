use nom::combinator::map_opt;
use nom::number::complete::be_u16;

use rvm_core::MethodAccessFlags;

use crate::attribute::AttributeInfo;
use crate::consts::ConstantPool;
use crate::{ConstPtr, IResult, UTF8Const};

//method_info {
//     u16             access_flags;
//     u16             name_index;
//     u16             descriptor_index;
//     u16             attributes_count;
//     attribute_info attributes[attributes_count];
// }
pub struct MethodInfo {
	pub access_flags: MethodAccessFlags,
	pub name_index: ConstPtr<UTF8Const>,
	pub descriptor_index: ConstPtr<UTF8Const>,
	pub attributes: Vec<AttributeInfo>,
}

impl MethodInfo {
	pub fn parse<'a>(input: &'a [u8], constant_pool: &ConstantPool) -> IResult<'a, Self> {
		let (input, access_flags) = map_opt(be_u16, MethodAccessFlags::from_bits)(input)?;
		let (input, name_index) = be_u16(input)?;
		let (input, descriptor_index) = be_u16(input)?;
		let (input, attribute_info) = AttributeInfo::parse_list(input, constant_pool)?;

		Ok((
			input,
			Self {
				access_flags,
				name_index: ConstPtr::new(name_index),
				descriptor_index: ConstPtr::new(descriptor_index),
				attributes: attribute_info,
			},
		))
	}
}
