pub mod apply;
pub mod check;
pub mod combine;
pub mod compare;
pub mod constant;
pub mod conversion;
pub mod invoke;
pub mod jump;
pub mod ret;
pub mod stack;
pub mod variable;

use crate::compiler::BlockCompiler;
use crate::op::apply::ApplyTask;
use crate::op::check::CheckTask;
use crate::op::compare::CompareTask;
use crate::op::constant::ConstTask;
use crate::op::conversion::ConversionTask;
use crate::op::invoke::InvokeTask;
use crate::op::jump::JumpTask;
use crate::op::ret::ReturnTask;
use crate::op::stack::StackTask;
use crate::op::variable::{IncrementTask, LoadVariableTask, StoreVariableTask};
use crate::resolver::BlockResolver;
use combine::CombineTask;
use std::fmt::{Display, Formatter};
use rvm_reader::Inst;

#[derive(Clone, Debug)]
pub enum Task {
	Nop,
	Apply(ApplyTask),
	Combine(CombineTask),
	Const(ConstTask),
	Stack(StackTask),
	Conversion(ConversionTask),
	Compare(CompareTask),
	Check(CheckTask),
	Jump(JumpTask),
	LoadVariable(LoadVariableTask),
	StoreVariable(StoreVariableTask),
	Increase(IncrementTask),
	Return(ReturnTask),
	Invoke(InvokeTask),
}

