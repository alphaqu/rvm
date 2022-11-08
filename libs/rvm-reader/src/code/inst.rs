use crate::{be_cp, IResult};
use crate::{ClassConst, ConstPtr, FieldConst, MethodConst};
use nom::combinator::map;
use nom::number::complete::{be_i16, be_i32, be_i8, be_u16, be_u8};
use nom::sequence::tuple;
use rvm_consts::print_op;
use rvm_core::{Kind, PrimitiveType, StackKind};
use tracing::trace;

#[derive(Copy, Clone, Debug)]
pub enum ConstInst {
	Null,
	Int(i32),
	Long(i64),
	Float(f32),
	Double(f64),
}

#[derive(Copy, Clone, Debug)]
pub enum StackInst {
	DUP,
	DUP_X1,
	DUP_X2,
	DUP2,
	DUP2_X1,
	DUP2_X2,
	POP,
	POP2,
	SWAP,
}

#[derive(Copy, Clone, Debug)]
pub enum ArrayInst {
	Length,
	Load(Kind),
	Store(Kind),
	NewPrim(PrimitiveType),
	NewRef(ConstPtr<ClassConst>),
	NewMultiRef {
		class: ConstPtr<ClassConst>,
		dimensions: u8,
	},
}

#[derive(Copy, Clone, Debug)]
pub enum MathInst {
	Add(PrimitiveType),
	Sub(PrimitiveType),
	Div(PrimitiveType),
	Mul(PrimitiveType),
	Rem(PrimitiveType),
	Neg(PrimitiveType),
	And(PrimitiveType),
	Or(PrimitiveType),
	Xor(PrimitiveType),
	Shl(PrimitiveType),
	Shr(PrimitiveType),
	Ushr(PrimitiveType),
}

#[derive(Copy, Clone, Debug)]
pub enum ConversionInst {
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
}

#[derive(Copy, Clone, Debug)]
pub enum ComparisonInst {
	DCMPG,
	DCMPL,
	FCMPG,
	FCMPL,
	LCMP,
}

#[derive(Copy, Clone, Debug)]
pub struct JumpInst {
	offset: i32,
	kind: JumpKind,
}

#[derive(Copy, Clone, Debug)]
pub enum JumpKind {
	IF_ACMPEQ,
	IF_ACMPNE,
	IF_ICMPEQ,
	IF_ICMPNE,
	IF_ICMPLT,
	IF_ICMPGE,
	IF_ICMPGT,
	IF_ICMPLE,
	IFEQ,
	IFNE,
	IFLT,
	IFGE,
	IFGT,
	IFLE,
	IFNONNULL,
	IFNULL,
	GOTO,
}

#[derive(Copy, Clone, Debug)]
pub enum LocalInst {
	Load(StackKind, u16),
	Store(StackKind, u16),
	Increment(i16, u16),
}

#[derive(Copy, Clone, Debug)]
pub struct ReturnInst {
	value: Option<StackKind>,
}

#[derive(Copy, Clone, Debug)]
pub struct NewInst {
	class: ConstPtr<ClassConst>,
}

#[derive(Copy, Clone, Debug)]
pub struct ThrowInst {
	// TODO
}

#[derive(Copy, Clone, Debug)]
pub struct PushInst(i16);

#[derive(Copy, Clone, Debug)]
pub struct CheckCastInst {
	value: ConstPtr<ClassConst>,
}

#[derive(Copy, Clone, Debug)]
pub struct InstanceOfInst {
	value: ConstPtr<ClassConst>,
}

#[derive(Copy, Clone, Debug)]
pub struct FieldInst {
	value: ConstPtr<FieldConst>,
	instance: bool,
	kind: FieldInstKind,
}

#[derive(Copy, Clone, Debug)]
pub enum FieldInstKind {
	Get,
	Put,
}

#[derive(Copy, Clone, Debug)]
pub struct InvokeInst {
	value: ConstPtr<MethodConst>,
	kind: InvokeInstKind,
}

