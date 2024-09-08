use crate::thread::{BenFrameMut, ThreadFrame};
use crate::value::StackValue;
use eyre::{bail, Context, ContextCompat};
use num_traits::{Bounded, PrimInt, Signed, WrappingAdd, WrappingMul, WrappingSub, Zero};
use rvm_core::{CastKindError, PrimitiveType};
use rvm_reader::MathInst;
use rvm_runtime::Value;
use std::fmt::{Display, Formatter};

use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Rem, Sub};

#[derive(Debug)]
pub struct CombineTask {
	pub ty: CombineTaskType,
	pub op: CombineTaskOperation,
}

impl Display for CombineTask {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self.op {
			CombineTaskOperation::Add => f.write_str("ADD"),
			CombineTaskOperation::Sub => f.write_str("SUB"),
			CombineTaskOperation::Div => f.write_str("DIV"),
			CombineTaskOperation::Mul => f.write_str("MUL"),
			CombineTaskOperation::Rem => f.write_str("REM"),
			CombineTaskOperation::And => f.write_str("AND"),
			CombineTaskOperation::Or => f.write_str("OR"),
			CombineTaskOperation::Xor => f.write_str("XOR"),
			CombineTaskOperation::Shl => f.write_str("SHL"),
			CombineTaskOperation::Shr => f.write_str("SHR"),
			CombineTaskOperation::UShr => f.write_str("USHR"),
			CombineTaskOperation::FCMPG => f.write_str("FMCPG"),
			CombineTaskOperation::FCMPL => f.write_str("FCMPL"),
			CombineTaskOperation::ICMP => f.write_str("ICMP"),
		}
	}
}

impl CombineTask {
	pub fn new(inst: &MathInst) -> CombineTask {
		let task = CombineTaskType::new(inst.ty());
		let operation = match inst {
			MathInst::Add(_) => CombineTaskOperation::Add,
			MathInst::Sub(_) => CombineTaskOperation::Sub,
			MathInst::Div(_) => CombineTaskOperation::Div,
			MathInst::Mul(_) => CombineTaskOperation::Mul,
			MathInst::Rem(_) => CombineTaskOperation::Rem,
			MathInst::And(_) => CombineTaskOperation::And,
			MathInst::Or(_) => CombineTaskOperation::Or,
			MathInst::Xor(_) => CombineTaskOperation::Xor,
			MathInst::Shl(_) => CombineTaskOperation::Shl,
			MathInst::Shr(_) => CombineTaskOperation::Shr,
			_ => unreachable!(),
		};

		CombineTask {
			ty: task,
			op: operation,
		}
	}

	#[inline(always)]
	pub fn exec(&self, frame: &mut BenFrameMut) -> eyre::Result<()> {
		let v1 = frame.pop();
		let v0 = frame.pop();

		macro_rules! math {
			($V0:ident $V1:ident $METHOD:expr) => {
				math!($V0 $V1 $METHOD, $METHOD)
			};
			($V0:ident $V1:ident $INT_METHOD:expr, $FLOAT_METHOD:expr) => {
				math!($V0 $V1 $INT_METHOD, $INT_METHOD, $FLOAT_METHOD, $FLOAT_METHOD)
			};
			($V0:ident $V1:ident $I32_METHOD:expr, $I64_METHOD:expr, $F32_METHOD:expr, $F64_METHOD:expr) => {
				match self.ty {
					CombineTaskType::Int => {
						let ($V0, $V1) = cast_values::<i32, i32>(v0, v1)?;
						frame.push($I32_METHOD.into());
					}
					CombineTaskType::Long => {
						let ($V0, $V1) = cast_values::<i64, i64>(v0, v1)?;
						frame.push($I64_METHOD.into());
					}
					CombineTaskType::Float => {
						let ($V0, $V1) = cast_values::<f32, f32>(v0, v1)?;
						frame.push($F32_METHOD.into());
					}
					CombineTaskType::Double => {
						let ($V0, $V1) = cast_values::<f64, f64>(v0, v1)?;
						frame.push($F64_METHOD.into());
					}
				}
			};
		}

		macro_rules! int_math {
			($METHOD:path => $TY32:ty:$TY64:ty) => {
				match self.ty {
					CombineTaskType::Int => {
						let (v0, v1) = cast_values::<i32, $TY32>(v0, v1)?;
						frame.push($METHOD(v0, v1).into());
					}
					CombineTaskType::Long => {
						let (v0, v1) = cast_values::<i64, $TY64>(v0, v1)?;
						frame.push($METHOD(v0, v1).into());
					}
					_ => panic!("Integer only operation"),
				}
			};
		}

		match &self.op {
			CombineTaskOperation::Add => {
				math!(v0 v1 WrappingAdd::wrapping_add(&v0, &v1), Add::add(v0, v1));
			}
			CombineTaskOperation::Sub => {
				math!(v0 v1 WrappingSub::wrapping_sub(&v0, &v1), Sub::sub(v0, v1));
			}
			CombineTaskOperation::Div => {
				math!(v0 v1 java_div(v0, v1)?, Div::div(v0, v1));
			}
			CombineTaskOperation::Mul => {
				math!(v0 v1 WrappingMul::wrapping_mul(&v0, &v1), Mul::mul(v0, v1));
			}
			CombineTaskOperation::Rem => {
				// TEST CASE (a/b)*b + (a%b) == a
				math!(v0 v1 java_rem(v0, v1).wrap_err("Arithmetic exception")?);
			}
			CombineTaskOperation::And => int_math!(BitAnd::bitand => i32:i64),
			CombineTaskOperation::Or => int_math!(BitOr::bitor => i32:i64),
			CombineTaskOperation::Xor => int_math!(BitXor::bitxor => i32:i64),
			CombineTaskOperation::Shr => int_math!(java_shr => i32:i32),
			CombineTaskOperation::Shl => int_math!(java_shl => i32:i32),
			CombineTaskOperation::UShr => int_math!(java_ushr => i32:i32),
			task => {
				todo!("{task:?}")
			}
		}
		Ok(())
	}
}

