use std::fmt::{Display, Formatter};
use inkwell::types::BasicTypeEnum;
use inkwell::values::BasicValueEnum;
use crate::compiler::compiler::BlockCompiler;
use crate::compiler::op::{Task};
use crate::compiler::op::variable::{LoadVariableTask, StoreVariableTask};
use crate::compiler::resolver::BlockResolver;
use crate::executor::Inst;
use crate::object::ValueType;


#[derive(Clone, Debug)]
pub enum StackTask {
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
impl StackTask {
    pub fn resolve(inst: &Inst, _: &mut BlockResolver) -> StackTask {
        match inst {
            Inst::DUP => StackTask::DUP,
            Inst::DUP_X1 => StackTask::DUP_X1,
            Inst::DUP_X2 => StackTask::DUP_X2,
            Inst::DUP2 => StackTask::DUP2,
            Inst::DUP2_X1 => StackTask::DUP2_X1,
            Inst::DUP2_X2 => StackTask::DUP2_X2,
            Inst::POP => StackTask::POP,
            Inst::POP2 => StackTask::POP2,
            Inst::SWAP => StackTask::SWAP,
            _ => panic!("what")
        }
    }
        pub fn compile<'b, 'ctx>(&self, bc: &mut BlockCompiler<'b, 'ctx>) {
        match self {
            StackTask::DUP => {
                let value = bc.pop();
                bc.push(value);
                bc.push(value);
            }
            StackTask::DUP_X1 => {
                let value1 = bc.pop();
                let value2 = bc.pop();

               bc.push((value1.clone()));
               bc.push((value2));
               bc.push((value1));
            }
            StackTask::DUP_X2 => {
                let value1 = bc.pop();
                let value2 = bc.pop();


                if !Self::is_category_2(value2.get_type(), bc) {
                    // Form 1
                    let value3 = bc.pop();
                    bc.push((value1.clone()));
                    bc.push((value3));
                    bc.push((value2));
                    bc.push((value1));
                } else {
                    // Form 2
                   bc.push((value1.clone()));
                   bc.push((value2));
                   bc.push((value1));
                }
            }
            StackTask::DUP2 => {
                let value1 = bc.pop();
                if !Self::is_category_2(value1.get_type(), bc) {
                    // Form 1
                    let value2 = bc.pop();
                    bc.push((value2.clone()));
                    bc.push((value1.clone()));
                    bc.push((value2));
                    bc.push((value1));
                } else {
                    // Form 2
                    bc.push((value1.clone()));
                    bc.push((value1));
                }
            }
            StackTask::DUP2_X1 => {
                let value1 = bc.pop();
                let value2 = bc.pop();
                if!Self::is_category_2(value1.get_type(), bc) {
                    // Form 1
                    let value3 = bc.pop();
                    bc.push((value1.clone()));
                    bc.push((value2.clone()));
                    bc.push((value3));
                    bc.push((value1));
                    bc.push((value2));
                } else {
                    // Form 2
                    bc.push((value1.clone()));
                    bc.push((value2));
                    bc.push((value1));
                }
            }
            // this is hell
            StackTask::DUP2_X2 => {
                let value1 = bc.pop();
                let value2 = bc.pop();

                if Self::is_category_2(value1.get_type(), bc) && Self::is_category_2(value2.get_type(), bc) {
                    // Form 4
                   bc.push((value1.clone()));
                   bc.push((value2));
                   bc.push((value1));
                    return;
                }

                let value3 =bc.pop();
                if Self::is_category_2(value3.get_type(), bc) {
                    // Form 3
                    bc.push((value2.clone()));
                    bc.push((value1.clone()));
                    bc.push((value3));
                    bc.push((value2));
                    bc.push((value1));
                    return ;
                }

                if Self::is_category_2(value1.get_type(), bc) {
                    // Form 2
                   bc.push((value1.clone()));
                   bc.push((value3));
                   bc.push((value2));
                   bc.push((value1));
                    return;
                }

                let value4 = bc.pop();
                {
                    // Form 1
                    bc.push((value2.clone()));
                    bc.push((value1.clone()));
                    bc.push((value4));
                    bc.push((value3));
                    bc.push((value2));
                    bc.push((value1));
                }
            }
            StackTask::POP => {
                let value = bc.pop();
                if Self::is_category_2(value.get_type(), bc) {
                    panic!("category 2 not allowed")
                }
            }
            StackTask::POP2 => {
                let value1 = bc.pop();
                if !Self::is_category_2(value1.get_type(), bc) {
                    let value2 = bc.pop();
                    if Self::is_category_2(value2.get_type(), bc) {
                        panic!("category 2 not allowed")
                    }
                }
            }
            StackTask::SWAP => {
                // funny jvm no form business this time
                let value1 = bc.pop();
                let value2 = bc.pop();
               bc.push((value1));
               bc.push((value2));
            }
            _ => {
                panic!("Invalid instruction");
            }
        }
    }

    fn is_category_2(value: BasicTypeEnum, bc: &mut BlockCompiler) -> bool {
        match value {
            BasicTypeEnum::FloatType(ty) => {
                ty == bc.double()
            }
            BasicTypeEnum::IntType(ty) => {
                ty == bc.long()
            }
            _ => false,
        }
    }
}

impl Display for StackTask {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StackTask::DUP => write!(f, "stack DUP"),
            StackTask::DUP_X1 => write!(f, "stack DUP_X1"),
            StackTask::DUP_X2 => write!(f, "stack DUP_X2"),
            StackTask::DUP2 => write!(f, "stack DUP2"),
            StackTask::DUP2_X1 => write!(f, "stack DUP2_X1"),
            StackTask::DUP2_X2 => write!(f, "stack DUP2_X2"),
            StackTask::POP => write!(f, "stack POP"),
            StackTask::POP2 => write!(f, "stack POP2"),
            StackTask::SWAP => write!(f, "stack SWAP"),
        }
    }
}

