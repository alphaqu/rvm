use nom::bytes::complete::take;
use nom::combinator::{map, map_opt};
use nom::error::context;
use nom::multi::length_count;
use nom::number::complete::{be_u16, be_u32};
use nom::sequence::tuple;

use crate::code::Code;
use crate::consts::{ConstantInfo, ConstantPool};
use crate::IResult;

pub struct AttributeException {
	start_pc: u16,
	end_pc: u16,
	handler_pc: u16,
	catch_type: u16,
}

impl AttributeException {
	pub fn parse(input: &[u8]) -> IResult<Self> {
		map(
			tuple((be_u16, be_u16, be_u16, be_u16)),
			|(start_pc, end_pc, handler_pc, catch_type)| AttributeException {
				start_pc,
				end_pc,
				handler_pc,
				catch_type,
			},
		)(input)
	}
}

pub struct AttributeClass {
	inner_class_info_index: u16,
	outer_class_info_index: u16,
	inner_name_index: u16,
	inner_class_access_flags: u16,
}

pub struct AttributeLineNumber {
	start_pc: u16,
	line_number: u16,
}

pub struct AttributeLocalVariable {
	start_pc: u16,
	length: u16,
	name_index: u16,
	descriptor_index: u16,
	index: u16,
}

pub struct AttributeLocalVariableType {
	start_pc: u16,
	length: u16,
	name_index: u16,
	signature_index: u16,
	index: u16,
}

pub struct AttributeBootstrapMethod {
	bootstrap_method_ref: u16,
	bootstrap_arguments: Vec<u16>,
}

pub enum AttributeInfo {
	ConstantValue {
		constant_index: u16,
	},
	CodeAttribute {
		code: Code,
	},
	// TODO do this
	StackMapTable,
	Exceptions {
		exception_index_table: Vec<u16>,
	},
	InnerClasses {
		classes: Vec<AttributeClass>,
	},
	EnclosingMethod {
		class_index: u16,
		method_index: u16,
	},
	Synthetic,
	Signature {
		signature_index: u16,
	},
	SourceFile {
		source_file_index: u16,
	},
	SourceDebugExtension {
		debug_extension: Vec<u8>,
	},
	LineNumberTable {
		line_number_table: Vec<AttributeLineNumber>,
	},
	LocalVariableTable {
		local_variable_table: Vec<AttributeLocalVariable>,
	},
	LocalVariableTypeTable {
		local_variable_type_table: Vec<AttributeLocalVariableType>,
	},
	Deprecated,
	// TODO do this
	RuntimeInvisibleAnnotations,
	// TODO do this
	RuntimeVisibleParameterAnnotations,
	// TODO do this
	RuntimeInvisibleParameterAnnotations,
	// TODO do this
	AnnotationDefault,
	// TODO do this
	RuntimeVisibleAnnotations,
	BootstrapMethods {
		bootstrap_methods: Vec<AttributeBootstrapMethod>,
	},
}

impl AttributeInfo {
	pub fn parse_list<'a>(input: &'a [u8], constant_pool: &ConstantPool) -> IResult<'a, Vec<Self>> {
		context(
			"Attributes",
			length_count(be_u16, |input| AttributeInfo::parse(input, constant_pool)),
		)(input)
	}
	pub fn parse<'a>(input: &'a [u8], constant_pool: &ConstantPool) -> IResult<'a, Self> {
		let (input, info) = map_opt(be_u16, |index| constant_pool.raw_get(index))(input)?;
		let (input, length) = be_u32(input)?;

		match info {
			ConstantInfo::UTF8(text) => match text.as_str() {
				"ConstantValue" => map(be_u16, |constant_index| AttributeInfo::ConstantValue {
					constant_index,
				})(input),
				"Code" => context(
					"Code",
					map(
						|input| Code::parse(input, constant_pool),
						|code| AttributeInfo::CodeAttribute { code },
					),
				)(input),
				_ => map(take(length), |_| AttributeInfo::AnnotationDefault)(input),
			},
			//discard the remaining bytes
			_ => map(take(length), |_| AttributeInfo::AnnotationDefault)(input),
		}
	}
}