fn java_div<V: Signed + Bounded>(v0: V, v1: V) -> eyre::Result<V> {
	if v1 == V::zero() {
		bail!("Division by 0");
	}
	if v1 == V::one().neg() && v0 == V::min_value() {
		return Ok(v0);
	}

	Ok(V::div(v0, v1))
}

fn java_rem<V: Rem<Output = V> + Zero + PartialEq>(v0: V, v1: V) -> Option<V> {
	if v1 == V::zero() {
		return None;
	}
	Some(v0.rem(v1))
}

fn shift_mask<V: Sized>() -> u32 {
	// FROM JVM SPEC, THIS IS USED TO MASK THE RIGHT HAND SIDE ON BITSHIFTS
	// i32 -> 31 (0x1f)
	// i64 -> 63 (0x3f)
	((size_of::<V>() * 8) - 1) as u32
}

fn java_shl<V: PrimInt>(v0: V, v1: i32) -> V {
	let s = v1 & shift_mask::<V>() as i32;
	V::signed_shl(v0, s as u32)
}

fn java_shr<V: PrimInt>(v0: V, v1: i32) -> V {
	let s = v1 & shift_mask::<V>() as i32;
	V::signed_shr(v0, s as u32)
}

fn java_ushr<V: PrimInt + Signed + WrappingAdd>(v0: V, v1: i32) -> V {
	if v0.is_negative() {
		let s = v1 & shift_mask::<V>() as i32;
		java_shr(v0, s).wrapping_add(&java_shl(V::one() + V::one(), !s))
	} else {
		java_shr(v0, v1)
	}
}

fn cast_values<V0, V1>(v0: StackValue, v1: StackValue) -> eyre::Result<(V0, V1)>
where
	StackValue: TryInto<V0, Error = CastKindError>,
	StackValue: TryInto<V1, Error = CastKindError>,
{
	let v0 = v0.try_into().wrap_err("Failed to convert value1")?;
	let v1 = v1.try_into().wrap_err("Failed to convert value2")?;
	Ok((v0, v1))
}

#[derive(Debug)]
pub enum CombineTaskOperation {
	Add,
	Sub,
	Div,
	Mul,
	Rem,
	And,
	Or,
	Xor,
	Shl,
	Shr,
	UShr,
	// Compare
	FCMPG,
	FCMPL,
	ICMP,
}

#[derive(Debug)]
pub enum CombineTaskType {
	Int,
	Long,
	Float,
	Double,
}

