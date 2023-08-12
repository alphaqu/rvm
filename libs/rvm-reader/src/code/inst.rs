use nom::combinator::map;
use nom::number::complete::{be_i16, be_i32, be_i8, be_u16, be_u8};
use nom::sequence::tuple;
use std::fmt::{Display, Formatter};
use tracing::trace;

use rvm_core::{Kind, Op, PrimitiveType, StackKind};

use crate::{be_cp, IResult};
use crate::{ClassConst, ConstPtr, FieldConst, MethodConst};

#[derive(Copy, Clone, Debug)]
pub enum ConstInst {
	Null,
	Int(i32),
	Long(i64),
	Float(f32),
	Double(f64),
	Ldc { id: u16, cat2: bool },
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
	pub offset: i32,
	pub kind: JumpKind,
}
impl Display for JumpInst {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?} -> {}", self.kind, self.offset)
	}
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

impl JumpKind {
	pub fn args(&self) -> u32 {
		match self {
			JumpKind::IF_ACMPEQ
			| JumpKind::IF_ACMPNE
			| JumpKind::IF_ICMPEQ
			| JumpKind::IF_ICMPNE
			| JumpKind::IF_ICMPLT
			| JumpKind::IF_ICMPGE
			| JumpKind::IF_ICMPGT
			| JumpKind::IF_ICMPLE => 2,
			JumpKind::IFEQ
			| JumpKind::IFNE
			| JumpKind::IFLT
			| JumpKind::IFGE
			| JumpKind::IFGT
			| JumpKind::IFLE
			| JumpKind::IFNONNULL
			| JumpKind::IFNULL => 1,
			JumpKind::GOTO => 0,
		}
	}
	pub fn is_conditional(&self) -> bool {
		matches!(
			self,
			JumpKind::IF_ACMPEQ
				| JumpKind::IF_ACMPNE
				| JumpKind::IF_ICMPEQ
				| JumpKind::IF_ICMPNE
				| JumpKind::IF_ICMPLT
				| JumpKind::IF_ICMPGE
				| JumpKind::IF_ICMPGT
				| JumpKind::IF_ICMPLE
				| JumpKind::IFEQ | JumpKind::IFNE
				| JumpKind::IFLT | JumpKind::IFGE
				| JumpKind::IFGT | JumpKind::IFLE
				| JumpKind::IFNONNULL
				| JumpKind::IFNULL
		)
	}
}
#[derive(Copy, Clone, Debug)]
pub enum LocalInst {
	Load(StackKind, u16),
	Store(StackKind, u16),
	Increment(i16, u16),
}

#[derive(Copy, Clone, Debug)]
pub struct ReturnInst {
	pub value: Option<StackKind>,
}

#[derive(Copy, Clone, Debug)]
pub struct NewInst {
	pub class: ConstPtr<ClassConst>,
}

#[derive(Copy, Clone, Debug)]
pub struct ThrowInst {
	// TODO
}

#[derive(Copy, Clone, Debug)]
pub struct CheckCastInst {
	pub value: ConstPtr<ClassConst>,
}

#[derive(Copy, Clone, Debug)]
pub struct InstanceOfInst {
	pub value: ConstPtr<ClassConst>,
}

#[derive(Copy, Clone, Debug)]
pub struct FieldInst {
	pub value: ConstPtr<FieldConst>,
	pub instance: bool,
	pub kind: FieldInstKind,
}

#[derive(Copy, Clone, Debug)]
pub enum FieldInstKind {
	Get,
	Put,
}

#[derive(Copy, Clone, Debug)]
pub struct InvokeInst {
	pub value: ConstPtr<MethodConst>,
	pub kind: InvokeInstKind,
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
	CheckCast(CheckCastInst),
	InstanceOf(InstanceOfInst),
	Field(FieldInst),
	Invoke(InvokeInst),
	// Grandpa shit
	JSR(BranchOffset),
	JSR_W(WideBranchOffset),
	RET(u8),
	// TODO read
	LOOKUPSWITCH,
	TABLESWITCH,
	MONITORENTER,
	MONITOREXIT,
}

