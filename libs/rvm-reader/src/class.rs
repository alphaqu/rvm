use crate::attribute::AttributeInfo;
use crate::consts::{ConstantInfo, ConstantPool};
use crate::field::FieldInfo;
use crate::method::MethodInfo;
use crate::{ClassConst, ConstPtr, IResult, InterfaceConst};
use nom::bytes::complete::tag;
use nom::combinator::{map, map_opt};
use nom::error::{context, dbg_dmp};
use nom::multi::length_count;
use nom::number::complete::be_u16;
use rvm_core::ClassAccessFlags;
use tracing::trace;

pub struct ClassInfo {
	pub minor_version: u16,
	pub major_version: u16,
	pub constant_pool: ConstantPool,
	pub access_flags: ClassAccessFlags,

	pub this_class: ConstPtr<ClassConst>,
	pub super_class: ConstPtr<ClassConst>,

	pub interfaces: Vec<ConstPtr<InterfaceConst>>,
	pub fields: Vec<FieldInfo>,
	pub methods: Vec<MethodInfo>,
	pub attributes: Vec<AttributeInfo>,
}

impl ClassInfo {
	pub fn parse(input: &[u8]) -> IResult<Self> {
		let (input, _) = context("CAFE", tag(b"\xca\xfe\xba\xbe"))(input)?;
		let (input, minor_version) = context("Java Minor Version", be_u16)(input)?;
		let (input, major_version) = context("Java Major Version", be_u16)(input)?;

		let mut count = 0;
		let mut skip = 0;
		let (input, constant_pool) = context(
			"Constant Pool",
			map(
				length_count(
					map(be_u16, |num| {
						trace!("cp_pool count {}", num - 1);
						num - 1
					}),
					|input| {
						count += 1;
						trace!("{count}");
						ConstantInfo::parse(input, &mut skip)
					},
				),
				ConstantPool::new,
			),
		)(input)?;
		let (input, access_flags) =
			context("Access flags", map_opt(be_u16, ClassAccessFlags::from_bits))(input)?;
		let (input, this_class) = context("This class", be_u16)(input)?;
		let (input, super_class) = context("Class Superclass", be_u16)(input)?;
		let (input, interfaces) = context(
			"Interfaces",
			length_count(be_u16, |v| {
				let (out, value) = be_u16(v)?;
				Ok((out, ConstPtr::new(value)))
			}),
		)(input)?;

		let (input, fields) = context(
			"Fields",
			length_count(be_u16, |input| FieldInfo::parse(input, &constant_pool)),
		)(input)?;
		let (input, methods) = context(
			"Methods",
			length_count(be_u16, |input| MethodInfo::parse(input, &constant_pool)),
		)(input)?;
		let (input, attributes) = context(
			"Attributes",
			length_count(be_u16, |input| AttributeInfo::parse(input, &constant_pool)),
		)(input)?;

		Ok((
			input,
			ClassInfo {
				minor_version,
				major_version,
				constant_pool,
				access_flags,
				this_class: ConstPtr::new(this_class),
				super_class: ConstPtr::new(super_class),
				interfaces,
				fields,
				methods,
				attributes,
			},
		))
	}
}
