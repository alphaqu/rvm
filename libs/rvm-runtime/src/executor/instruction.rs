use crate::reader::{be_cp, BaseDesc, IResult};
use crate::{ClassConst, ConstPtr, FieldConst, MethodConst};
use nom::combinator::map;
use nom::number::complete::{be_i16, be_i32, be_i8, be_u16, be_u8};
use nom::sequence::tuple;
use rvm_consts::print_op;
use tracing::trace;

#[allow(non_camel_case_types)]
#[derive(Clone, Debug)]
pub enum Inst {
	NOP,
	// Constants
	ACONST_NULL,
	DCONST_0,
	DCONST_1,
	FCONST_0,
	FCONST_1,
	FCONST_2,
	ICONST_M1,
	ICONST_0,
	ICONST_1,
	ICONST_2,
	ICONST_3,
	ICONST_4,
	ICONST_5,
	LCONST_0,
	LCONST_1,
	// Stack operations
	DUP,
	DUP_X1,
	DUP_X2,
	DUP2,
	DUP2_X1,
	DUP2_X2,
	POP,
	POP2,
	SWAP,
	// Array
	NEWARRAY(BaseDesc),
	AALOAD,
	AASTORE,
	BALOAD,
	BASTORE,
	CALOAD,
	CASTORE,
	DALOAD,
	DASTORE,
	FALOAD,
	FASTORE,
	IALOAD,
	IASTORE,
	LALOAD,
	LASTORE,
	SALOAD,
	SASTORE,
	ARRAYLENGTH,
	ANEWARRAY(ConstPtr<ClassConst>),
	MULTIANEWARRAY {
		class: ConstPtr<ClassConst>,
		dimensions: u8,
	},
	// Math
	DADD,
	DDIV,
	DMUL,
	DNEG,
	DREM,
	DSUB,
	FADD,
	FDIV,
	FMUL,
	FNEG,
	FREM,
	FSUB,
	IADD,
	IDIV,
	IMUL,
	INEG,
	IREM,
	ISUB,
	IAND,
	IOR,
	ISHL,
	ISHR,
	IUSHR,
	IXOR,
	LADD,
	LDIV,
	LMUL,
	LNEG,
	LREM,
	LSUB,
	LAND,
	LOR,
	LSHL,
	LSHR,
	LUSHR,
	LXOR,
	// Conversions
	D2F,
	D2I,
	D2L,
	F2D,
	F2I,
	F2L,
	I2B,
	I2C,
	I2D,
	I2F,
	I2L,
	I2S,
	L2D,
	L2F,
	L2I,
	// Comparisons
	DCMPG,
	DCMPL,
	FCMPG,
	FCMPL,
	LCMP,
	// Jumps
	IF_ACMPEQ(BranchOffset),
	IF_ACMPNE(BranchOffset),
	IF_ICMPEQ(BranchOffset),
	IF_ICMPNE(BranchOffset),
	IF_ICMPLT(BranchOffset),
	IF_ICMPGE(BranchOffset),
	IF_ICMPGT(BranchOffset),
	IF_ICMPLE(BranchOffset),
	IFEQ(BranchOffset),
	IFNE(BranchOffset),
	IFLT(BranchOffset),
	IFGE(BranchOffset),
	IFGT(BranchOffset),
	IFLE(BranchOffset),
	IFNONNULL(BranchOffset),
	IFNULL(BranchOffset),
	GOTO(BranchOffset),
	GOTO_W(WideBranchOffset),
	// Locals
	ALOAD(u8),
	ALOAD_W(u16),
	ALOAD0,
	ALOAD1,
	ALOAD2,
	ALOAD3,
	DLOAD(u8),
	DLOAD_W(u16),
	DLOAD0,
	DLOAD1,
	DLOAD2,
	DLOAD3,
	FLOAD(u8),
	FLOAD_W(u16),
	FLOAD0,
	FLOAD1,
	FLOAD2,
	FLOAD3,
	ILOAD(u8),
	ILOAD_W(u16),
	ILOAD0,
	ILOAD1,
	ILOAD2,
	ILOAD3,
	LLOAD(u8),
	LLOAD_W(u16),
	LLOAD0,
	LLOAD1,
	LLOAD2,
	LLOAD3,
	ASTORE(u8),
	ASTORE_W(u16),
	ASTORE0,
	ASTORE1,
	ASTORE2,
	ASTORE3,
	DSTORE(u8),
	DSTORE_W(u16),
	DSTORE0,
	DSTORE1,
	DSTORE2,
	DSTORE3,
	FSTORE(u8),
	FSTORE_W(u16),
	FSTORE0,
	FSTORE1,
	FSTORE2,
	FSTORE3,
	ISTORE(u8),
	ISTORE_W(u16),
	ISTORE0,
	ISTORE1,
	ISTORE2,
	ISTORE3,
	LSTORE(u8),
	LSTORE_W(u16),
	LSTORE0,
	LSTORE1,
	LSTORE2,
	LSTORE3,

