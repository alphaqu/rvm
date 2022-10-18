use nom::combinator::map_opt;
use nom::multi::length_count;
use nom::number::complete::be_u16;
use tracing::trace;
use rvm_consts::MethodAccessFlags;
use crate::reader::attribute::AttributeInfo;
use crate::reader::consts::ConstantPool;
use crate::reader::{ConstPtr, IResult, UTF8Const};

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
	pub attribute_info: Vec<AttributeInfo>,
}

impl MethodInfo {
	pub fn parse<'a>(input: &'a [u8], constant_pool: &ConstantPool) -> IResult<'a, Self> {
		trace!("access flags");
		let (input, access_flags) = map_opt(be_u16, MethodAccessFlags::from_bits)(input)?;
		trace!("name_index");
		let (input, name_index) = be_u16(input)?;
		trace!("descriptor");
		let (input, descriptor_index) = be_u16(input)?;
		trace!("attribute");
		let (input, attribute_info) =
			length_count(be_u16, |input| AttributeInfo::parse(input, constant_pool))(input)?;

		Ok((
			input,
			Self {
				access_flags,
				name_index: ConstPtr::new(name_index),
				descriptor_index: ConstPtr::new(descriptor_index),
				attribute_info,
			},
		))
	}
}