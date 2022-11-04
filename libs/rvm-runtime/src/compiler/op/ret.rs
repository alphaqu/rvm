use crate::compiler::compiler::BlockCompiler;
use inkwell::values::BasicValue;
use std::fmt::{Display, Formatter};

use crate::compiler::resolver::BlockResolver;
use crate::executor::Inst;
use crate::object::ValueType;
use crate::reader::ReturnDescriptor;

#[derive(Clone, Debug)]
pub struct ReturnTask {
	returns: Option<ValueType>,
}

impl ReturnTask {
	pub fn resolve(inst: &Inst, _: &mut BlockResolver) -> ReturnTask {
		let returns = match inst {
			Inst::RETURN => None,
			Inst::ARETURN => Some(ValueType::Reference),
			Inst::DRETURN => Some(ValueType::Double),
			Inst::FRETURN => Some(ValueType::Float),
			Inst::IRETURN => Some(ValueType::Int),
			Inst::LRETURN => Some(ValueType::Long),
			_ => {
				panic!("what")
			}
		};

		ReturnTask { returns }
	}

	pub fn compile(&self, bc: &mut BlockCompiler) {
		match bc.returns().clone() {
			ReturnDescriptor::Field(value) => {
				let value_enum = bc.pop();
				let value = match value.ty() {
					ValueType::Boolean => bc
						.build_int_cast(value_enum.into_int_value(), bc.boolean(), "return")
						.as_basic_value_enum(),
					ValueType::Byte => bc
						.build_int_cast(value_enum.into_int_value(), bc.i8(), "return")
						.as_basic_value_enum(),
					ValueType::Short => bc
						.build_int_cast(value_enum.into_int_value(), bc.short(), "return")
						.as_basic_value_enum(),
					ValueType::Int => value_enum,
					ValueType::Long => bc
						.build_int_cast(value_enum.into_int_value(), bc.long(), "return")
						.as_basic_value_enum(),
					ValueType::Char => bc
						.build_int_cast(value_enum.into_int_value(), bc.char(), "return")
						.as_basic_value_enum(),
					ValueType::Float => bc
						.build_float_cast(value_enum.into_float_value(), bc.float(), "return")
						.as_basic_value_enum(),
					ValueType::Double => bc
						.build_float_cast(value_enum.into_float_value(), bc.double(), "return")
						.as_basic_value_enum(),
					ValueType::Reference => value_enum,
				};
				bc.build_return(Some(&value));
			}
			ReturnDescriptor::Void => {
				bc.build_return(None);
			}
		}
	}
}

impl Display for ReturnTask {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"return {}",
			if let Some(value) = &self.returns {
				format!("{value:?}")
			} else {
				String::new()
			}
		)?;
		Ok(())
	}
}