#[derive(Copy, Clone, Debug)]
pub enum InvokeInstKind {
	Dynamic,
	Interface(u8),
	Special,
	Static,
	Virtual,
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
pub enum Inst {
	Nop,
	Const(ConstInst),
	Stack(StackInst),
	Array(ArrayInst),
	Math(MathInst),
	Conversion(ConversionInst),
	Comparison(ComparisonInst),
	Jump(JumpInst),
	Local(LocalInst),
	Return(ReturnInst),
	New(NewInst),
	Throw(ThrowInst),
	Push(PushInst),
	CheckCast(CheckCastInst),
	InstanceOf(InstanceOfInst),
	Field(FieldInst),
	Invoke(InvokeInst),
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
			rvm_consts::NOP => (input, Inst::Nop),
			// Consts
			rvm_consts::ACONST_NULL => (input, Inst::Const(ConstInst::Null)),
			rvm_consts::DCONST_0 => (input, Inst::Const(ConstInst::Double(0.0))),
			rvm_consts::DCONST_1 => (input, Inst::Const(ConstInst::Double(1.0))),
			rvm_consts::FCONST_0 => (input, Inst::Const(ConstInst::Float(0.0))),
			rvm_consts::FCONST_1 => (input, Inst::Const(ConstInst::Float(1.0))),
			rvm_consts::FCONST_2 => (input, Inst::Const(ConstInst::Float(2.0))),
			rvm_consts::ICONST_M1 => (input, Inst::Const(ConstInst::Int(-1))),
			rvm_consts::ICONST_0 => (input, Inst::Const(ConstInst::Int(0))),
			rvm_consts::ICONST_1 => (input, Inst::Const(ConstInst::Int(1))),
			rvm_consts::ICONST_2 => (input, Inst::Const(ConstInst::Int(2))),
			rvm_consts::ICONST_3 => (input, Inst::Const(ConstInst::Int(3))),
			rvm_consts::ICONST_4 => (input, Inst::Const(ConstInst::Int(4))),
			rvm_consts::ICONST_5 => (input, Inst::Const(ConstInst::Int(5))),
			rvm_consts::LCONST_0 => (input, Inst::Const(ConstInst::Long(0))),
			rvm_consts::LCONST_1 => (input, Inst::Const(ConstInst::Long(1))),
			// Stack operations
			rvm_consts::DUP => (input, Inst::Stack(StackInst::DUP)),
			rvm_consts::DUP_X1 => (input, Inst::Stack(StackInst::DUP_X1)),
			rvm_consts::DUP_X2 => (input, Inst::Stack(StackInst::DUP_X2)),
			rvm_consts::DUP2 => (input, Inst::Stack(StackInst::DUP2)),
			rvm_consts::DUP2_X1 => (input, Inst::Stack(StackInst::DUP2_X1)),
			rvm_consts::DUP2_X2 => (input, Inst::Stack(StackInst::DUP2_X2)),
			rvm_consts::POP => (input, Inst::Stack(StackInst::POP)),
			rvm_consts::POP2 => (input, Inst::Stack(StackInst::POP2)),
			rvm_consts::SWAP => (input, Inst::Stack(StackInst::SWAP)),
			// Math
			rvm_consts::DADD => (input, Inst::Math(MathInst::Add(PrimitiveType::Double))),
			rvm_consts::DDIV => (input, Inst::Math(MathInst::Div(PrimitiveType::Double))),
			rvm_consts::DMUL => (input, Inst::Math(MathInst::Mul(PrimitiveType::Double))),
			rvm_consts::DNEG => (input, Inst::Math(MathInst::Neg(PrimitiveType::Double))),
			rvm_consts::DREM => (input, Inst::Math(MathInst::Rem(PrimitiveType::Double))),
			rvm_consts::DSUB => (input, Inst::Math(MathInst::Sub(PrimitiveType::Double))),
			rvm_consts::FADD => (input, Inst::Math(MathInst::Add(PrimitiveType::Float))),
			rvm_consts::FDIV => (input, Inst::Math(MathInst::Div(PrimitiveType::Float))),
			rvm_consts::FMUL => (input, Inst::Math(MathInst::Mul(PrimitiveType::Float))),
			rvm_consts::FNEG => (input, Inst::Math(MathInst::Neg(PrimitiveType::Float))),
			rvm_consts::FREM => (input, Inst::Math(MathInst::Rem(PrimitiveType::Float))),
			rvm_consts::FSUB => (input, Inst::Math(MathInst::Sub(PrimitiveType::Float))),
			rvm_consts::IADD => (input, Inst::Math(MathInst::Add(PrimitiveType::Int))),
			rvm_consts::IDIV => (input, Inst::Math(MathInst::Div(PrimitiveType::Int))),
			rvm_consts::IMUL => (input, Inst::Math(MathInst::Mul(PrimitiveType::Int))),
			rvm_consts::INEG => (input, Inst::Math(MathInst::Neg(PrimitiveType::Int))),
			rvm_consts::IREM => (input, Inst::Math(MathInst::Rem(PrimitiveType::Int))),
			rvm_consts::ISUB => (input, Inst::Math(MathInst::Sub(PrimitiveType::Int))),
			rvm_consts::IAND => (input, Inst::Math(MathInst::And(PrimitiveType::Int))),
			rvm_consts::IOR => (input, Inst::Math(MathInst::Or(PrimitiveType::Int))),
			rvm_consts::ISHL => (input, Inst::Math(MathInst::Shl(PrimitiveType::Int))),
			rvm_consts::ISHR => (input, Inst::Math(MathInst::Shr(PrimitiveType::Int))),
			rvm_consts::IUSHR => (input, Inst::Math(MathInst::Ushr(PrimitiveType::Int))),
			rvm_consts::IXOR => (input, Inst::Math(MathInst::Xor(PrimitiveType::Int))),
			rvm_consts::LADD => (input, Inst::Math(MathInst::Add(PrimitiveType::Long))),
			rvm_consts::LDIV => (input, Inst::Math(MathInst::Div(PrimitiveType::Long))),
			rvm_consts::LMUL => (input, Inst::Math(MathInst::Mul(PrimitiveType::Long))),
			rvm_consts::LNEG => (input, Inst::Math(MathInst::Neg(PrimitiveType::Long))),
			rvm_consts::LREM => (input, Inst::Math(MathInst::Rem(PrimitiveType::Long))),
			rvm_consts::LSUB => (input, Inst::Math(MathInst::Sub(PrimitiveType::Long))),
			rvm_consts::LAND => (input, Inst::Math(MathInst::And(PrimitiveType::Long))),
			rvm_consts::LOR => (input, Inst::Math(MathInst::Or(PrimitiveType::Long))),
			rvm_consts::LSHL => (input, Inst::Math(MathInst::Shl(PrimitiveType::Long))),
			rvm_consts::LSHR => (input, Inst::Math(MathInst::Shr(PrimitiveType::Long))),
			rvm_consts::LUSHR => (input, Inst::Math(MathInst::Ushr(PrimitiveType::Long))),
			rvm_consts::LXOR => (input, Inst::Math(MathInst::Xor(PrimitiveType::Long))),
			// Conversions
			rvm_consts::D2F => (input, Inst::Conversion(ConversionInst::D2F)),
			rvm_consts::D2I => (input, Inst::Conversion(ConversionInst::D2I)),
			rvm_consts::D2L => (input, Inst::Conversion(ConversionInst::D2L)),
			rvm_consts::F2D => (input, Inst::Conversion(ConversionInst::F2D)),
			rvm_consts::F2I => (input, Inst::Conversion(ConversionInst::F2I)),
			rvm_consts::F2L => (input, Inst::Conversion(ConversionInst::F2L)),
			rvm_consts::I2B => (input, Inst::Conversion(ConversionInst::I2B)),
			rvm_consts::I2C => (input, Inst::Conversion(ConversionInst::I2C)),
			rvm_consts::I2D => (input, Inst::Conversion(ConversionInst::I2D)),
			rvm_consts::I2F => (input, Inst::Conversion(ConversionInst::I2F)),
			rvm_consts::I2L => (input, Inst::Conversion(ConversionInst::I2L)),
			rvm_consts::I2S => (input, Inst::Conversion(ConversionInst::I2S)),
			rvm_consts::L2D => (input, Inst::Conversion(ConversionInst::L2D)),
			rvm_consts::L2F => (input, Inst::Conversion(ConversionInst::L2F)),
			rvm_consts::L2I => (input, Inst::Conversion(ConversionInst::L2I)),
			// Comparisons
			rvm_consts::DCMPG => (input, Inst::Comparison(ComparisonInst::DCMPG)),
			rvm_consts::DCMPL => (input, Inst::Comparison(ComparisonInst::DCMPL)),
			rvm_consts::FCMPG => (input, Inst::Comparison(ComparisonInst::FCMPG)),
			rvm_consts::FCMPL => (input, Inst::Comparison(ComparisonInst::FCMPL)),
			rvm_consts::LCMP => (input, Inst::Comparison(ComparisonInst::LCMP)),
			// Return
			rvm_consts::RETURN => (input, Inst::Return(ReturnInst { value: None })),
			rvm_consts::ARETURN => (
				input,
				Inst::Return(ReturnInst {
					value: Some(StackKind::Reference),
				}),
			),
			rvm_consts::DRETURN => (
				input,
				Inst::Return(ReturnInst {
					value: Some(StackKind::Double),
				}),
			),
			rvm_consts::FRETURN => (
				input,
				Inst::Return(ReturnInst {
					value: Some(StackKind::Float),
				}),
			),
			rvm_consts::IRETURN => (
				input,
				Inst::Return(ReturnInst {
					value: Some(StackKind::Int),
				}),
			),
			rvm_consts::LRETURN => (
				input,
				Inst::Return(ReturnInst {
					value: Some(StackKind::Long),
				}),
			),