impl CombineTaskType {
	pub fn new(ty: PrimitiveType) -> CombineTaskType {
		match ty {
			PrimitiveType::Int => CombineTaskType::Int,
			PrimitiveType::Long => CombineTaskType::Long,
			PrimitiveType::Float => CombineTaskType::Float,
			PrimitiveType::Double => CombineTaskType::Double,
			_ => panic!("invalid cast"),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn bitshifting_i32() {
		let int_numbers = [0, -1, 1, -31, 31, 32, 63, 64, i32::MAX, i32::MIN];
		let results = [
			(0, 0, 0),
			(0, 0, 0),
			(0, 0, 0),
			(0, 0, 0),
			(0, 0, 0),
			(0, 0, 0),
			(0, 0, 0),
			(0, 0, 0),
			(0, 0, 0),
			(0, 0, 0),
			(-1, -1, -1),
			(-2147483648, -1, 1),
			(-2, -1, 2147483647),
			(-2, -1, 2147483647),
			(-2147483648, -1, 1),
			(-1, -1, -1),
			(-2147483648, -1, 1),
			(-1, -1, -1),
			(-2147483648, -1, 1),
			(-1, -1, -1),
			(1, 1, 1),
			(-2147483648, 0, 0),
			(2, 0, 0),
			(2, 0, 0),
			(-2147483648, 0, 0),
			(1, 1, 1),
			(-2147483648, 0, 0),
			(1, 1, 1),
			(-2147483648, 0, 0),
			(1, 1, 1),
			(-31, -31, -31),
			(-2147483648, -1, 1),
			(-62, -16, 2147483632),
			(-62, -16, 2147483632),
			(-2147483648, -1, 1),
			(-31, -31, -31),
			(-2147483648, -1, 1),
			(-31, -31, -31),
			(-2147483648, -1, 1),
			(-31, -31, -31),
			(31, 31, 31),
			(-2147483648, 0, 0),
			(62, 15, 15),
			(62, 15, 15),
			(-2147483648, 0, 0),
			(31, 31, 31),
			(-2147483648, 0, 0),
			(31, 31, 31),
			(-2147483648, 0, 0),
			(31, 31, 31),
			(32, 32, 32),
			(0, 0, 0),
			(64, 16, 16),
			(64, 16, 16),
			(0, 0, 0),
			(32, 32, 32),
			(0, 0, 0),
			(32, 32, 32),
			(0, 0, 0),
			(32, 32, 32),
			(63, 63, 63),
			(-2147483648, 0, 0),
			(126, 31, 31),
			(126, 31, 31),
			(-2147483648, 0, 0),
			(63, 63, 63),
			(-2147483648, 0, 0),
			(63, 63, 63),
			(-2147483648, 0, 0),
			(63, 63, 63),
			(64, 64, 64),
			(0, 0, 0),
			(128, 32, 32),
			(128, 32, 32),
			(0, 0, 0),
			(64, 64, 64),
			(0, 0, 0),
			(64, 64, 64),
			(0, 0, 0),
			(64, 64, 64),
			(2147483647, 2147483647, 2147483647),
			(-2147483648, 0, 0),
			(-2, 1073741823, 1073741823),
			(-2, 1073741823, 1073741823),
			(-2147483648, 0, 0),
			(2147483647, 2147483647, 2147483647),
			(-2147483648, 0, 0),
			(2147483647, 2147483647, 2147483647),
			(-2147483648, 0, 0),
			(2147483647, 2147483647, 2147483647),
			(-2147483648, -2147483648, -2147483648),
			(0, -1, 1),
			(0, -1073741824, 1073741824),
			(0, -1073741824, 1073741824),
			(0, -1, 1),
			(-2147483648, -2147483648, -2147483648),
			(0, -1, 1),
			(-2147483648, -2147483648, -2147483648),
			(0, -1, 1),
			(-2147483648, -2147483648, -2147483648),
		];

		let mut i = 0;
		for v0 in int_numbers {
			for v1 in int_numbers {
				let (shl, shr, ushr) = results[i];
				assert_eq!(java_shl::<i32>(v0, v1), shl, "shl {v0} << {v1}");
				assert_eq!(java_shr::<i32>(v0, v1), shr, "shr {v0} >> {v1}");
				assert_eq!(java_ushr::<i32>(v0, v1), ushr, "ushr {v0} >>> {v1}");
				i += 1;
			}
		}
	}

	#[test]
	fn bitshifting_i64() {
		let int_numbers = [0, -1, 1, -31, 31, 32, 63, 64, i64::MAX, i64::MIN];
		let results = [
			(0, 0, 0),
			(0, 0, 0),
			(0, 0, 0),
			(0, 0, 0),
			(0, 0, 0),
			(0, 0, 0),
			(0, 0, 0),
			(0, 0, 0),
			(0, 0, 0),
			(0, 0, 0),
			(-1, -1, -1),
			(-9223372036854775808, -1, 1),
			(-2, -1, 9223372036854775807),
			(-8589934592, -1, 2147483647),
			(-2147483648, -1, 8589934591),
			(-4294967296, -1, 4294967295),
			(-9223372036854775808, -1, 1),
			(-1, -1, -1),
			(-9223372036854775808, -1, 1),
			(-1, -1, -1),
			(1, 1, 1),
			(-9223372036854775808, 0, 0),
			(2, 0, 0),
			(8589934592, 0, 0),
			(2147483648, 0, 0),
			(4294967296, 0, 0),
			(-9223372036854775808, 0, 0),
			(1, 1, 1),
			(-9223372036854775808, 0, 0),
			(1, 1, 1),
			(-31, -31, -31),
			(-9223372036854775808, -1, 1),
			(-62, -16, 9223372036854775792),
			(-266287972352, -1, 2147483647),
			(-66571993088, -1, 8589934591),
			(-133143986176, -1, 4294967295),
			(-9223372036854775808, -1, 1),
			(-31, -31, -31),
			(-9223372036854775808, -1, 1),
			(-31, -31, -31),
			(31, 31, 31),
			(-9223372036854775808, 0, 0),
			(62, 15, 15),
			(266287972352, 0, 0),
			(66571993088, 0, 0),
			(133143986176, 0, 0),
			(-9223372036854775808, 0, 0),
			(31, 31, 31),
			(-9223372036854775808, 0, 0),
			(31, 31, 31),
			(32, 32, 32),
			(0, 0, 0),
			(64, 16, 16),
			(274877906944, 0, 0),
			(68719476736, 0, 0),
			(137438953472, 0, 0),
			(0, 0, 0),
			(32, 32, 32),
			(0, 0, 0),
			(32, 32, 32),
			(63, 63, 63),
			(-9223372036854775808, 0, 0),
			(126, 31, 31),
			(541165879296, 0, 0),
			(135291469824, 0, 0),
			(270582939648, 0, 0),
			(-9223372036854775808, 0, 0),
			(63, 63, 63),
			(-9223372036854775808, 0, 0),
			(63, 63, 63),
			(64, 64, 64),
			(0, 0, 0),
			(128, 32, 32),
			(549755813888, 0, 0),
			(137438953472, 0, 0),
			(274877906944, 0, 0),
			(0, 0, 0),
			(64, 64, 64),
			(0, 0, 0),
			(64, 64, 64),
			(
				9223372036854775807,
				9223372036854775807,
				9223372036854775807,
			),
			(-9223372036854775808, 0, 0),
			(-2, 4611686018427387903, 4611686018427387903),
			(-8589934592, 1073741823, 1073741823),
			(-2147483648, 4294967295, 4294967295),
			(-4294967296, 2147483647, 2147483647),
			(-9223372036854775808, 0, 0),
			(
				9223372036854775807,
				9223372036854775807,
				9223372036854775807,
			),
			(-9223372036854775808, 0, 0),
			(
				9223372036854775807,
				9223372036854775807,
				9223372036854775807,
			),
			(
				-9223372036854775808,
				-9223372036854775808,
				-9223372036854775808,
			),
			(0, -1, 1),
			(0, -4611686018427387904, 4611686018427387904),
			(0, -1073741824, 1073741824),
			(0, -4294967296, 4294967296),
			(0, -2147483648, 2147483648),
			(0, -1, 1),
			(
				-9223372036854775808,
				-9223372036854775808,
				-9223372036854775808,
			),
			(0, -1, 1),
			(
				-9223372036854775808,
				-9223372036854775808,
				-9223372036854775808,
			),
		];

		let mut i = 0;
		for v0 in int_numbers {
			for v1 in int_numbers {
				let v1 = v1 as i32;
				let (shl, shr, ushr) = results[i];
				assert_eq!(java_shl::<i64>(v0, v1), shl, "shl {v0} << {v1}");
				assert_eq!(java_shr::<i64>(v0, v1), shr, "shr {v0} >> {v1}");
				assert_eq!(java_ushr::<i64>(v0, v1), ushr, "ushr {v0} >>> {v1}");
				i += 1;
			}
		}
	}
}
