use crate::executor::Inst;
use crate::reader::attribute::{AttributeException, AttributeInfo};
use crate::reader::consts::ConstantPool;
use crate::reader::IResult;
use nom::multi::length_count;
use nom::number::streaming::{be_u16, be_u32};
use tracing::trace;
pub struct Code {
	pub max_stack: u16,
	pub max_locals: u16,
	pub instructions: Vec<Inst>,
	pub exception_table: Vec<AttributeException>,
	pub attribute_info: Vec<AttributeInfo>,
}

impl Code {
	pub fn parse<'a>(input: &'a [u8], constant_pool: &ConstantPool) -> IResult<'a, Self> {
		trace!("Parsing code");
		let (input, max_stack) = be_u16(input)?;
		trace!("Parsing code locals");
		let (input, max_locals) = be_u16(input)?;
		trace!("Parsing code length");
		let (input, code_length) = be_u32(input)?;
		trace!("Parsing code input");
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
				Inst::IF_ACMPEQ(offset)
				| Inst::IF_ACMPNE(offset)
				| Inst::IF_ICMPEQ(offset)
				| Inst::IF_ICMPNE(offset)
				| Inst::IF_ICMPLT(offset)
				| Inst::IF_ICMPGE(offset)
				| Inst::IF_ICMPGT(offset)
				| Inst::IF_ICMPLE(offset)
				| Inst::IFEQ(offset)
				| Inst::IFNE(offset)
				| Inst::IFLT(offset)
				| Inst::IFGE(offset)
				| Inst::IFGT(offset)
				| Inst::IFLE(offset)
				| Inst::IFNONNULL(offset)
				| Inst::IFNULL(offset)
				| Inst::GOTO(offset) => {
					let i = ((op_byte as i64) + (offset.0 as i64)) as usize;
					let jump_pos = op_byte_to_op[i] as i64;

					offset.0 = (jump_pos - op_pos as i64) as i16;
				}
				_ => {}
			};
			// match op {
			// 				Instruction::DualComparisonJump { jump } => {
			// 					jump.union.apply(op_byte, &op_byte_to_op);
			// 				}
			// 				Instruction::ComparisonJump { jump } => {
			// 					jump.union.apply(op_byte, &op_byte_to_op);
			// 				}
			// 				Instruction::Jump { jump } => {
			// 					jump.union.apply(op_byte, &op_byte_to_op);
			// 				}
			// 				_ => {}
			// 			};

			code.insert(op_pos as usize, op);
			op_pos += 1;
		}

		trace!("Exceptions");
		let (input, exception_table) = length_count(be_u16, AttributeException::parse)(input)?;

		trace!("Attributes");
		let (input, attribute_info) =
			length_count(be_u16, |input| AttributeInfo::parse(input, constant_pool))(input)?;
		trace!("Parsed code");

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

