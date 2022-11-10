use inkwell::values::BasicValue;
use std::fmt::{Display, Formatter};
use rvm_core::{Kind, StackKind};
use rvm_reader::ReturnInst;
use crate::compiler::compiler::BlockCompiler;

use crate::compiler::resolver::BlockResolver;

#[derive(Clone, Debug)]
pub struct ReturnTask {
	returns: Option<StackKind>,
}

impl ReturnTask {
	pub fn resolve(inst: &ReturnInst, _: &mut BlockResolver) -> ReturnTask {
		ReturnTask { returns: inst.value }
	}

	pub fn compile(&self, bc: &mut BlockCompiler) {
		match &self.returns {
			None => {
				bc.build_return(None);
			}
			Some(value) => {
				let value_enum = bc.pop();
				let value = match value.kind() {
					Kind::Boolean => bc
						.build_int_cast(value_enum.into_int_value(), bc.boolean(), "return")
						.as_basic_value_enum(),
					Kind::Byte => bc
						.build_int_cast(value_enum.into_int_value(), bc.i8(), "return")
						.as_basic_value_enum(),
					Kind::Short => bc
						.build_int_cast(value_enum.into_int_value(), bc.short(), "return")
						.as_basic_value_enum(),
					Kind::Int => value_enum,
					Kind::Long => bc
						.build_int_cast(value_enum.into_int_value(), bc.long(), "return")
						.as_basic_value_enum(),
					Kind::Char => bc
						.build_int_cast(value_enum.into_int_value(), bc.char(), "return")
						.as_basic_value_enum(),
					Kind::Float => bc
						.build_float_cast(value_enum.into_float_value(), bc.float(), "return")
						.as_basic_value_enum(),
					Kind::Double => bc
						.build_float_cast(value_enum.into_float_value(), bc.double(), "return")
						.as_basic_value_enum(),
					Kind::Reference => value_enum,
				};
				bc.build_return(Some(&value));
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