			// Array
			rvm_consts::AALOAD => (input, Inst::Array(ArrayInst::Load(Kind::Reference))),
			rvm_consts::BALOAD => (input, Inst::Array(ArrayInst::Load(Kind::Byte))),
			rvm_consts::CALOAD => (input, Inst::Array(ArrayInst::Load(Kind::Char))),
			rvm_consts::DALOAD => (input, Inst::Array(ArrayInst::Load(Kind::Double))),
			rvm_consts::FALOAD => (input, Inst::Array(ArrayInst::Load(Kind::Float))),
			rvm_consts::IALOAD => (input, Inst::Array(ArrayInst::Load(Kind::Int))),
			rvm_consts::LALOAD => (input, Inst::Array(ArrayInst::Load(Kind::Long))),
			rvm_consts::SALOAD => (input, Inst::Array(ArrayInst::Load(Kind::Short))),
			rvm_consts::AASTORE => (input, Inst::Array(ArrayInst::Store(Kind::Reference))),
			rvm_consts::BASTORE => (input, Inst::Array(ArrayInst::Store(Kind::Byte))),
			rvm_consts::CASTORE => (input, Inst::Array(ArrayInst::Store(Kind::Char))),
			rvm_consts::DASTORE => (input, Inst::Array(ArrayInst::Store(Kind::Double))),
			rvm_consts::FASTORE => (input, Inst::Array(ArrayInst::Store(Kind::Float))),
			rvm_consts::IASTORE => (input, Inst::Array(ArrayInst::Store(Kind::Int))),
			rvm_consts::LASTORE => (input, Inst::Array(ArrayInst::Store(Kind::Long))),
			rvm_consts::SASTORE => (input, Inst::Array(ArrayInst::Store(Kind::Short))),
			rvm_consts::ARRAYLENGTH => (input, Inst::Array(ArrayInst::Length)),
			rvm_consts::NEWARRAY => map(be_u8, |v: u8| {
				Inst::Array(ArrayInst::NewPrim(
					(match v {
						4 => PrimitiveType::Boolean,
						5 => PrimitiveType::Char,
						6 => PrimitiveType::Float,
						7 => PrimitiveType::Double,
						8 => PrimitiveType::Byte,
						9 => PrimitiveType::Short,
						10 => PrimitiveType::Int,
						11 => PrimitiveType::Long,
						_ => {
							panic!("Invalid type")
						}
					}),
				))
			})(input)?,
			rvm_consts::ANEWARRAY => map(be_cp, |v| Inst::Array(ArrayInst::NewRef(v)))(input)?,
			rvm_consts::MULTIANEWARRAY => map(tuple((be_cp, be_u8)), |(class, dimensions)| {
				Inst::Array(ArrayInst::NewMultiRef { class, dimensions })
			})(input)?,
			// Jumps
			rvm_consts::IF_ACMPEQ => map(be_i16, |v| {
				Inst::Jump(JumpInst {
					offset: v as i32,
					kind: JumpKind::IF_ACMPEQ,
				})
			})(input)?,
			rvm_consts::IF_ACMPNE => map(be_i16, |v| {
				Inst::Jump(JumpInst {
					offset: v as i32,
					kind: JumpKind::IF_ACMPNE,
				})
			})(input)?,
			rvm_consts::IF_ICMPEQ => map(be_i16, |v| {
				Inst::Jump(JumpInst {
					offset: v as i32,
					kind: JumpKind::IF_ICMPEQ,
				})
			})(input)?,
			rvm_consts::IF_ICMPNE => map(be_i16, |v| {
				Inst::Jump(JumpInst {
					offset: v as i32,
					kind: JumpKind::IF_ICMPNE,
				})
			})(input)?,
			rvm_consts::IF_ICMPLT => map(be_i16, |v| {
				Inst::Jump(JumpInst {
					offset: v as i32,
					kind: JumpKind::IF_ICMPLT,
				})
			})(input)?,
			rvm_consts::IF_ICMPGE => map(be_i16, |v| {
				Inst::Jump(JumpInst {
					offset: v as i32,
					kind: JumpKind::IF_ICMPGE,
				})
			})(input)?,
			rvm_consts::IF_ICMPGT => map(be_i16, |v| {
				Inst::Jump(JumpInst {
					offset: v as i32,
					kind: JumpKind::IF_ICMPGT,
				})
			})(input)?,
			rvm_consts::IF_ICMPLE => map(be_i16, |v| {
				Inst::Jump(JumpInst {
					offset: v as i32,
					kind: JumpKind::IF_ICMPLE,
				})
			})(input)?,
			rvm_consts::IFEQ => map(be_i16, |v| {
				Inst::Jump(JumpInst {
					offset: v as i32,
					kind: JumpKind::IFEQ,
				})
			})(input)?,
			rvm_consts::IFNE => map(be_i16, |v| {
				Inst::Jump(JumpInst {
					offset: v as i32,
					kind: JumpKind::IFNE,
				})
			})(input)?,
			rvm_consts::IFLT => map(be_i16, |v| {
				Inst::Jump(JumpInst {
					offset: v as i32,
					kind: JumpKind::IFLT,
				})
			})(input)?,
			rvm_consts::IFGE => map(be_i16, |v| {
				Inst::Jump(JumpInst {
					offset: v as i32,
					kind: JumpKind::IFGE,
				})
			})(input)?,
			rvm_consts::IFGT => map(be_i16, |v| {
				Inst::Jump(JumpInst {
					offset: v as i32,
					kind: JumpKind::IFGT,
				})
			})(input)?,
			rvm_consts::IFLE => map(be_i16, |v| {
				Inst::Jump(JumpInst {
					offset: v as i32,
					kind: JumpKind::IFLE,
				})
			})(input)?,
			rvm_consts::IFNONNULL => map(be_i16, |v| {
				Inst::Jump(JumpInst {
					offset: v as i32,
					kind: JumpKind::IFNONNULL,
				})
			})(input)?,
			rvm_consts::IFNULL => map(be_i16, |v| {
				Inst::Jump(JumpInst {
					offset: v as i32,
					kind: JumpKind::IFNULL,
				})
			})(input)?,
			rvm_consts::GOTO => map(be_i16, |v| {
				Inst::Jump(JumpInst {
					offset: v as i32,
					kind: JumpKind::GOTO,
				})
			})(input)?,
			rvm_consts::GOTO_W => map(be_i32, |v| {
				Inst::Jump(JumpInst {
					offset: v as i32,
					kind: JumpKind::GOTO,
				})
			})(input)?,
			// Locals
			rvm_consts::ALOAD => map(be_u8, |v| {
				Inst::Local(LocalInst::Load(StackKind::Reference, v as u16))
			})(input)?,
			rvm_consts::ALOAD_0 => (
				input,
				Inst::Local(LocalInst::Load(StackKind::Reference, 0 as u16)),
			),
			rvm_consts::ALOAD_1 => (
				input,
				Inst::Local(LocalInst::Load(StackKind::Reference, 1 as u16)),
			),
			rvm_consts::ALOAD_2 => (
				input,
				Inst::Local(LocalInst::Load(StackKind::Reference, 2 as u16)),
			),
			rvm_consts::ALOAD_3 => (
				input,
				Inst::Local(LocalInst::Load(StackKind::Reference, 3 as u16)),
			),
			rvm_consts::DLOAD => map(be_u8, |v| {
				Inst::Local(LocalInst::Load(StackKind::Double, v as u16))
			})(input)?,
			rvm_consts::DLOAD_0 => (
				input,
				Inst::Local(LocalInst::Load(StackKind::Double, 0 as u16)),
			),
			rvm_consts::DLOAD_1 => (
				input,
				Inst::Local(LocalInst::Load(StackKind::Double, 1 as u16)),
			),
			rvm_consts::DLOAD_2 => (
				input,
				Inst::Local(LocalInst::Load(StackKind::Double, 2 as u16)),
			),
			rvm_consts::DLOAD_3 => (
				input,
				Inst::Local(LocalInst::Load(StackKind::Double, 3 as u16)),
			),
			rvm_consts::FLOAD => map(be_u8, |v| {
				Inst::Local(LocalInst::Load(StackKind::Float, v as u16))
			})(input)?,
			rvm_consts::FLOAD_0 => (
				input,
				Inst::Local(LocalInst::Load(StackKind::Float, 0 as u16)),
			),
			rvm_consts::FLOAD_1 => (
				input,
				Inst::Local(LocalInst::Load(StackKind::Float, 1 as u16)),
			),
			rvm_consts::FLOAD_2 => (
				input,
				Inst::Local(LocalInst::Load(StackKind::Float, 2 as u16)),
			),
			rvm_consts::FLOAD_3 => (
				input,
				Inst::Local(LocalInst::Load(StackKind::Float, 3 as u16)),
			),
			rvm_consts::ILOAD => map(be_u8, |v| {
				Inst::Local(LocalInst::Load(StackKind::Int, v as u16))
			})(input)?,
			rvm_consts::ILOAD_0 => (
				input,
				Inst::Local(LocalInst::Load(StackKind::Int, 0 as u16)),
			),
			rvm_consts::ILOAD_1 => (
				input,
				Inst::Local(LocalInst::Load(StackKind::Int, 1 as u16)),
			),
			rvm_consts::ILOAD_2 => (
				input,
				Inst::Local(LocalInst::Load(StackKind::Int, 2 as u16)),
			),
			rvm_consts::ILOAD_3 => (
				input,
				Inst::Local(LocalInst::Load(StackKind::Int, 3 as u16)),
			),
			rvm_consts::LLOAD => map(be_u8, |v| {
				Inst::Local(LocalInst::Load(StackKind::Long, v as u16))
			})(input)?,
			rvm_consts::LLOAD_0 => (
				input,
				Inst::Local(LocalInst::Load(StackKind::Long, 0 as u16)),
			),
			rvm_consts::LLOAD_1 => (
				input,
				Inst::Local(LocalInst::Load(StackKind::Long, 1 as u16)),
			),
			rvm_consts::LLOAD_2 => (
				input,
				Inst::Local(LocalInst::Load(StackKind::Long, 2 as u16)),
			),
			rvm_consts::LLOAD_3 => (
				input,
				Inst::Local(LocalInst::Load(StackKind::Long, 3 as u16)),
			),
			rvm_consts::ASTORE => map(be_u8, |v| {
				Inst::Local(LocalInst::Store(StackKind::Reference, v as u16))
			})(input)?,
			rvm_consts::ASTORE_0 => (
				input,
				Inst::Local(LocalInst::Store(StackKind::Reference, 0 as u16)),
			),
			rvm_consts::ASTORE_1 => (
				input,
				Inst::Local(LocalInst::Store(StackKind::Reference, 1 as u16)),
			),
			rvm_consts::ASTORE_2 => (
				input,
				Inst::Local(LocalInst::Store(StackKind::Reference, 2 as u16)),
			),
			rvm_consts::ASTORE_3 => (
				input,
				Inst::Local(LocalInst::Store(StackKind::Reference, 3 as u16)),
			),
			rvm_consts::DSTORE => map(be_u8, |v| {
				Inst::Local(LocalInst::Store(StackKind::Double, v as u16))
			})(input)?,
			rvm_consts::DSTORE_0 => (
				input,
				Inst::Local(LocalInst::Store(StackKind::Double, 0 as u16)),
			),
			rvm_consts::DSTORE_1 => (
				input,
				Inst::Local(LocalInst::Store(StackKind::Double, 1 as u16)),
			),
			rvm_consts::DSTORE_2 => (
				input,
				Inst::Local(LocalInst::Store(StackKind::Double, 2 as u16)),
			),
			rvm_consts::DSTORE_3 => (
				input,
				Inst::Local(LocalInst::Store(StackKind::Double, 3 as u16)),
			),
			rvm_consts::FSTORE => map(be_u8, |v| {
				Inst::Local(LocalInst::Store(StackKind::Float, v as u16))
			})(input)?,
			rvm_consts::FSTORE_0 => (
				input,
				Inst::Local(LocalInst::Store(StackKind::Float, 0 as u16)),
			),
			rvm_consts::FSTORE_1 => (
				input,
				Inst::Local(LocalInst::Store(StackKind::Float, 1 as u16)),
			),
			rvm_consts::FSTORE_2 => (
				input,
				Inst::Local(LocalInst::Store(StackKind::Float, 2 as u16)),
			),
			rvm_consts::FSTORE_3 => (
				input,
				Inst::Local(LocalInst::Store(StackKind::Float, 3 as u16)),
			),
			rvm_consts::ISTORE => map(be_u8, |v| {
				Inst::Local(LocalInst::Store(StackKind::Int, v as u16))
			})(input)?,
			rvm_consts::ISTORE_0 => (
				input,
				Inst::Local(LocalInst::Store(StackKind::Int, 0 as u16)),
			),
			rvm_consts::ISTORE_1 => (
				input,
				Inst::Local(LocalInst::Store(StackKind::Int, 1 as u16)),
			),
			rvm_consts::ISTORE_2 => (
				input,
				Inst::Local(LocalInst::Store(StackKind::Int, 2 as u16)),
			),
			rvm_consts::ISTORE_3 => (
				input,
				Inst::Local(LocalInst::Store(StackKind::Int, 3 as u16)),
			),
			rvm_consts::LSTORE => map(be_u8, |v| {
				Inst::Local(LocalInst::Store(StackKind::Long, v as u16))
			})(input)?,
			rvm_consts::LSTORE_0 => (
				input,
				Inst::Local(LocalInst::Store(StackKind::Long, 0 as u16)),
			),
			rvm_consts::LSTORE_1 => (
				input,
				Inst::Local(LocalInst::Store(StackKind::Long, 1 as u16)),
			),
			rvm_consts::LSTORE_2 => (
				input,
				Inst::Local(LocalInst::Store(StackKind::Long, 2 as u16)),
			),
			rvm_consts::LSTORE_3 => (
				input,
				Inst::Local(LocalInst::Store(StackKind::Long, 3 as u16)),
			),
			rvm_consts::IINC => map(tuple((be_u8, be_i8)), |(v0, v1)| {
				Inst::Local(LocalInst::Increment(v1 as i16, v0 as u16))
			})(input)?,
			// ConstantPool Loading
			rvm_consts::LDC => map(be_u8, Inst::LDC)(input)?,
			rvm_consts::LDC_W => map(be_u16, Inst::LDC_W)(input)?,
			rvm_consts::LDC2_W => map(be_u16, Inst::LDC2_W)(input)?,
			// Misc
			rvm_consts::NEW => map(be_cp, |v| Inst::New(NewInst { class: v }))(input)?,
			rvm_consts::ATHROW => (input, Inst::Throw(ThrowInst {})),
			rvm_consts::BIPUSH => map(be_i8, |v| Inst::Push(PushInst(v as i16)))(input)?,
			rvm_consts::SIPUSH => map(be_i16, |v| Inst::Push(PushInst(v as i16)))(input)?,
			rvm_consts::CHECKCAST => {
				map(be_cp, |value| Inst::CheckCast(CheckCastInst { value }))(input)?
			}
			rvm_consts::INSTANCEOF => {
				map(be_cp, |value| Inst::InstanceOf(InstanceOfInst { value }))(input)?
			}
			rvm_consts::GETFIELD => map(be_cp, |value| {
				Inst::Field(FieldInst {
					value,
					instance: true,
					kind: FieldInstKind::Get,
				})
			})(input)?,
			rvm_consts::GETSTATIC => map(be_cp, |value| {
				Inst::Field(FieldInst {
					value,
					instance: false,
					kind: FieldInstKind::Get,
				})
			})(input)?,
			rvm_consts::PUTFIELD => map(be_cp, |value| {
				Inst::Field(FieldInst {
					value,
					instance: true,
					kind: FieldInstKind::Put,
				})
			})(input)?,
			rvm_consts::PUTSTATIC => map(be_cp, |value| {
				Inst::Field(FieldInst {
					value,
					instance: false,
					kind: FieldInstKind::Put,
				})
			})(input)?,
			rvm_consts::INVOKEDYNAMIC => {
				map(tuple((be_cp, be_u16)), |(v, _)| Inst::Invoke(InvokeInst {
					value: v,
					kind: InvokeInstKind::Dynamic
				}))(input)?
			}
			rvm_consts::INVOKEINTERFACE => map(tuple((be_cp, be_u8, be_u8)), |(v, count, _)| {
				Inst::Invoke(InvokeInst {
					value: v,
					kind: InvokeInstKind::Interface(count)
				})
			})(input)?,
			rvm_consts::INVOKESPECIAL => map(be_cp, |v| Inst::Invoke(InvokeInst {
				value: v,
				kind: InvokeInstKind::Special
			}))(input)?,
			rvm_consts::INVOKESTATIC => map(be_cp, |v| Inst::Invoke(InvokeInst {
				value: v,
				kind: InvokeInstKind::Static
			}))(input)?,
			rvm_consts::INVOKEVIRTUAL => map(be_cp, |v| Inst::Invoke(InvokeInst {
				value: v,
				kind: InvokeInstKind::Virtual
			}))(input)?,
			v => {
				panic!("instruction {v}:{} kinda dodo", print_op(v))
			}
		})
	}
}

#[derive(Copy, Clone, Debug)]
pub struct BranchOffset(pub i16);
#[derive(Copy, Clone, Debug)]
pub struct WideBranchOffset(pub i32);
