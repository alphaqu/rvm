use crate::{AnyValue, Reference, Runtime};
use rvm_core::Kind;
use std::mem::transmute;
use std::sync::Arc;

pub struct JNIFunction {
	function: extern "C" fn(),
	signature: Option<JNIFunctionSignature>,
}

#[derive(Clone)]
pub struct JNIFunctionSignature {
	pub parameters: Vec<Kind>,
	pub returns: Option<Kind>,
}

impl JNIFunction {
	/// # Safety
	/// Caller must ensure the function follows the signature of decs, else some UB might happen.
	pub unsafe fn new(function: extern "C" fn(), desc: JNIFunctionSignature) -> Self {
		Self {
			function,
			signature: Some(desc),
		}
	}

	pub fn call(
		&self,
		runtime: &Runtime,
		parameters: &[AnyValue],
		returns: Option<Kind>,
	) -> Option<AnyValue> {
		todo!("JNIEnv is blocking")
		//if let Some(signature) = &self.signature {
		// 			assert_eq!(returns, signature.returns);
		// 			assert_eq!(
		// 				signature.parameters.len(),
		// 				parameters.len(),
		// 				"Parameter count missmatch"
		// 			);
		// 			for (i, kind) in signature.parameters.iter().enumerate() {
		// 				assert_eq!(
		// 					parameters[i].kind(),
		// 					*kind,
		// 					"Parameter {i} kind does not match"
		// 				);
		// 			}
		// 		}
		//
		// 		let runtime_ptr = Arc::into_raw(runtime.clone());
		// 		macro_rules! param {
		// 			($($V:ty),*) => {
		// 				unsafe {
		// 					let f = transmute::<extern "C" fn(), extern "C" fn(*const Runtime, $($V),*) -> usize>(self.function);
		// 					let mut i = 0usize;
		// 					f(
		// 						runtime_ptr,
		// 						$(
		// 						self.param((&mut i) as &mut $V, parameters),
		// 						)*
		// 					)
		// 				}
		// 			};
		// 		}
		//
		// 		let value = match parameters.len() {
		// 			#[allow(unused)]
		// 			0 => param!(),
		// 			1 => param!(usize),
		// 			2 => param!(usize, usize),
		// 			3 => param!(usize, usize, usize),
		// 			4 => param!(usize, usize, usize, usize),
		// 			5 => param!(usize, usize, usize, usize, usize),
		// 			6 => param!(usize, usize, usize, usize, usize, usize),
		// 			_ => {
		// 				panic!()
		// 			}
		// 		};
		//
		// 		unsafe {
		// 			// We decrement our strong reference count
		// 			Arc::from_raw(runtime_ptr);
		// 		}
		//
		// 		returns.map(|returns| Self::convert_from(value, returns))
	}

	//unsafe fn param(&self, idx: &mut usize, parameters: &[AnyValue]) -> usize {
	//	let index = *idx;
	//	*idx += 1;
	//	let value = parameters[index];
	//	if let Some(signature) = &self.signature {
	//		let kind = signature.parameters[index];
	//		if value.kind() != kind {
	//			panic!("Missmatched types, {} != {}", value.kind(), kind);
	//		}
	//	}
	//	Self::convert_to(value)
	//}
	//fn convert_from(value: usize, kind: Kind) -> AnyValue {
	//	match kind {
	//		Kind::Byte => AnyValue::Byte(value as _),
	//		Kind::Short => AnyValue::Short(value as _),
	//		Kind::Int => AnyValue::Int(value as _),
	//		Kind::Long => AnyValue::Long(value as _),
	//		Kind::Char => AnyValue::Char(value as _),
	//		Kind::Float => AnyValue::Float(value as _),
	//		Kind::Double => AnyValue::Double(value as _),
	//		Kind::Boolean => AnyValue::Boolean(value != 0),
	//		Kind::Reference => AnyValue::Reference(Reference::new(value as _)),
	//	}
	//}
	//fn convert_to(value: AnyValue) -> usize {
	//	match value {
	//		AnyValue::Byte(v) => v as usize,
	//		AnyValue::Short(v) => v as usize,
	//		AnyValue::Int(v) => v as usize,
	//		AnyValue::Long(v) => v as usize,
	//		AnyValue::Char(v) => v as usize,
	//		AnyValue::Float(v) => v as usize,
	//		AnyValue::Double(v) => v as usize,
	//		AnyValue::Boolean(v) => v as usize,
	//		AnyValue::Reference(v) => v.0 as usize,
	//	}
	//}
}