	IINC(u8, i8),
	IINC_W(u16, i16),
	// Return
	RETURN,
	ARETURN,
	DRETURN,
	FRETURN,
	IRETURN,
	LRETURN,
	// Misc
	NEW(ConstPtr<ClassConst>),
	ATHROW,
	BIPUSH(i8),
	SIPUSH(i16),
	CHECKCAST(ConstPtr<ClassConst>),
	INSTANCEOF(ConstPtr<ClassConst>),
	GETFIELD(ConstPtr<FieldConst>),
	GETSTATIC(ConstPtr<FieldConst>),
	PUTFIELD(ConstPtr<FieldConst>),
	PUTSTATIC(ConstPtr<FieldConst>),
	INVOKEDYNAMIC(ConstPtr<MethodConst>),
	INVOKEINTERFACE(ConstPtr<MethodConst>, u8),
	INVOKESPECIAL(ConstPtr<MethodConst>),
	INVOKESTATIC(ConstPtr<MethodConst>),
	INVOKEVIRTUAL(ConstPtr<MethodConst>),
	// Grandpa shit
	JSR(BranchOffset),
	JSR_W(WideBranchOffset),
	RET(u8),
	// ConstantPool Loading
	LDC(u8),
	LDC_W(u16),
	LDC2_W(u16),
	// TODO read
	LOOKUPSWITCH,
	TABLESWITCH,
	MONITORENTER,
	MONITOREXIT,
}

