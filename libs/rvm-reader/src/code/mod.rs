use nom::multi::length_count;
use nom::number::streaming::{be_u16, be_u32};

use crate::attribute::{AttributeException, AttributeInfo};
pub use crate::code::inst::*;
use crate::consts::ConstantPool;
use crate::IResult;

mod inst;

pub struct Code {
	pub max_stack: u16,
	pub max_locals: u16,
	pub instructions: Vec<Inst>,
	pub exception_table: Vec<AttributeException>,
	pub attribute_info: Vec<AttributeInfo>,
}

impl Code {
	pub fn parse<'a>(input: &'a [u8], constant_pool: &ConstantPool) -> IResult<'a, Self> {
		let (input, max_stack) = be_u16(input)?;
		let (input, max_locals) = be_u16(input)?;
		let (input, code_length) = be_u32(input)?;
		let mut input = input;

		let mut op_byte_to_op: Vec<u32> = Vec::with_capacity(code_length as usize);
		let mut op_byte_ops: Vec<(u32, Inst)> = Vec::new();

		let mut op_byte_pos: usize = 0;
		let mut op_pos: u32 = 0;

		// Read code and create opbyte to op vec
		while op_byte_pos < code_length as usize {
			let old = input.len();
			let (input2, op) = Inst::parse(input)?;
			let new = input2.len();

			let op_byte_length = old - new;
			op_byte_ops.push((op_byte_pos as u32, op));
			for _i in 0..op_byte_length {
				op_byte_to_op.push(op_pos);
			}
			input = input2;
			op_byte_pos += op_byte_length;
			op_pos += 1;
		}

		// Apply all jumps, as jumps are relative to byte location not op location,
		// This also adds stuff to the split vec which is all of the spots which it should split the code on.
		let mut code: Vec<Inst> = Vec::with_capacity(op_byte_ops.len());
		op_pos = 0;
		for (op_byte, mut op) in op_byte_ops {
			match &mut op {
				Inst::Jump(JumpInst { offset, kind: _ }) => {
					let i = ((op_byte as i64) + (*offset as i64)) as usize;
					let jump_pos = op_byte_to_op[i] as i64;

					*offset = (jump_pos - op_pos as i64) as i16 as i32;
				}
				_ => {}
			};

			code.insert(op_pos as usize, op);
			op_pos += 1;
		}

		let (input, exception_table) = length_count(be_u16, AttributeException::parse)(input)?;
		let (input, attribute_info) =
			length_count(be_u16, |input| AttributeInfo::parse(input, constant_pool))(input)?;

		Ok((
			input,
			Code {
				max_stack,
				max_locals,
				instructions: code,
				exception_table,
				attribute_info,
			},
		))
	}
}
