use std::fmt::{Display, Formatter};
use inkwell::IntPredicate;
use crate::compiler::compiler::BlockCompiler;
use crate::compiler::op::{Task};
use crate::compiler::resolver::BlockResolver;
use crate::executor::Inst;

/// Checks a single value against a constant value
#[derive(Clone, Debug)]
pub struct CheckTask {
    pub target: usize,
    pub kind: CheckKind,
}

impl CheckTask {
    pub fn resolve(i: usize, inst: &Inst, resolver: &mut BlockResolver) -> CheckTask {
        let (kind, offset) = match inst {
            Inst::IFEQ(v) => (CheckKind::EqualZero, v),
            Inst::IFNE(v) => (CheckKind::NotEqualZero, v),
            Inst::IFLT(v) => (CheckKind::LessThanZero, v),
            Inst::IFLE(v) => (CheckKind::LessOrEqualZero, v),
            Inst::IFGT(v) => (CheckKind::GreaterThanZero, v),
            Inst::IFGE(v) => (CheckKind::GreaterOrEqualZero, v),
            Inst::IFNONNULL(v) => (CheckKind::NotNull, v),
            Inst::IFNULL(v) => (CheckKind::Null, v),
            _ => {
                panic!("invalid input, inputs needs to be matched")
            }
        };

        CheckTask {
            target: resolver.inst_to_block(i.saturating_add_signed(offset.0 as isize)),
            kind
        }
    }

    pub fn compile(&self, bc: &mut BlockCompiler) {

        let lhs = bc.pop().into_int_value();
        let zero = lhs.get_type().const_int(0, false);
        let then_block = bc.get_block(self.target);
        let else_block = bc.next_block();

        let op = match self.kind {
            CheckKind::EqualZero => {
                IntPredicate::EQ
            }
            CheckKind::NotEqualZero => {
                IntPredicate::NE            }
            CheckKind::LessThanZero => {
                IntPredicate::SLT
            }
            CheckKind::LessOrEqualZero => {
                IntPredicate::SLE
            }
            CheckKind::GreaterThanZero => {
                IntPredicate::SGT
            }
            CheckKind::GreaterOrEqualZero => {
                IntPredicate::SGE
            }
            CheckKind::NotNull => {
                IntPredicate::NE
            }
            CheckKind::Null => {
                IntPredicate::EQ
            }
        };

        let name = bc.gen.next();
        let comparison = bc.build_int_compare(op, lhs, zero, &name);
        bc.build_conditional_branch(comparison, then_block, else_block);

    }
}

impl Display for CheckTask {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let op = match self.kind {
            CheckKind::EqualZero => "== 0",
            CheckKind::NotEqualZero => "!= 0",
            CheckKind::LessThanZero => "< 0",
            CheckKind::LessOrEqualZero => "<= 0",
            CheckKind::GreaterThanZero => "> 0",
            CheckKind::GreaterOrEqualZero => ">= 0",
            CheckKind::NotNull => "!= null",
            CheckKind::Null => "== null",
        };
        write!(f, "if (v0 {op}) then block{}", self.target)
    }
}

#[derive(Clone, Debug)]
pub enum CheckKind {
    EqualZero,
    NotEqualZero,
    LessThanZero,
    LessOrEqualZero,
    GreaterThanZero,
    GreaterOrEqualZero,
    NotNull,
    Null,
}