impl Task {
	pub fn resolve(i: usize, inst: &Inst, resolver: &mut BlockResolver) -> Task {
		match inst {
			Inst::NOP => Task::Nop,
			// Apply
			Inst::FNEG | Inst::DNEG | Inst::INEG | Inst::LNEG => {
				Task::Apply(ApplyTask::resolve(inst, resolver))
			}
			// Combine
			Inst::DADD
			| Inst::DDIV
			| Inst::DMUL
			| Inst::DREM
			| Inst::DSUB
			| Inst::FADD
			| Inst::FDIV
			| Inst::FMUL
			| Inst::FREM
			| Inst::FSUB
			| Inst::IADD
			| Inst::IDIV
			| Inst::IMUL
			| Inst::IREM
			| Inst::ISUB
			| Inst::LADD
			| Inst::LDIV
			| Inst::LMUL
			| Inst::LREM
			| Inst::LSUB
			| Inst::IAND
			| Inst::IOR
			| Inst::ISHL
			| Inst::ISHR
			| Inst::IUSHR
			| Inst::IXOR
			| Inst::LAND
			| Inst::LOR
			| Inst::LSHL
			| Inst::LSHR
			| Inst::LUSHR
			| Inst::LXOR
			| Inst::FCMPG
			| Inst::DCMPG
			| Inst::LCMP
			| Inst::FCMPL
			| Inst::DCMPL => Task::Combine(CombineTask::resolve(inst, resolver)),
			// Const
			Inst::ACONST_NULL
			| Inst::DCONST_0
			| Inst::DCONST_1
			| Inst::FCONST_0
			| Inst::FCONST_1
			| Inst::FCONST_2
			| Inst::ICONST_M1
			| Inst::ICONST_0
			| Inst::ICONST_1
			| Inst::ICONST_2
			| Inst::ICONST_3
			| Inst::ICONST_4
			| Inst::ICONST_5
			| Inst::LCONST_0
			| Inst::LCONST_1
			| Inst::BIPUSH(_)
			| Inst::SIPUSH(_)
			| Inst::LDC(_)
			| Inst::LDC_W(_)
			| Inst::LDC2_W(_) => Task::Const(ConstTask::resolve(inst, resolver)),
			// Stack
			Inst::DUP
			| Inst::DUP_X1
			| Inst::DUP_X2
			| Inst::DUP2
			| Inst::DUP2_X1
			| Inst::DUP2_X2
			| Inst::POP
			| Inst::POP2
			| Inst::SWAP => Task::Stack(StackTask::resolve(inst, resolver)),
			// Array
			Inst::NEWARRAY(_)
			| Inst::AALOAD
			| Inst::AASTORE
			| Inst::BALOAD
			| Inst::BASTORE
			| Inst::CALOAD
			| Inst::CASTORE
			| Inst::DALOAD
			| Inst::DASTORE
			| Inst::FALOAD
			| Inst::FASTORE
			| Inst::IALOAD
			| Inst::IASTORE
			| Inst::LALOAD
			| Inst::LASTORE
			| Inst::SALOAD
			| Inst::SASTORE
			| Inst::ARRAYLENGTH
			| Inst::ANEWARRAY(_)
			| Inst::MULTIANEWARRAY { .. } => todo!("array compilation"),
			// Conversion
			Inst::D2F
			| Inst::D2I
			| Inst::D2L
			| Inst::F2D
			| Inst::F2I
			| Inst::F2L
			| Inst::I2B
			| Inst::I2C
			| Inst::I2D
			| Inst::I2F
			| Inst::I2L
			| Inst::I2S
			| Inst::L2D
			| Inst::L2F
			| Inst::L2I => Task::Conversion(ConversionTask::resolve(inst, resolver)),
			// Compare
			Inst::IF_ACMPEQ(_)
			| Inst::IF_ACMPNE(_)
			| Inst::IF_ICMPEQ(_)
			| Inst::IF_ICMPNE(_)
			| Inst::IF_ICMPLT(_)
			| Inst::IF_ICMPGE(_)
			| Inst::IF_ICMPGT(_)
			| Inst::IF_ICMPLE(_) => Task::Compare(CompareTask::resolve(i, inst, resolver)),
			// Check
			Inst::IFEQ(_)
			| Inst::IFNE(_)
			| Inst::IFLT(_)
			| Inst::IFGE(_)
			| Inst::IFGT(_)
			| Inst::IFLE(_)
			| Inst::IFNONNULL(_)
			| Inst::IFNULL(_) => Task::Check(CheckTask::resolve(i, inst, resolver)),
			// Jump
			Inst::GOTO(_) | Inst::GOTO_W(_) => Task::Jump(JumpTask::resolve(i, inst, resolver)),
			// LoadVar
			Inst::ALOAD(_)
			| Inst::ALOAD_W(_)
			| Inst::ALOAD0
			| Inst::ALOAD1
			| Inst::ALOAD2
			| Inst::ALOAD3
			| Inst::DLOAD(_)
			| Inst::DLOAD_W(_)
			| Inst::DLOAD0
			| Inst::DLOAD1
			| Inst::DLOAD2
			| Inst::DLOAD3
			| Inst::FLOAD(_)
			| Inst::FLOAD_W(_)
			| Inst::FLOAD0
			| Inst::FLOAD1
			| Inst::FLOAD2
			| Inst::FLOAD3
			| Inst::ILOAD(_)
			| Inst::ILOAD_W(_)
			| Inst::ILOAD0
			| Inst::ILOAD1
			| Inst::ILOAD2
			| Inst::ILOAD3
			| Inst::LLOAD(_)
			| Inst::LLOAD_W(_)
			| Inst::LLOAD0
			| Inst::LLOAD1
			| Inst::LLOAD2
			| Inst::LLOAD3 => Task::LoadVariable(LoadVariableTask::resolve(inst, resolver)),
			Inst::ASTORE(_)
			| Inst::ASTORE_W(_)
			| Inst::ASTORE0
			| Inst::ASTORE1
			| Inst::ASTORE2
			| Inst::ASTORE3
			| Inst::DSTORE(_)
			| Inst::DSTORE_W(_)
			| Inst::DSTORE0
			| Inst::DSTORE1
			| Inst::DSTORE2
			| Inst::DSTORE3
			| Inst::FSTORE(_)
			| Inst::FSTORE_W(_)
			| Inst::FSTORE0
			| Inst::FSTORE1
			| Inst::FSTORE2
			| Inst::FSTORE3
			| Inst::ISTORE(_)
			| Inst::ISTORE_W(_)
			| Inst::ISTORE0
			| Inst::ISTORE1
			| Inst::ISTORE2
			| Inst::ISTORE3
			| Inst::LSTORE(_)
			| Inst::LSTORE_W(_)
			| Inst::LSTORE0
			| Inst::LSTORE1
			| Inst::LSTORE2
			| Inst::LSTORE3 => Task::StoreVariable(StoreVariableTask::resolve(inst, resolver)),
			Inst::IINC(_, _) | Inst::IINC_W(_, _) => {
				Task::Increase(IncrementTask::resolve(inst, resolver))
			}
			// Return
			Inst::RETURN
			| Inst::ARETURN
			| Inst::DRETURN
			| Inst::FRETURN
			| Inst::IRETURN
			| Inst::LRETURN => Task::Return(ReturnTask::resolve(inst, resolver)),
			// grandpa shit
			Inst::JSR(_) => todo!("grandpa shit"),
			Inst::JSR_W(_) => todo!("grandpa shit"),
			Inst::RET(_) => todo!("grandpa shit"),
			Inst::ATHROW => {
				todo!("throw")
			}
			Inst::CHECKCAST(class) => {
				todo!("checkcast")
			}
			Inst::INSTANCEOF(_) => {
				todo!("instanceof")
			}
			// alpha reading challange any%
			Inst::LOOKUPSWITCH => todo!("read"),
			Inst::TABLESWITCH => todo!("read"),
			Inst::MONITORENTER => todo!("read"),
			Inst::MONITOREXIT => todo!("read"),

			Inst::NEW(_) => todo!("NEW"),
			Inst::GETFIELD(_) => todo!("GETFIELD"),
			Inst::GETSTATIC(_) => todo!("GETSTATIC"),
			Inst::PUTFIELD(_) => todo!("PUTFIELD"),
			Inst::PUTSTATIC(_) => todo!("PUTSTATIC"),
			Inst::INVOKEDYNAMIC(_)
			| Inst::INVOKEINTERFACE(_, _)
			| Inst::INVOKESPECIAL(_)
			| Inst::INVOKESTATIC(_)
			| Inst::INVOKEVIRTUAL(_) => Task::Invoke(InvokeTask::resolve(inst, resolver)),
		}
	}