impl Inst {
	pub fn parse(input: &[u8]) -> IResult<Inst> {
		let (input, value): (_, u8) = be_u8(input)?;
		trace!("Parsed {}", print_op(value));
		Ok(match value {
			rvm_consts::NOP => (input, Inst::NOP),
			// Consts
			rvm_consts::ACONST_NULL => (input, Inst::ACONST_NULL),
			rvm_consts::DCONST_0 => (input, Inst::DCONST_0),
			rvm_consts::DCONST_1 => (input, Inst::DCONST_1),
			rvm_consts::FCONST_0 => (input, Inst::FCONST_0),
			rvm_consts::FCONST_1 => (input, Inst::FCONST_1),
			rvm_consts::FCONST_2 => (input, Inst::FCONST_2),
			rvm_consts::ICONST_M1 => (input, Inst::ICONST_M1),
			rvm_consts::ICONST_0 => (input, Inst::ICONST_0),
			rvm_consts::ICONST_1 => (input, Inst::ICONST_1),
			rvm_consts::ICONST_2 => (input, Inst::ICONST_2),
			rvm_consts::ICONST_3 => (input, Inst::ICONST_3),
			rvm_consts::ICONST_4 => (input, Inst::ICONST_4),
			rvm_consts::ICONST_5 => (input, Inst::ICONST_5),
			rvm_consts::LCONST_0 => (input, Inst::LCONST_0),
			rvm_consts::LCONST_1 => (input, Inst::LCONST_1),
			// Stack operations
			rvm_consts::DUP => (input, Inst::DUP),
			rvm_consts::DUP_X1 => (input, Inst::DUP_X1),
			rvm_consts::DUP_X2 => (input, Inst::DUP_X2),
			rvm_consts::DUP2 => (input, Inst::DUP2),
			rvm_consts::DUP2_X1 => (input, Inst::DUP2_X1),
			rvm_consts::DUP2_X2 => (input, Inst::DUP2_X2),
			rvm_consts::POP => (input, Inst::POP),
			rvm_consts::POP2 => (input, Inst::POP2),
			rvm_consts::SWAP => (input, Inst::SWAP),
			// Math
			rvm_consts::DADD => (input, Inst::DADD),
			rvm_consts::DDIV => (input, Inst::DDIV),
			rvm_consts::DMUL => (input, Inst::DMUL),
			rvm_consts::DNEG => (input, Inst::DNEG),
			rvm_consts::DREM => (input, Inst::DREM),
			rvm_consts::DSUB => (input, Inst::DSUB),
			rvm_consts::FADD => (input, Inst::FADD),
			rvm_consts::FDIV => (input, Inst::FDIV),
			rvm_consts::FMUL => (input, Inst::FMUL),
			rvm_consts::FNEG => (input, Inst::FNEG),
			rvm_consts::FREM => (input, Inst::FREM),
			rvm_consts::FSUB => (input, Inst::FSUB),
			rvm_consts::IADD => (input, Inst::IADD),
			rvm_consts::IDIV => (input, Inst::IDIV),
			rvm_consts::IMUL => (input, Inst::IMUL),
			rvm_consts::INEG => (input, Inst::INEG),
			rvm_consts::IREM => (input, Inst::IREM),
			rvm_consts::ISUB => (input, Inst::ISUB),
			rvm_consts::IAND => (input, Inst::IAND),
			rvm_consts::IOR => (input, Inst::IOR),
			rvm_consts::ISHL => (input, Inst::ISHL),
			rvm_consts::ISHR => (input, Inst::ISHR),
			rvm_consts::IUSHR => (input, Inst::IUSHR),
			rvm_consts::IXOR => (input, Inst::IXOR),
			rvm_consts::LADD => (input, Inst::LADD),
			rvm_consts::LDIV => (input, Inst::LDIV),
			rvm_consts::LMUL => (input, Inst::LMUL),
			rvm_consts::LNEG => (input, Inst::LNEG),
			rvm_consts::LREM => (input, Inst::LREM),
			rvm_consts::LSUB => (input, Inst::LSUB),
			rvm_consts::LAND => (input, Inst::LAND),
			rvm_consts::LOR => (input, Inst::LOR),
			rvm_consts::LSHL => (input, Inst::LSHL),
			rvm_consts::LSHR => (input, Inst::LSHR),
			rvm_consts::LUSHR => (input, Inst::LUSHR),
			rvm_consts::LXOR => (input, Inst::LXOR),
			// Conversions
			rvm_consts::D2F => (input, Inst::D2F),
			rvm_consts::D2I => (input, Inst::D2I),
			rvm_consts::D2L => (input, Inst::D2L),
			rvm_consts::F2D => (input, Inst::F2D),
			rvm_consts::F2I => (input, Inst::F2I),
			rvm_consts::F2L => (input, Inst::F2L),
			rvm_consts::I2B => (input, Inst::I2B),
			rvm_consts::I2C => (input, Inst::I2C),
			rvm_consts::I2D => (input, Inst::I2D),
			rvm_consts::I2F => (input, Inst::I2F),
			rvm_consts::I2L => (input, Inst::I2L),
			rvm_consts::I2S => (input, Inst::I2S),
			rvm_consts::L2D => (input, Inst::L2D),
			rvm_consts::L2F => (input, Inst::L2F),
			rvm_consts::L2I => (input, Inst::L2I),
			// Comparisons
			rvm_consts::DCMPG => (input, Inst::DCMPG),
			rvm_consts::DCMPL => (input, Inst::DCMPL),
			rvm_consts::FCMPG => (input, Inst::FCMPG),
			rvm_consts::FCMPL => (input, Inst::FCMPL),
			rvm_consts::LCMP => (input, Inst::LCMP),
			// Return
			rvm_consts::RETURN => (input, Inst::RETURN),
			rvm_consts::ARETURN => (input, Inst::ARETURN),
			rvm_consts::DRETURN => (input, Inst::DRETURN),
			rvm_consts::FRETURN => (input, Inst::FRETURN),
			rvm_consts::IRETURN => (input, Inst::IRETURN),
			rvm_consts::LRETURN => (input, Inst::LRETURN),

			// Array
			rvm_consts::AALOAD => (input, Inst::AALOAD),
			rvm_consts::AASTORE => (input, Inst::AASTORE),
			rvm_consts::BALOAD => (input, Inst::BALOAD),
			rvm_consts::BASTORE => (input, Inst::BASTORE),
			rvm_consts::CALOAD => (input, Inst::CALOAD),
			rvm_consts::CASTORE => (input, Inst::CASTORE),
			rvm_consts::DALOAD => (input, Inst::DALOAD),
			rvm_consts::DASTORE => (input, Inst::DASTORE),
			rvm_consts::FALOAD => (input, Inst::FALOAD),
			rvm_consts::FASTORE => (input, Inst::FASTORE),
			rvm_consts::IALOAD => (input, Inst::IALOAD),
			rvm_consts::IASTORE => (input, Inst::IASTORE),
			rvm_consts::LALOAD => (input, Inst::LALOAD),
			rvm_consts::LASTORE => (input, Inst::LASTORE),
			rvm_consts::SALOAD => (input, Inst::SALOAD),
			rvm_consts::SASTORE => (input, Inst::SASTORE),
			rvm_consts::ARRAYLENGTH => (input, Inst::ARRAYLENGTH),
			rvm_consts::NEWARRAY => map(be_u8, |v: u8| {
				Inst::NEWARRAY(match v {
					4 => BaseDesc::Boolean,
					5 => BaseDesc::Char,
					6 => BaseDesc::Float,
					7 => BaseDesc::Double,
					8 => BaseDesc::Byte,
					9 => BaseDesc::Short,
					10 => BaseDesc::Int,
					11 => BaseDesc::Long,
					_ => {
						panic!("Invalid type")
					}
				})
			})(input)?,
			rvm_consts::ANEWARRAY => map(be_cp, Inst::ANEWARRAY)(input)?,
			rvm_consts::MULTIANEWARRAY => map(tuple((be_cp, be_u8)), |(class, dimensions)| {
				Inst::MULTIANEWARRAY { class, dimensions }
			})(input)?,
			// Jumps
			rvm_consts::IF_ACMPEQ => map(be_i16, |v| Inst::IF_ACMPEQ(BranchOffset(v)))(input)?,
			rvm_consts::IF_ACMPNE => map(be_i16, |v| Inst::IF_ACMPNE(BranchOffset(v)))(input)?,
			rvm_consts::IF_ICMPEQ => map(be_i16, |v| Inst::IF_ICMPEQ(BranchOffset(v)))(input)?,
			rvm_consts::IF_ICMPNE => map(be_i16, |v| Inst::IF_ICMPNE(BranchOffset(v)))(input)?,
			rvm_consts::IF_ICMPLT => map(be_i16, |v| Inst::IF_ICMPLT(BranchOffset(v)))(input)?,
			rvm_consts::IF_ICMPGE => map(be_i16, |v| Inst::IF_ICMPGE(BranchOffset(v)))(input)?,
			rvm_consts::IF_ICMPGT => map(be_i16, |v| Inst::IF_ICMPGT(BranchOffset(v)))(input)?,
			rvm_consts::IF_ICMPLE => map(be_i16, |v| Inst::IF_ICMPLE(BranchOffset(v)))(input)?,
			rvm_consts::IFEQ => map(be_i16, |v| Inst::IFEQ(BranchOffset(v)))(input)?,
			rvm_consts::IFNE => map(be_i16, |v| Inst::IFNE(BranchOffset(v)))(input)?,
			rvm_consts::IFLT => map(be_i16, |v| Inst::IFLT(BranchOffset(v)))(input)?,
			rvm_consts::IFGE => map(be_i16, |v| Inst::IFGE(BranchOffset(v)))(input)?,
			rvm_consts::IFGT => map(be_i16, |v| Inst::IFGT(BranchOffset(v)))(input)?,
			rvm_consts::IFLE => map(be_i16, |v| Inst::IFLE(BranchOffset(v)))(input)?,
			rvm_consts::IFNONNULL => map(be_i16, |v| Inst::IFNONNULL(BranchOffset(v)))(input)?,
			rvm_consts::IFNULL => map(be_i16, |v| Inst::IFNULL(BranchOffset(v)))(input)?,
			rvm_consts::GOTO => map(be_i16, |v| Inst::GOTO(BranchOffset(v)))(input)?,
			rvm_consts::GOTO_W => map(be_i32, |v| Inst::GOTO_W(WideBranchOffset(v)))(input)?,
			// Locals
			rvm_consts::ALOAD => map(be_u8, Inst::ALOAD)(input)?,
			rvm_consts::ALOAD_0 => (input, Inst::ALOAD0),
			rvm_consts::ALOAD_1 => (input, Inst::ALOAD1),
			rvm_consts::ALOAD_2 => (input, Inst::ALOAD2),
			rvm_consts::ALOAD_3 => (input, Inst::ALOAD3),
			rvm_consts::DLOAD => map(be_u8, Inst::DLOAD)(input)?,
			rvm_consts::DLOAD_0 => (input, Inst::DLOAD0),
			rvm_consts::DLOAD_1 => (input, Inst::DLOAD1),
			rvm_consts::DLOAD_2 => (input, Inst::DLOAD2),
			rvm_consts::DLOAD_3 => (input, Inst::DLOAD3),
			rvm_consts::FLOAD => map(be_u8, Inst::FLOAD)(input)?,
			rvm_consts::FLOAD_0 => (input, Inst::FLOAD0),
			rvm_consts::FLOAD_1 => (input, Inst::FLOAD1),
			rvm_consts::FLOAD_2 => (input, Inst::FLOAD2),
			rvm_consts::FLOAD_3 => (input, Inst::FLOAD3),
			rvm_consts::ILOAD => map(be_u8, Inst::ILOAD)(input)?,
			rvm_consts::ILOAD_0 => (input, Inst::ILOAD0),
			rvm_consts::ILOAD_1 => (input, Inst::ILOAD1),
			rvm_consts::ILOAD_2 => (input, Inst::ILOAD2),
			rvm_consts::ILOAD_3 => (input, Inst::ILOAD3),
			rvm_consts::LLOAD => map(be_u8, Inst::LLOAD)(input)?,
			rvm_consts::LLOAD_0 => (input, Inst::LLOAD0),
			rvm_consts::LLOAD_1 => (input, Inst::LLOAD1),
			rvm_consts::LLOAD_2 => (input, Inst::LLOAD2),
			rvm_consts::LLOAD_3 => (input, Inst::LLOAD3),
			rvm_consts::ASTORE => map(be_u8, Inst::ASTORE)(input)?,
			rvm_consts::ASTORE_0 => (input, Inst::ASTORE0),
			rvm_consts::ASTORE_1 => (input, Inst::ASTORE1),
			rvm_consts::ASTORE_2 => (input, Inst::ASTORE2),
			rvm_consts::ASTORE_3 => (input, Inst::ASTORE3),
			rvm_consts::DSTORE => map(be_u8, Inst::DSTORE)(input)?,
			rvm_consts::DSTORE_0 => (input, Inst::DSTORE0),
			rvm_consts::DSTORE_1 => (input, Inst::DSTORE1),
			rvm_consts::DSTORE_2 => (input, Inst::DSTORE2),
			rvm_consts::DSTORE_3 => (input, Inst::DSTORE3),
			rvm_consts::FSTORE => map(be_u8, Inst::FSTORE)(input)?,
			rvm_consts::FSTORE_0 => (input, Inst::FSTORE0),
			rvm_consts::FSTORE_1 => (input, Inst::FSTORE1),
			rvm_consts::FSTORE_2 => (input, Inst::FSTORE2),
			rvm_consts::FSTORE_3 => (input, Inst::FSTORE3),
			rvm_consts::ISTORE => map(be_u8, Inst::ISTORE)(input)?,
			rvm_consts::ISTORE_0 => (input, Inst::ISTORE0),
			rvm_consts::ISTORE_1 => (input, Inst::ISTORE1),
			rvm_consts::ISTORE_2 => (input, Inst::ISTORE2),
			rvm_consts::ISTORE_3 => (input, Inst::ISTORE3),
			rvm_consts::LSTORE => map(be_u8, Inst::LSTORE)(input)?,
			rvm_consts::LSTORE_0 => (input, Inst::LSTORE0),
			rvm_consts::LSTORE_1 => (input, Inst::LSTORE1),
			rvm_consts::LSTORE_2 => (input, Inst::LSTORE2),
			rvm_consts::LSTORE_3 => (input, Inst::LSTORE3),
			rvm_consts::IINC => map(tuple((be_u8, be_i8)), |(v0, v1)| Inst::IINC(v0, v1))(input)?,
			// ConstantPool Loading
			rvm_consts::LDC => map(be_u8, Inst::LDC)(input)?,
			rvm_consts::LDC_W => map(be_u16, Inst::LDC_W)(input)?,
			rvm_consts::LDC2_W => map(be_u16, Inst::LDC2_W)(input)?,
			// Misc
			rvm_consts::NEW => map(be_cp, Inst::NEW)(input)?,
			rvm_consts::ATHROW => (input, Inst::ATHROW),
			rvm_consts::BIPUSH => map(be_i8, Inst::BIPUSH)(input)?,
			rvm_consts::SIPUSH => map(be_i16, Inst::SIPUSH)(input)?,
			rvm_consts::CHECKCAST => map(be_cp, Inst::CHECKCAST)(input)?,
			rvm_consts::INSTANCEOF => map(be_cp, Inst::INSTANCEOF)(input)?,
			rvm_consts::GETFIELD => map(be_cp, Inst::GETFIELD)(input)?,
			rvm_consts::GETSTATIC => map(be_cp, Inst::GETSTATIC)(input)?,
			rvm_consts::PUTFIELD => map(be_cp, Inst::PUTFIELD)(input)?,
			rvm_consts::PUTSTATIC => map(be_cp, Inst::PUTSTATIC)(input)?,
			rvm_consts::INVOKEDYNAMIC => {
				map(tuple((be_cp, be_u16)), |(v, _)| Inst::INVOKEDYNAMIC(v))(input)?
			}
			rvm_consts::INVOKEINTERFACE => map(tuple((be_cp, be_u8, be_u8)), |(v, count, _)| {
				Inst::INVOKEINTERFACE(v, count)
			})(input)?,
			rvm_consts::INVOKESPECIAL => map(be_cp, Inst::INVOKESPECIAL)(input)?,
			rvm_consts::INVOKESTATIC => map(be_cp, Inst::INVOKESTATIC)(input)?,
			rvm_consts::INVOKEVIRTUAL => map(be_cp, Inst::INVOKEVIRTUAL)(input)?,
			v => {
				panic!("instruction {v}:{} kinda dodo", print_op(v))
			}
		})
	}
}

#[derive(Clone, Debug)]
pub struct BranchOffset(pub i16);
#[derive(Clone, Debug)]
pub struct WideBranchOffset(pub i32);