//pub struct Op {
// 	pub op: u8,
// 	pub inst: Instruction,
// }
//
// impl Op {
// 	pub fn parse(input: &[u8]) -> IResult<(Self, u8)> {
// 		trace!("bit");
// 		let (input, op) = be_u8(input)?;
// 		trace!("instruction");
// 		let old = input.len();
// 		let (input, (inst, length, op)) = Instruction::parse(input, op)?;
// 		let new = input.len();
// 		assert_eq!((length as usize), old - new);
// 		Ok((input, (Op { op, inst }, length + 1))) // instructionType length and op
// 	}
// }
//
// #[derive(Debug)]
// pub enum Instruction {
// 	// nop
// 	NOP,
// 	ACONST_NULL,
// 	ICONST_M1,
// 	ICONST_0,
// 	ICONST_1,
// 	ICONST_2,
// 	ICONST_3,
// 	ICONST_4,
// 	ICONST_5,
// 	LCONST_0,
// 	LCONST_1,
// 	FCONST_0,
// 	FCONST_1,
// 	FCONST_2,
// 	DCONST_0,
// 	DCONST_1,
// 	ARRAYLENGTH,
// 	POP,
// 	POP2,
// 	DUP,
// 	DUP_X1,
// 	DUP_X2,
// 	DUP2,
// 	DUP2_X1,
// 	DUP2_X2,
// 	SWAP,
// 	IADD,
// 	LADD,
// 	FADD,
// 	DADD,
// 	ISUB,
// 	LSUB,
// 	FSUB,
// 	DSUB,
// 	IMUL,
// 	LMUL,
// 	FMUL,
// 	DMUL,
// 	IDIV,
// 	LDIV,
// 	FDIV,
// 	DDIV,
// 	IREM,
// 	LREM,
// 	FREM,
// 	DREM,
// 	INEG,
// 	LNEG,
// 	FNEG,
// 	DNEG,
// 	ISHL,
// 	LSHL,
// 	ISHR,
// 	LSHR,
// 	IUSHR,
// 	LUSHR,
// 	IAND,
// 	LAND,
// 	IOR,
// 	LOR,
// 	IXOR,
// 	LXOR,
// 	I2L,
// 	I2F,
// 	I2D,
// 	L2I,
// 	L2F,
// 	L2D,
// 	F2I,
// 	F2L,
// 	F2D,
// 	D2I,
// 	D2L,
// 	D2F,
// 	I2B,
// 	I2C,
// 	I2S,
// 	IRETURN,
// 	LRETURN,
// 	FRETURN,
// 	DRETURN,
// 	ARETURN,
// 	RETURN,
// 	// athrow
// 	Throw { pool_pos: u16 },
// 	// ldc
// 	ConstantPool { pool: u8 },
// 	// ldc_w, ldc2_w
// 	ConstantPoolWide { pool: u16 },
// 	// bipush
// 	PushByte { value: i8 },
// 	// sipush
// 	PushShort { value: i16 },
// 	// iinc
// 	Increment { var: u16, amount: u8 },
// 	// iload iload_0, iload_1, iload_2, iload_3,
// 	// lload lload_0, lload_1, lload_2, lload_3,
// 	// fload fload_0, fload_1, fload_2, fload_3,
// 	// dload dload_0, dload_1, dload_2, dload_3,
// 	// aload aload_0, aload_1, aload_2, aload_3,
// 	Load { var: u16 },
// 	// iaload, laload, faload, daload, aaload, baload, caload, saload
// 	ArrayLoad,
// 	// istore, lstore, fstore, dstore, astore,
// 	// istore_0, istore_1, istore_2, istore_3,
// 	// lstore_0, lstore_1, lstore_2, lstore_3,
// 	// fstore_0, fstore_1, fstore_2, fstore_3,
// 	// dstore_0, dstore_1, dstore_2, dstore_3,
// 	// astore_0, astore_1, astore_2, astore_3,
// 	Store { var: u16 },
// 	// iastore, lastore, fastore, dastore, aastore, bastore, castore, sastore
// 	ArrayStore,
// 	// lcmp, fcmpl, fcmpg, dcmpl, dcmpg
// 	Comparison,
// 	// checkcast
// 	Cast { pool_pos: u16 },
// 	// instanceof
// 	Instanceof { pool_pos: u16 },
// 	// if_icmpeq, if_icmpne, if_icmplt, if_icmpge, if_icmpgt, if_icmple, if_acmpeq, if_acmpne
// 	DualComparisonJump { jump: JumpValue },
// 	// ifeq, ifne, iflt, ifge, ifgt, ifle, ifnull, ifnonnull
// 	ComparisonJump { jump: JumpValue },
// 	// tableswitch lookupswitch
// 	SwitchJump { jumps: Vec<JumpValue> },
// 	// goto, jsr, goto_w, jsr_w
// 	Jump { jump: JumpValue },
// 	// new, anewarray
// 	New { class: ConstPtr<ClassConst> },
// 	// newarray
// 	NewPrimitiveArray { array_type: u8 },
// 	// getfield
// 	GetField { field: ConstPtr<FieldConst> },
// 	// getstatic
// 	GetStaticField { field: ConstPtr<FieldConst> },
// 	// putfield
// 	PutField { field: ConstPtr<FieldConst> },
// 	// putstatic
// 	PutStaticField { field: ConstPtr<FieldConst> },
// 	// invokevirtual, invokespecial, invokestatic, invokeinterface, invokedynamic
// 	InvokeVirtual { method: ConstPtr<MethodConst> },
// 	InvokeSpecial { method: ConstPtr<MethodConst> },
// 	InvokeStatic { method: ConstPtr<MethodConst> },
// 	InvokeInterface { method: ConstPtr<MethodConst> },
// 	InvokeDynamic { method: ConstPtr<MethodConst> },
// 	// monitorenter, monitorexit
// 	Monitor,
// }
//
// impl Instruction {
// 	pub fn parse(input: &[u8], op: u8) -> IResult<(Self, u8, u8)> {
// 		use rvm_consts::*;
// 		trace!("Parsing instruction {op}");
// 		match op {
// 			// nop
// 			NOP => Ok((input, (Instruction::NOP, 0, op))),
// 			// Constant
// 			ACONST_NULL => Ok((input, (Instruction::ACONST_NULL, 0, op))),
// 			ICONST_M1 => Ok((input, (Instruction::ICONST_M1, 0, op))),
// 			ICONST_0 => Ok((input, (Instruction::ICONST_0, 0, op))),
// 			ICONST_1 => Ok((input, (Instruction::ICONST_1, 0, op))),
// 			ICONST_2 => Ok((input, (Instruction::ICONST_2, 0, op))),
// 			ICONST_3 => Ok((input, (Instruction::ICONST_3, 0, op))),
// 			ICONST_4 => Ok((input, (Instruction::ICONST_4, 0, op))),
// 			ICONST_5 => Ok((input, (Instruction::ICONST_5, 0, op))),
// 			LCONST_0 => Ok((input, (Instruction::LCONST_0, 0, op))),
// 			LCONST_1 => Ok((input, (Instruction::LCONST_1, 0, op))),
// 			FCONST_0 => Ok((input, (Instruction::FCONST_0, 0, op))),
// 			FCONST_1 => Ok((input, (Instruction::FCONST_1, 0, op))),
// 			FCONST_2 => Ok((input, (Instruction::FCONST_2, 0, op))),
// 			DCONST_0 => Ok((input, (Instruction::DCONST_0, 0, op))),
// 			DCONST_1 => Ok((input, (Instruction::DCONST_1, 0, op))),
// 			ARRAYLENGTH => Ok((input, (Instruction::ARRAYLENGTH, 0, op))),
// 			// Stack
// 			POP => Ok((input, (Instruction::POP, 0, op))),
// 			POP2 => Ok((input, (Instruction::POP2, 0, op))),
// 			DUP => Ok((input, (Instruction::DUP, 0, op))),
// 			DUP_X1 => Ok((input, (Instruction::DUP_X1, 0, op))),
// 			DUP_X2 => Ok((input, (Instruction::DUP_X2, 0, op))),
// 			DUP2 => Ok((input, (Instruction::DUP2, 0, op))),
// 			DUP2_X1 => Ok((input, (Instruction::DUP2_X1, 0, op))),
// 			DUP2_X2 => Ok((input, (Instruction::DUP2_X2, 0, op))),
// 			SWAP => Ok((input, (Instruction::SWAP, 0, op))),
//
// 			// Math
// 			IADD => Ok((input, (Instruction::IADD, 0, op))),
// 			LADD => Ok((input, (Instruction::LADD, 0, op))),
// 			FADD => Ok((input, (Instruction::FADD, 0, op))),
// 			DADD => Ok((input, (Instruction::DADD, 0, op))),
// 			ISUB => Ok((input, (Instruction::ISUB, 0, op))),
// 			LSUB => Ok((input, (Instruction::LSUB, 0, op))),
// 			FSUB => Ok((input, (Instruction::FSUB, 0, op))),
// 			DSUB => Ok((input, (Instruction::DSUB, 0, op))),
// 			IMUL => Ok((input, (Instruction::IMUL, 0, op))),
// 			LMUL => Ok((input, (Instruction::LMUL, 0, op))),
// 			FMUL => Ok((input, (Instruction::FMUL, 0, op))),
// 			DMUL => Ok((input, (Instruction::DMUL, 0, op))),
// 			IDIV => Ok((input, (Instruction::IDIV, 0, op))),
// 			LDIV => Ok((input, (Instruction::LDIV, 0, op))),
// 			FDIV => Ok((input, (Instruction::FDIV, 0, op))),
// 			DDIV => Ok((input, (Instruction::DDIV, 0, op))),
// 			IREM => Ok((input, (Instruction::IREM, 0, op))),
// 			LREM => Ok((input, (Instruction::LREM, 0, op))),
// 			FREM => Ok((input, (Instruction::FREM, 0, op))),
// 			DREM => Ok((input, (Instruction::DREM, 0, op))),
// 			INEG => Ok((input, (Instruction::INEG, 0, op))),
// 			LNEG => Ok((input, (Instruction::LNEG, 0, op))),
// 			FNEG => Ok((input, (Instruction::FNEG, 0, op))),
// 			DNEG => Ok((input, (Instruction::DNEG, 0, op))),
// 			ISHL => Ok((input, (Instruction::ISHL, 0, op))),
// 			LSHL => Ok((input, (Instruction::LSHL, 0, op))),
// 			ISHR => Ok((input, (Instruction::ISHR, 0, op))),
// 			LSHR => Ok((input, (Instruction::LSHR, 0, op))),
// 			IUSHR => Ok((input, (Instruction::IUSHR, 0, op))),
// 			LUSHR => Ok((input, (Instruction::LUSHR, 0, op))),
// 			IAND => Ok((input, (Instruction::IAND, 0, op))),
// 			LAND => Ok((input, (Instruction::LAND, 0, op))),
// 			IOR => Ok((input, (Instruction::IOR, 0, op))),
// 			LOR => Ok((input, (Instruction::LOR, 0, op))),
// 			IXOR => Ok((input, (Instruction::IXOR, 0, op))),
// 			LXOR => Ok((input, (Instruction::LXOR, 0, op))),
//
// 			// Conversions
// 			I2L => Ok((input, (Instruction::I2L, 0, op))),
// 			I2F => Ok((input, (Instruction::I2F, 0, op))),
// 			I2D => Ok((input, (Instruction::I2D, 0, op))),
// 			L2I => Ok((input, (Instruction::L2I, 0, op))),
// 			L2F => Ok((input, (Instruction::L2F, 0, op))),
// 			L2D => Ok((input, (Instruction::L2D, 0, op))),
// 			F2I => Ok((input, (Instruction::F2I, 0, op))),
// 			F2L => Ok((input, (Instruction::F2L, 0, op))),
// 			F2D => Ok((input, (Instruction::F2D, 0, op))),
// 			D2I => Ok((input, (Instruction::D2I, 0, op))),
// 			D2L => Ok((input, (Instruction::D2L, 0, op))),
// 			D2F => Ok((input, (Instruction::D2F, 0, op))),
// 			I2B => Ok((input, (Instruction::I2B, 0, op))),
// 			I2C => Ok((input, (Instruction::I2C, 0, op))),
// 			I2S => Ok((input, (Instruction::I2S, 0, op))),
// 			// Return
// 			IRETURN => Ok((input, (Instruction::IRETURN, 0, op))),
// 			LRETURN => Ok((input, (Instruction::LRETURN, 0, op))),
// 			FRETURN => Ok((input, (Instruction::FRETURN, 0, op))),
// 			DRETURN => Ok((input, (Instruction::DRETURN, 0, op))),
// 			ARETURN => Ok((input, (Instruction::ARETURN, 0, op))),
// 			RETURN => Ok((input, (Instruction::RETURN, 0, op))),
//
// 			ATHROW => map(be_u16, |pool_pos| (Instruction::Throw { pool_pos }, 2, op))(input),
// 			// Constant Pool related
// 			LDC => map(be_u8, |pool| (Instruction::ConstantPool { pool }, 1, op))(input),
// 			LDC_W | LDC2_W => map(be_u16, |pool| {
// 				(Instruction::ConstantPoolWide { pool }, 2, op)
// 			})(input),
// 			// Push
// 			BIPUSH => map(be_i8, |value| (Instruction::PushByte { value }, 1, op))(input),
// 			SIPUSH => map(be_i16, |value| (Instruction::PushShort { value }, 2, op))(input),
// 			// Increment
// 			// TODO wide instruction
// 			IINC => map(pair(be_u8, be_u8), |(var, amount)| {
// 				(
// 					Instruction::Increment {
// 						var: var as u16,
// 						amount,
// 					},
// 					2,
// 					op,
// 				)
// 			})(input),
// 			// Load
// 			// TODO wide instruction
// 			ILOAD | LLOAD | FLOAD | DLOAD | ALOAD => {
// 				map(be_u8, |var| (Instruction::Load { var: var as u16 }, 1, op))(input)
// 			}
// 			IALOAD | LALOAD | FALOAD | DALOAD | AALOAD | BALOAD | CALOAD | SALOAD => {
// 				Ok((input, (Instruction::ArrayLoad, 0, op)))
// 			}
// 			// TODO op instruction should be LOAD not LOAD_<x>
// 			ILOAD_0 | ILOAD_1 | ILOAD_2 | ILOAD_3 | LLOAD_0 | LLOAD_1 | LLOAD_2 | LLOAD_3
// 			| FLOAD_0 | FLOAD_1 | FLOAD_2 | FLOAD_3 | DLOAD_0 | DLOAD_1 | DLOAD_2 | DLOAD_3
// 			| ALOAD_0 | ALOAD_1 | ALOAD_2 | ALOAD_3 => Ok((input, {
// 				let opcode = op - ILOAD_0;
// 				(
// 					Instruction::Load {
// 						var: (opcode & 0x3) as u16,
// 					},
// 					0,
// 					ILOAD + (opcode >> 2),
// 				)
// 			})),
// 			// Store
// 			// TODO wide instruction
// 			ISTORE | LSTORE | FSTORE | DSTORE | ASTORE => {
// 				map(be_u8, |var| (Instruction::Store { var: var as u16 }, 1, op))(input)
// 			}
// 			IASTORE | LASTORE | FASTORE | DASTORE | AASTORE | BASTORE | CASTORE | SASTORE => {
// 				Ok((input, (Instruction::ArrayStore, 0, op)))
// 			}
// 			// TODO op instruction should be STORE not STORE_<x>
// 			ISTORE_0 | ISTORE_1 | ISTORE_2 | ISTORE_3 | LSTORE_0 | LSTORE_1 | LSTORE_2
// 			| LSTORE_3 | FSTORE_0 | FSTORE_1 | FSTORE_2 | FSTORE_3 | DSTORE_0 | DSTORE_1
// 			| DSTORE_2 | DSTORE_3 | ASTORE_0 | ASTORE_1 | ASTORE_2 | ASTORE_3 => Ok((input, {
// 				let opcode = op - ISTORE_0;
// 				(
// 					Instruction::Store {
// 						var: (opcode & 0x3) as u16,
// 					},
// 					0,
// 					ISTORE + (opcode >> 2),
// 				)
// 			})),
// 			// Comparisons
// 			LCMP | FCMPL | FCMPG | DCMPL | DCMPG => Ok((input, (Instruction::Comparison, 0, op))),
// 			CHECKCAST => map(be_u16, |pool_pos| {
// 				(Instruction::Cast { pool_pos }, 2, op)
// 			})(input),
// 			INSTANCEOF => map(be_u16, |pool_pos| {
// 				(Instruction::Instanceof { pool_pos }, 2, op)
// 			})(input),
// 			// Jumps
// 			IF_ICMPEQ | IF_ICMPNE | IF_ICMPLT | IF_ICMPGE | IF_ICMPGT | IF_ICMPLE | IF_ACMPEQ
// 			| IF_ACMPNE => map(be_i16, |jump_offset| {
// 				(
// 					Instruction::DualComparisonJump { jump: JumpValue::new(jump_offset as i32) },
// 					2,
// 					op,
// 				)
// 			})(input),
//
// 			IFEQ | IFNE | IFLT | IFGE | IFGT | IFLE | IFNULL | IFNONNULL => map(be_i16, |jump_offset| {
// 				(
// 					Instruction::ComparisonJump { jump: JumpValue::new(jump_offset as i32) },
// 					2,
// 					op,
// 				)
// 			})(input),
// 			// TODO Switch
//
// 			// Jump
// 			GOTO | JSR => map(be_i16, |jump_offset| {
// 				(
// 					Instruction::Jump { jump: JumpValue::new(jump_offset as i32) },
// 					2,
// 					op,
// 				)
// 			})(input),
// 			GOTO_W | JSR_W => map(be_i32, |jump_offset| {
// 				(
// 					Instruction::Jump { jump: JumpValue::new(jump_offset) },
// 					4,
// 					op,
// 				)
// 			})(input),
// 			// New
// 			NEW | ANEWARRAY => {
// 				map(be_u16, |pool_pos| (Instruction::New { class: ConstPtr::new(pool_pos) }, 2, op))(input)
// 			}
// 			NEWARRAY => map(be_u8, |array_type| {
// 				(Instruction::NewPrimitiveArray { array_type }, 1, op)
// 			})(input),
// 			// Get
// 			GETFIELD => map(be_u16, |pool_pos| (Instruction::GetField { field: ConstPtr::new(pool_pos) }, 2, op))(input),
// 			GETSTATIC => map(be_u16, |pool_pos| (Instruction::GetStaticField { field: ConstPtr::new(pool_pos) }, 2, op))(input),
// 			// Put
// 			PUTFIELD => map(be_u16, |pool_pos| (Instruction::PutField { field: ConstPtr::new(pool_pos) }, 2, op))(input),
// 			PUTSTATIC => map(be_u16, |pool_pos| (Instruction::PutStaticField { field: ConstPtr::new(pool_pos) }, 2, op))(input),
// 			// Invoke
// 			INVOKEVIRTUAL => {
// 				map(be_u16, |pool_pos| (Instruction::InvokeVirtual { method: ConstPtr::new(pool_pos) }, 2, op))(input)
// 			}
// 			INVOKESPECIAL => {
// 				map(be_u16, |pool_pos| (Instruction::InvokeSpecial { method: ConstPtr::new(pool_pos) }, 2, op))(input)
// 			}
// 			INVOKESTATIC => {
// 				map(be_u16, |pool_pos| (Instruction::InvokeStatic { method: ConstPtr::new(pool_pos) }, 2, op))(input)
// 			}
// 			INVOKEINTERFACE => {
// 				map(be_u16, |pool_pos| (Instruction::InvokeInterface { method: ConstPtr::new(pool_pos) }, 2, op))(input)
// 			}
// 			INVOKEDYNAMIC => {
// 				map(be_u16, |pool_pos| (Instruction::InvokeDynamic { method: ConstPtr::new(pool_pos) }, 2, op))(input)
// 			}
// 			MONITORENTER | MONITOREXIT => Ok((input, (Instruction::Monitor, 0, op))),
// 			_ => Err(nom::Err::Error(make_error(input, ErrorKind::Fail))),
// 		}
// 	}
// }
//
// pub struct JumpValue {
// 	union: JumpUnion,
// }
//
// impl JumpValue {
// 	pub fn new(offset: i32) -> JumpValue {
// 		JumpValue {
// 			union: JumpUnion {
// 				jump_offset: offset,
// 			},
// 		}
// 	}
//
// 	pub fn get_pos(&self) -> u32 {
// 		unsafe {
// 			self.union.jump_pos
// 		}
// 	}
// }
//
// impl Debug for JumpValue {
// 	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
// 		f.write_str(&*format!("Jump {}", self.deref().get_pos()))
// 	}
// }
//
// pub union JumpUnion {
// 	jump_pos: u32,
// 	jump_offset: i32,
// }
//
// impl JumpUnion {
// 	pub fn apply(&mut self, op_byte: u32, op_byte_to_op: &[u32]) {
// 		unsafe {
// 			let i = ((op_byte as i64) + (self.jump_offset as i64)) as usize;
// 			self.jump_pos = *op_byte_to_op.get(i).unwrap();
// 		}
// 	}
// }