	pub fn compile<'b, 'a>(&self, bc: &mut BlockCompiler<'b, 'a>) {
		match self {
			Task::Nop => {
				// pray
				panic!("nop instruction is intended to be temporary. shit blew up")
			}
			Task::Apply(v) => v.compile(bc),
			Task::Combine(v) => v.compile(bc),
			Task::Const(v) => v.compile(bc),
			Task::Conversion(v) => v.compile(bc),
			Task::LoadVariable(v) => v.compile(bc),
			Task::Invoke(v) => v.compile(bc),
			Task::Stack(v) => v.compile(bc),
			Task::Compare(v) => v.compile(bc),
			Task::Check(v) => v.compile(bc),
			Task::Jump(v) => v.compile(bc),
			Task::StoreVariable(v) => v.compile(bc),
			Task::Increase(v) => v.compile(bc),
			Task::Return(v) => v.compile(bc),
		}
	}
}

impl Display for Task {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Task::Nop => {
				write!(f, "nop")
			}
			Task::Apply(v) => v.fmt(f),
			Task::Combine(v) => v.fmt(f),
			Task::Const(v) => v.fmt(f),
			Task::Stack(v) => v.fmt(f),
			Task::Conversion(v) => v.fmt(f),
			Task::Compare(v) => v.fmt(f),
			Task::Check(v) => v.fmt(f),
			Task::Jump(v) => v.fmt(f),
			Task::LoadVariable(v) => v.fmt(f),
			Task::StoreVariable(v) => v.fmt(f),
			Task::Increase(v) => v.fmt(f),
			Task::Return(v) => v.fmt(f),
			Task::Invoke(v) => v.fmt(f),
		}
	}
}