impl Inst {
	pub fn parse(input: &[u8]) -> IResult<Inst> {
		let (input, value): (_, u8) = be_u8(input)?;
		let value = Op::parse(value);
		trace!("Parsed {value:?}");
		Ok(match (value) {
			Op::NOP => (input, Inst::Nop),
			// Consts
			Op::ACONST_NULL => (input, Inst::Const(ConstInst::Null)),
			Op::DCONST_0 => (input, Inst::Const(ConstInst::Double(0.0))),
			Op::DCONST_1 => (input, Inst::Const(ConstInst::Double(1.0))),
			Op::FCONST_0 => (input, Inst::Const(ConstInst::Float(0.0))),
			Op::FCONST_1 => (input, Inst::Const(ConstInst::Float(1.0))),
			Op::FCONST_2 => (input, Inst::Const(ConstInst::Float(2.0))),
			Op::ICONST_M1 => (input, Inst::Const(ConstInst::Int(-1))),
			Op::ICONST_0 => (input, Inst::Const(ConstInst::Int(0))),
			Op::ICONST_1 => (input, Inst::Const(ConstInst::Int(1))),
			Op::ICONST_2 => (input, Inst::Const(ConstInst::Int(2))),
			Op::ICONST_3 => (input, Inst::Const(ConstInst::Int(3))),
			Op::ICONST_4 => (input, Inst::Const(ConstInst::Int(4))),
			Op::ICONST_5 => (input, Inst::Const(ConstInst::Int(5))),
			Op::LCONST_0 => (input, Inst::Const(ConstInst::Long(0))),
			Op::LCONST_1 => (input, Inst::Const(ConstInst::Long(1))),
			// Stack operations
			Op::DUP => (input, Inst::Stack(StackInst::DUP)),
			Op::DUP_X1 => (input, Inst::Stack(StackInst::DUP_X1)),
			Op::DUP_X2 => (input, Inst::Stack(StackInst::DUP_X2)),
			Op::DUP2 => (input, Inst::Stack(StackInst::DUP2)),
			Op::DUP2_X1 => (input, Inst::Stack(StackInst::DUP2_X1)),
			Op::DUP2_X2 => (input, Inst::Stack(StackInst::DUP2_X2)),
			Op::POP => (input, Inst::Stack(StackInst::POP)),
			Op::POP2 => (input, Inst::Stack(StackInst::POP2)),
			Op::SWAP => (input, Inst::Stack(StackInst::SWAP)),
			// Math
			Op::DADD => (input, Inst::Math(MathInst::Add(PrimitiveType::Double))),
			Op::DDIV => (input, Inst::Math(MathInst::Div(PrimitiveType::Double))),
			Op::DMUL => (input, Inst::Math(MathInst::Mul(PrimitiveType::Double))),
			Op::DNEG => (input, Inst::Math(MathInst::Neg(PrimitiveType::Double))),
			Op::DREM => (input, Inst::Math(MathInst::Rem(PrimitiveType::Double))),
			Op::DSUB => (input, Inst::Math(MathInst::Sub(PrimitiveType::Double))),
			Op::FADD => (input, Inst::Math(MathInst::Add(PrimitiveType::Float))),
			Op::FDIV => (input, Inst::Math(MathInst::Div(PrimitiveType::Float))),
			Op::FMUL => (input, Inst::Math(MathInst::Mul(PrimitiveType::Float))),
			Op::FNEG => (input, Inst::Math(MathInst::Neg(PrimitiveType::Float))),
			Op::FREM => (input, Inst::Math(MathInst::Rem(PrimitiveType::Float))),
			Op::FSUB => (input, Inst::Math(MathInst::Sub(PrimitiveType::Float))),
			Op::IADD => (input, Inst::Math(MathInst::Add(PrimitiveType::Int))),
			Op::IDIV => (input, Inst::Math(MathInst::Div(PrimitiveType::Int))),
			Op::IMUL => (input, Inst::Math(MathInst::Mul(PrimitiveType::Int))),
			Op::INEG => (input, Inst::Math(MathInst::Neg(PrimitiveType::Int))),
			Op::IREM => (input, Inst::Math(MathInst::Rem(PrimitiveType::Int))),
			Op::ISUB => (input, Inst::Math(MathInst::Sub(PrimitiveType::Int))),
			Op::IAND => (input, Inst::Math(MathInst::And(PrimitiveType::Int))),
			Op::IOR => (input, Inst::Math(MathInst::Or(PrimitiveType::Int))),
			Op::ISHL => (input, Inst::Math(MathInst::Shl(PrimitiveType::Int))),
			Op::ISHR => (input, Inst::Math(MathInst::Shr(PrimitiveType::Int))),
			Op::IUSHR => (input, Inst::Math(MathInst::Ushr(PrimitiveType::Int))),
			Op::IXOR => (input, Inst::Math(MathInst::Xor(PrimitiveType::Int))),
			Op::LADD => (input, Inst::Math(MathInst::Add(PrimitiveType::Long))),
			Op::LDIV => (input, Inst::Math(MathInst::Div(PrimitiveType::Long))),
			Op::LMUL => (input, Inst::Math(MathInst::Mul(PrimitiveType::Long))),
			Op::LNEG => (input, Inst::Math(MathInst::Neg(PrimitiveType::Long))),
			Op::LREM => (input, Inst::Math(MathInst::Rem(PrimitiveType::Long))),
			Op::LSUB => (input, Inst::Math(MathInst::Sub(PrimitiveType::Long))),
			Op::LAND => (input, Inst::Math(MathInst::And(PrimitiveType::Long))),
			Op::LOR => (input, Inst::Math(MathInst::Or(PrimitiveType::Long))),
			Op::LSHL => (input, Inst::Math(MathInst::Shl(PrimitiveType::Long))),
			Op::LSHR => (input, Inst::Math(MathInst::Shr(PrimitiveType::Long))),
			Op::LUSHR => (input, Inst::Math(MathInst::Ushr(PrimitiveType::Long))),
			Op::LXOR => (input, Inst::Math(MathInst::Xor(PrimitiveType::Long))),
			// Conversions
			Op::D2F => (input, Inst::Conversion(ConversionInst::D2F)),
			Op::D2I => (input, Inst::Conversion(ConversionInst::D2I)),
			Op::D2L => (input, Inst::Conversion(ConversionInst::D2L)),
			Op::F2D => (input, Inst::Conversion(ConversionInst::F2D)),
			Op::F2I => (input, Inst::Conversion(ConversionInst::F2I)),
			Op::F2L => (input, Inst::Conversion(ConversionInst::F2L)),
			Op::I2B => (input, Inst::Conversion(ConversionInst::I2B)),
			Op::I2C => (input, Inst::Conversion(ConversionInst::I2C)),
			Op::I2D => (input, Inst::Conversion(ConversionInst::I2D)),
			Op::I2F => (input, Inst::Conversion(ConversionInst::I2F)),
			Op::I2L => (input, Inst::Conversion(ConversionInst::I2L)),
			Op::I2S => (input, Inst::Conversion(ConversionInst::I2S)),
			Op::L2D => (input, Inst::Conversion(ConversionInst::L2D)),
			Op::L2F => (input, Inst::Conversion(ConversionInst::L2F)),
			Op::L2I => (input, Inst::Conversion(ConversionInst::L2I)),
			// Comparisons
			Op::DCMPG => (input, Inst::Comparison(ComparisonInst::DCMPG)),
			Op::DCMPL => (input, Inst::Comparison(ComparisonInst::DCMPL)),
			Op::FCMPG => (input, Inst::Comparison(ComparisonInst::FCMPG)),
			Op::FCMPL => (input, Inst::Comparison(ComparisonInst::FCMPL)),
			Op::LCMP => (input, Inst::Comparison(ComparisonInst::LCMP)),
			// Return
			Op::RETURN => (input, Inst::Return(ReturnInst { value: None })),
			Op::ARETURN => (
				input,
				Inst::Return(ReturnInst {
					value: Some(StackKind::Reference),
				}),
			),
			Op::DRETURN => (
				input,
				Inst::Return(ReturnInst {
					value: Some(StackKind::Double),
				}),
			),
			Op::FRETURN => (
				input,
				Inst::Return(ReturnInst {
					value: Some(StackKind::Float),
				}),
			),
			Op::IRETURN => (
				input,
				Inst::Return(ReturnInst {
					value: Some(StackKind::Int),
				}),
			),
			Op::LRETURN => (
				input,
				Inst::Return(ReturnInst {
					value: Some(StackKind::Long),
				}),
			),

			// Array
			Op::AALOAD => (input, Inst::Array(ArrayInst::Load(Kind::Reference))),
			Op::BALOAD => (input, Inst::Array(ArrayInst::Load(Kind::Byte))),
			Op::CALOAD => (input, Inst::Array(ArrayInst::Load(Kind::Char))),
			Op::DALOAD => (input, Inst::Array(ArrayInst::Load(Kind::Double))),
			Op::FALOAD => (input, Inst::Array(ArrayInst::Load(Kind::Float))),
			Op::IALOAD => (input, Inst::Array(ArrayInst::Load(Kind::Int))),
			Op::LALOAD => (input, Inst::Array(ArrayInst::Load(Kind::Long))),
			Op::SALOAD => (input, Inst::Array(ArrayInst::Load(Kind::Short))),
			Op::AASTORE => (input, Inst::Array(ArrayInst::Store(Kind::Reference))),
			Op::BASTORE => (input, Inst::Array(ArrayInst::Store(Kind::Byte))),
			Op::CASTORE => (input, Inst::Array(ArrayInst::Store(Kind::Char))),
			Op::DASTORE => (input, Inst::Array(ArrayInst::Store(Kind::Double))),
			Op::FASTORE => (input, Inst::Array(ArrayInst::Store(Kind::Float))),
			Op::IASTORE => (input, Inst::Array(ArrayInst::Store(Kind::Int))),
			Op::LASTORE => (input, Inst::Array(ArrayInst::Store(Kind::Long))),
			Op::SASTORE => (input, Inst::Array(ArrayInst::Store(Kind::Short))),
			Op::ARRAYLENGTH => (input, Inst::Array(ArrayInst::Length)),
			Op::NEWARRAY => map(be_u8, |v: u8| {
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
			Op::ANEWARRAY => map(be_cp, |v| Inst::Array(ArrayInst::NewRef(v)))(input)?,
			Op::MULTIANEWARRAY => map(tuple((be_cp, be_u8)), |(class, dimensions)| {
				Inst::Array(ArrayInst::NewMultiRef { class, dimensions })
			})(input)?,
			// Jumps
			Op::IF_ACMPEQ => map(be_i16, |v| {
				Inst::Jump(JumpInst {
					offset: v as i32,
					kind: JumpKind::IF_ACMPEQ,
				})
			})(input)?,
			Op::IF_ACMPNE => map(be_i16, |v| {
				Inst::Jump(JumpInst {
					offset: v as i32,
					kind: JumpKind::IF_ACMPNE,
				})
			})(input)?,
			Op::IF_ICMPEQ => map(be_i16, |v| {
				Inst::Jump(JumpInst {
					offset: v as i32,
					kind: JumpKind::IF_ICMPEQ,
				})
			})(input)?,
			Op::IF_ICMPNE => map(be_i16, |v| {
				Inst::Jump(JumpInst {
					offset: v as i32,
					kind: JumpKind::IF_ICMPNE,
				})
			})(input)?,
			Op::IF_ICMPLT => map(be_i16, |v| {
				Inst::Jump(JumpInst {
					offset: v as i32,
					kind: JumpKind::IF_ICMPLT,
				})
			})(input)?,
			Op::IF_ICMPGE => map(be_i16, |v| {
				Inst::Jump(JumpInst {
					offset: v as i32,
					kind: JumpKind::IF_ICMPGE,
				})
			})(input)?,
			Op::IF_ICMPGT => map(be_i16, |v| {
				Inst::Jump(JumpInst {
					offset: v as i32,
					kind: JumpKind::IF_ICMPGT,
				})
			})(input)?,
			Op::IF_ICMPLE => map(be_i16, |v| {
				Inst::Jump(JumpInst {
					offset: v as i32,
					kind: JumpKind::IF_ICMPLE,
				})
			})(input)?,
			Op::IFEQ => map(be_i16, |v| {
				Inst::Jump(JumpInst {
					offset: v as i32,
					kind: JumpKind::IFEQ,
				})
			})(input)?,
			Op::IFNE => map(be_i16, |v| {
				Inst::Jump(JumpInst {
					offset: v as i32,
					kind: JumpKind::IFNE,
				})
			})(input)?,
			Op::IFLT => map(be_i16, |v| {
				Inst::Jump(JumpInst {
					offset: v as i32,
					kind: JumpKind::IFLT,
				})
			})(input)?,
			Op::IFGE => map(be_i16, |v| {
				Inst::Jump(JumpInst {
					offset: v as i32,
					kind: JumpKind::IFGE,
				})
			})(input)?,
			Op::IFGT => map(be_i16, |v| {
				Inst::Jump(JumpInst {
					offset: v as i32,
					kind: JumpKind::IFGT,
				})
			})(input)?,
			Op::IFLE => map(be_i16, |v| {
				Inst::Jump(JumpInst {
					offset: v as i32,
					kind: JumpKind::IFLE,
				})
			})(input)?,
			Op::IFNONNULL => map(be_i16, |v| {
				Inst::Jump(JumpInst {
					offset: v as i32,
					kind: JumpKind::IFNONNULL,
				})
			})(input)?,
			Op::IFNULL => map(be_i16, |v| {
				Inst::Jump(JumpInst {
					offset: v as i32,
					kind: JumpKind::IFNULL,
				})
			})(input)?,
			Op::GOTO => map(be_i16, |v| {
				Inst::Jump(JumpInst {
					offset: v as i32,
					kind: JumpKind::GOTO,
				})
			})(input)?,
			Op::GOTO_W => map(be_i32, |v| {
				Inst::Jump(JumpInst {
					offset: v as i32,
					kind: JumpKind::GOTO,
				})
			})(input)?,
			// Locals
			Op::ALOAD => map(be_u8, |v| {
				Inst::Local(LocalInst::Load(StackKind::Reference, v as u16))
			})(input)?,
			Op::ALOAD_0 => (
				input,
				Inst::Local(LocalInst::Load(StackKind::Reference, 0 as u16)),
			),
			Op::ALOAD_1 => (
				input,
				Inst::Local(LocalInst::Load(StackKind::Reference, 1 as u16)),
			),
			Op::ALOAD_2 => (
				input,
				Inst::Local(LocalInst::Load(StackKind::Reference, 2 as u16)),
			),
			Op::ALOAD_3 => (
				input,
				Inst::Local(LocalInst::Load(StackKind::Reference, 3 as u16)),
			),
			Op::DLOAD => map(be_u8, |v| {
				Inst::Local(LocalInst::Load(StackKind::Double, v as u16))
			})(input)?,
			Op::DLOAD_0 => (
				input,
				Inst::Local(LocalInst::Load(StackKind::Double, 0 as u16)),
			),
			Op::DLOAD_1 => (
				input,
				Inst::Local(LocalInst::Load(StackKind::Double, 1 as u16)),
			),
			Op::DLOAD_2 => (
				input,
				Inst::Local(LocalInst::Load(StackKind::Double, 2 as u16)),
			),
			Op::DLOAD_3 => (
				input,
				Inst::Local(LocalInst::Load(StackKind::Double, 3 as u16)),
			),
			Op::FLOAD => map(be_u8, |v| {
				Inst::Local(LocalInst::Load(StackKind::Float, v as u16))
			})(input)?,
			Op::FLOAD_0 => (
				input,
				Inst::Local(LocalInst::Load(StackKind::Float, 0 as u16)),
			),
			Op::FLOAD_1 => (
				input,
				Inst::Local(LocalInst::Load(StackKind::Float, 1 as u16)),
			),
			Op::FLOAD_2 => (
				input,
				Inst::Local(LocalInst::Load(StackKind::Float, 2 as u16)),
			),
			Op::FLOAD_3 => (
				input,
				Inst::Local(LocalInst::Load(StackKind::Float, 3 as u16)),
			),
			Op::ILOAD => map(be_u8, |v| {
				Inst::Local(LocalInst::Load(StackKind::Int, v as u16))
			})(input)?,
			Op::ILOAD_0 => (
				input,
				Inst::Local(LocalInst::Load(StackKind::Int, 0 as u16)),
			),
			Op::ILOAD_1 => (
				input,
				Inst::Local(LocalInst::Load(StackKind::Int, 1 as u16)),
			),
			Op::ILOAD_2 => (
				input,
				Inst::Local(LocalInst::Load(StackKind::Int, 2 as u16)),
			),
			Op::ILOAD_3 => (
				input,
				Inst::Local(LocalInst::Load(StackKind::Int, 3 as u16)),
			),
			Op::LLOAD => map(be_u8, |v| {
				Inst::Local(LocalInst::Load(StackKind::Long, v as u16))
			})(input)?,
			Op::LLOAD_0 => (
				input,
				Inst::Local(LocalInst::Load(StackKind::Long, 0 as u16)),
			),
			Op::LLOAD_1 => (
				input,
				Inst::Local(LocalInst::Load(StackKind::Long, 1 as u16)),
			),
			Op::LLOAD_2 => (
				input,
				Inst::Local(LocalInst::Load(StackKind::Long, 2 as u16)),
			),
			Op::LLOAD_3 => (
				input,
				Inst::Local(LocalInst::Load(StackKind::Long, 3 as u16)),
			),
			Op::ASTORE => map(be_u8, |v| {
				Inst::Local(LocalInst::Store(StackKind::Reference, v as u16))
			})(input)?,
			Op::ASTORE_0 => (
				input,
				Inst::Local(LocalInst::Store(StackKind::Reference, 0 as u16)),
			),
			Op::ASTORE_1 => (
				input,
				Inst::Local(LocalInst::Store(StackKind::Reference, 1 as u16)),
			),
			Op::ASTORE_2 => (
				input,
				Inst::Local(LocalInst::Store(StackKind::Reference, 2 as u16)),
			),
			Op::ASTORE_3 => (
				input,
				Inst::Local(LocalInst::Store(StackKind::Reference, 3 as u16)),
			),
			Op::DSTORE => map(be_u8, |v| {
				Inst::Local(LocalInst::Store(StackKind::Double, v as u16))
			})(input)?,
			Op::DSTORE_0 => (
				input,
				Inst::Local(LocalInst::Store(StackKind::Double, 0 as u16)),
			),
			Op::DSTORE_1 => (
				input,
				Inst::Local(LocalInst::Store(StackKind::Double, 1 as u16)),
			),
			Op::DSTORE_2 => (
				input,
				Inst::Local(LocalInst::Store(StackKind::Double, 2 as u16)),
			),
			Op::DSTORE_3 => (
				input,
				Inst::Local(LocalInst::Store(StackKind::Double, 3 as u16)),
			),
			Op::FSTORE => map(be_u8, |v| {
				Inst::Local(LocalInst::Store(StackKind::Float, v as u16))
			})(input)?,
			Op::FSTORE_0 => (
				input,
				Inst::Local(LocalInst::Store(StackKind::Float, 0 as u16)),
			),
			Op::FSTORE_1 => (
				input,
				Inst::Local(LocalInst::Store(StackKind::Float, 1 as u16)),
			),
			Op::FSTORE_2 => (
				input,
				Inst::Local(LocalInst::Store(StackKind::Float, 2 as u16)),
			),
			Op::FSTORE_3 => (
				input,
				Inst::Local(LocalInst::Store(StackKind::Float, 3 as u16)),
			),
			Op::ISTORE => map(be_u8, |v| {
				Inst::Local(LocalInst::Store(StackKind::Int, v as u16))
			})(input)?,
			Op::ISTORE_0 => (
				input,
				Inst::Local(LocalInst::Store(StackKind::Int, 0 as u16)),
			),
			Op::ISTORE_1 => (
				input,
				Inst::Local(LocalInst::Store(StackKind::Int, 1 as u16)),
			),
			Op::ISTORE_2 => (
				input,
				Inst::Local(LocalInst::Store(StackKind::Int, 2 as u16)),
			),
			Op::ISTORE_3 => (
				input,
				Inst::Local(LocalInst::Store(StackKind::Int, 3 as u16)),
			),
			Op::LSTORE => map(be_u8, |v| {
				Inst::Local(LocalInst::Store(StackKind::Long, v as u16))
			})(input)?,
			Op::LSTORE_0 => (
				input,
				Inst::Local(LocalInst::Store(StackKind::Long, 0 as u16)),
			),
			Op::LSTORE_1 => (
				input,
				Inst::Local(LocalInst::Store(StackKind::Long, 1 as u16)),
			),
			Op::LSTORE_2 => (
				input,
				Inst::Local(LocalInst::Store(StackKind::Long, 2 as u16)),
			),
			Op::LSTORE_3 => (
				input,
				Inst::Local(LocalInst::Store(StackKind::Long, 3 as u16)),
			),
			Op::IINC => map(tuple((be_u8, be_i8)), |(v0, v1)| {
				Inst::Local(LocalInst::Increment(v1 as i16, v0 as u16))
			})(input)?,
			// ConstantPool Loading
			Op::LDC => map(be_u8, |v| {
				Inst::Const(ConstInst::Ldc {
					id: v as u16,
					cat2: false,
				})
			})(input)?,
			Op::LDC_W => map(be_u16, |v| {
				Inst::Const(ConstInst::Ldc {
					id: v as u16,
					cat2: false,
				})
			})(input)?,
			Op::LDC2_W => map(be_u16, |v| {
				Inst::Const(ConstInst::Ldc {
					id: v as u16,
					cat2: true,
				})
			})(input)?,
			// Misc
			Op::NEW => map(be_cp, |v| Inst::New(NewInst { class: v }))(input)?,
			Op::ATHROW => (input, Inst::Throw(ThrowInst {})),
			Op::BIPUSH => map(be_i8, |v| Inst::Const(ConstInst::Int(v as i32)))(input)?,
			Op::SIPUSH => map(be_i16, |v| Inst::Const(ConstInst::Int(v as i32)))(input)?,
			Op::CHECKCAST => map(be_cp, |value| Inst::CheckCast(CheckCastInst { value }))(input)?,
			Op::INSTANCEOF => {
				map(be_cp, |value| Inst::InstanceOf(InstanceOfInst { value }))(input)?
			}
			Op::GETFIELD => map(be_cp, |value| {
				Inst::Field(FieldInst {
					value,
					instance: true,
					kind: FieldInstKind::Get,
				})
			})(input)?,
			Op::GETSTATIC => map(be_cp, |value| {
				Inst::Field(FieldInst {
					value,
					instance: false,
					kind: FieldInstKind::Get,
				})
			})(input)?,
			Op::PUTFIELD => map(be_cp, |value| {
				Inst::Field(FieldInst {
					value,
					instance: true,
					kind: FieldInstKind::Put,
				})
			})(input)?,
			Op::PUTSTATIC => map(be_cp, |value| {
				Inst::Field(FieldInst {
					value,
					instance: false,
					kind: FieldInstKind::Put,
				})
			})(input)?,
			Op::INVOKEDYNAMIC => map(tuple((be_cp, be_u16)), |(v, _)| {
				Inst::Invoke(InvokeInst {
					value: v,
					kind: InvokeInstKind::Dynamic,
				})
			})(input)?,
			Op::INVOKEINTERFACE => map(tuple((be_cp, be_u8, be_u8)), |(v, count, _)| {
				Inst::Invoke(InvokeInst {
					value: v,
					kind: InvokeInstKind::Interface(count),
				})
			})(input)?,
			Op::INVOKESPECIAL => map(be_cp, |v| {
				Inst::Invoke(InvokeInst {
					value: v,
					kind: InvokeInstKind::Special,
				})
			})(input)?,
			Op::INVOKESTATIC => map(be_cp, |v| {
				Inst::Invoke(InvokeInst {
					value: v,
					kind: InvokeInstKind::Static,
				})
			})(input)?,
			Op::INVOKEVIRTUAL => map(be_cp, |v| {
				Inst::Invoke(InvokeInst {
					value: v,
					kind: InvokeInstKind::Virtual,
				})
			})(input)?,
			v => {
				panic!("instruction {v:?} kinda dodo");
			}
		})
	}
}

#[derive(Copy, Clone, Debug)]
pub struct BranchOffset(pub i16);
#[derive(Copy, Clone, Debug)]
pub struct WideBranchOffset(pub i32);
