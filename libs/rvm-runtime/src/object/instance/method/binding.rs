use std::mem::transmute;
use std::sync::Arc;

use rvm_core::{Kind, MethodDescriptor};

use crate::{AnyValue, Reference, Runtime, Value};

#[derive(Clone)]
pub struct MethodBinding {
	function: extern "C" fn(),
	signature: Option<MethodSignature>,
}

#[derive(Clone)]
pub struct MethodSignature {
	parameters: Vec<Kind>,
	returns: Option<Kind>,
}

impl MethodBinding {
	/// # Safety
	/// Caller must ensure the function follows the signature of decs, else some UB might happen.
	pub unsafe fn new(function: extern "C" fn(), desc: MethodDescriptor) -> MethodBinding {
		MethodBinding {
			function,
			signature: Some(MethodSignature {
				parameters: desc.parameters.iter().map(|v| v.kind()).collect(),
				returns: desc.returns.map(|v| v.kind()),
			}),
		}
	}

	pub fn call(
		&self,
		runtime: &Arc<Runtime>,
		parameters: &[AnyValue],
		returns: Option<Kind>,
	) -> Option<AnyValue> {
		if let Some(signature) = &self.signature {
			assert_eq!(returns, signature.returns);
			assert_eq!(
				signature.parameters.len(),
				parameters.len(),
				"Parameter count missmatch"
			);
			for (i, kind) in signature.parameters.iter().enumerate() {
				assert_eq!(
					parameters[i].kind(),
					*kind,
					"Parameter {i} kind does not match"
				);
			}
		}

		let runtime_ptr = Arc::into_raw(runtime.clone());
		macro_rules! param {
			($($V:ty),*) => {
				unsafe {
					let f = transmute::<_, extern "C" fn(*const Runtime, $($V),*) -> usize>(self.function);
					let mut i = 0usize;
					f(
						runtime_ptr,
						$(
						self.param((&mut i) as &mut $V, parameters),
						)*
					)
				}
			};
		}

		let value = match parameters.len() {
			0 => param!(),
			1 => param!(usize),
			2 => param!(usize, usize),
			3 => param!(usize, usize, usize),
			4 => param!(usize, usize, usize, usize),
			5 => param!(usize, usize, usize, usize, usize),
			6 => param!(usize, usize, usize, usize, usize, usize),
			_ => {
				panic!()
			}
		};

		unsafe {
			// We decrement our strong reference count
			Arc::from_raw(runtime_ptr);
		}

		returns.map(|returns| Self::convert_from(value, returns))
	}

	unsafe fn param(&self, idx: &mut usize, parameters: &[AnyValue]) -> usize {
		let index = *idx;
		*idx += 1;
		let value = parameters[index];
		if let Some(signature) = &self.signature {
			let kind = signature.parameters[index];
			if value.kind() != kind {
				panic!("Missmatched types, {} != {}", value.kind(), kind);
			}
		}
		Self::convert_to(value)
	}
	fn convert_from(value: usize, kind: Kind) -> AnyValue {
		match kind {
			Kind::Byte => AnyValue::Byte(value as _),
			Kind::Short => AnyValue::Short(value as _),
			Kind::Int => AnyValue::Int(value as _),
			Kind::Long => AnyValue::Long(value as _),
			Kind::Char => AnyValue::Char(value as _),
			Kind::Float => AnyValue::Float(value as _),
			Kind::Double => AnyValue::Double(value as _),
			Kind::Boolean => AnyValue::Boolean(value != 0),
			Kind::Reference => AnyValue::Reference(Reference(value as _)),
		}
	}
	fn convert_to(value: AnyValue) -> usize {
		match value {
			AnyValue::Byte(v) => v as usize,
			AnyValue::Short(v) => v as usize,
			AnyValue::Int(v) => v as usize,
			AnyValue::Long(v) => v as usize,
			AnyValue::Char(v) => v as usize,
			AnyValue::Float(v) => v as usize,
			AnyValue::Double(v) => v as usize,
			AnyValue::Boolean(v) => v as usize,
			AnyValue::Reference(v) => v.0 as usize,
		}
	}
}

#[macro_export]
macro_rules! java_binding {
		(fn $NAME:ident($($P_NAME:ident: $P_TY:ty),*) $(->  $RET:ty)? $BLOCK:block) => {
			unsafe {
				extern "C" fn $NAME($($P_NAME: $P_TY),*) $(-> $RET)?
					$BLOCK


				let desc = java_desc!(fn($($P_TY),*) $(-> $RET)?);
				($crate::MethodBinding::new(
					std::mem::transmute($NAME as usize),
					MethodDescriptor::parse(desc).unwrap(),
				), MethodIdentifier {
					name: stringify!($NAME).to_string(),
					descriptor: desc.to_string(),
				})
			}
		};
	}
#[cfg(test)]
mod tests {
	use rvm_macro::java_desc;

	use super::*;

	#[test]
	fn basic() {
		unsafe {
			let binding = java_binding!(|hi: i32, another: i32| -> i32 {
				println!("{hi} {another}");
				69420
			});

			// from
			//fn(hi: i32, another: i32) -> i32 {
			//	println!("{hi} {another_hi}");
			//	69420
			//}

			// to
			//let binding = unsafe {
			//	#[no_mangle]
			//	extern "C" fn binding(hi: i32, another_hi: i32) -> i32 {
			//		println!("{hi} {another_hi}");
			//		69420
			//	}
			//
			//	MethodBinding::new(
			//		transmute(binding as usize),
			//		MethodDesc::parse(java_desc!(fn(i32, i32) -> i32)).unwrap(),
			//	)
			//};

			let i = 0;
			let value = binding
				.call(&[AnyValue::Int(32), AnyValue::Int(420)])
				.unwrap();
			let i2 = 0;
			println!("{i} {i2} {value:?}")
		}
	}
}
