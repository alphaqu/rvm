use std::fmt::{Display, Formatter};

use inkwell::values::BasicValue;

use rvm_core::Kind;
use rvm_reader::ConversionInst;

use crate::compiler::BlockCompiler;
use crate::resolver::BlockResolver;

#[derive(Clone, Debug)]
pub struct ConversionTask {
	kind: ConversionKind,
}

impl ConversionTask {
	pub fn resolve(inst: &ConversionInst, _: &mut BlockResolver) -> ConversionTask {
		let kind = match inst {
			ConversionInst::D2F => ConversionKind::D2F,
			ConversionInst::D2I => ConversionKind::D2I,
			ConversionInst::D2L => ConversionKind::D2L,
			ConversionInst::F2D => ConversionKind::F2D,
			ConversionInst::F2I => ConversionKind::F2I,
			ConversionInst::F2L => ConversionKind::F2L,
			ConversionInst::I2B => ConversionKind::I2B,
			ConversionInst::I2C => ConversionKind::I2C,
			ConversionInst::I2D => ConversionKind::I2D,
			ConversionInst::I2F => ConversionKind::I2F,
			ConversionInst::I2L => ConversionKind::I2L,
			ConversionInst::I2S => ConversionKind::I2S,
			ConversionInst::L2D => ConversionKind::L2D,
			ConversionInst::L2F => ConversionKind::L2F,
			ConversionInst::L2I => ConversionKind::L2I,
			_ => panic!("what"),
		};

		ConversionTask {
			kind,
		}
	}

	pub fn compile<'b, 'a>(&self, bc: &mut BlockCompiler<'b, 'a>) {
		let name = bc.gen.next();
		let name2 = bc.gen.next();

		let value = bc.pop();
		let output = match self.kind {
			ConversionKind::D2F => bc
				.build_float_cast(value.into_float_value(), bc.float(), &name)
				.as_basic_value_enum(),
			ConversionKind::D2I => bc
				.build_float_to_signed_int(value.into_float_value(), bc.int(), &name)
				.as_basic_value_enum(),
			ConversionKind::D2L => bc
				.build_float_to_signed_int(value.into_float_value(), bc.long(), &name)
				.as_basic_value_enum(),
			ConversionKind::F2D => bc
				.build_float_cast(value.into_float_value(), bc.double(), &name)
				.as_basic_value_enum(),
			ConversionKind::F2I => bc
				.build_float_to_signed_int(value.into_float_value(), bc.int(), &name)
				.as_basic_value_enum(),
			ConversionKind::F2L => bc
				.build_float_to_signed_int(value.into_float_value(), bc.long(), &name)
				.as_basic_value_enum(),
			ConversionKind::I2B => bc
				.build_int_cast(
					bc.build_int_cast(value.into_int_value(), bc.i8(), &name),
					bc.int(),
					&name2,
				)
				.as_basic_value_enum(),
			ConversionKind::I2C => bc
				.build_int_cast(
					bc.build_int_cast(value.into_int_value(), bc.char(), &name),
					bc.int(),
					&name2,
				)
				.as_basic_value_enum(),
			ConversionKind::I2D => bc
				.build_signed_int_to_float(value.into_int_value(), bc.double(), &name)
				.as_basic_value_enum(),
			ConversionKind::I2F => bc
				.build_signed_int_to_float(value.into_int_value(), bc.float(), &name)
				.as_basic_value_enum(),
			ConversionKind::I2L => bc
				.build_int_cast(value.into_int_value(), bc.long(), &name)
				.as_basic_value_enum(),
			ConversionKind::I2S => bc
				.build_int_cast(
					bc.build_int_cast(value.into_int_value(), bc.short(), &name),
					bc.int(),
					&name2,
				)
				.as_basic_value_enum(),
			ConversionKind::L2D => bc
				.build_signed_int_to_float(value.into_int_value(), bc.double(), &name)
				.as_basic_value_enum(),
			ConversionKind::L2F => bc
				.build_signed_int_to_float(value.into_int_value(), bc.float(), &name)
				.as_basic_value_enum(),
			ConversionKind::L2I => bc
				.build_int_cast(value.into_int_value(), bc.int(), &name)
				.as_basic_value_enum(),
		};
		bc.push(output);
	}

	pub fn get_type(&self) -> Kind {
		match self.kind {
			ConversionKind::F2I | ConversionKind::L2I | ConversionKind::D2I => Kind::Int,
			ConversionKind::D2F | ConversionKind::I2F | ConversionKind::L2F => Kind::Float,
			ConversionKind::D2L | ConversionKind::F2L | ConversionKind::I2L => Kind::Long,
			ConversionKind::F2D | ConversionKind::I2D | ConversionKind::L2D => Kind::Double,
			ConversionKind::I2B => Kind::Int,
			ConversionKind::I2C => Kind::Int,
			ConversionKind::I2S => Kind::Int,
		}
	}
}

impl Display for ConversionTask {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let ty = self.get_type();
		write!(f, "convert {ty}")
	}
}

#[derive(Clone, Debug)]
pub enum ConversionKind {
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
