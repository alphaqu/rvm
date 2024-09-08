//use rvm_core::Kind;
// use rvm_core::{CastKindError, StackKind};
// use std::fmt::{Display, Formatter};
// #[repr(C)]
// #[derive(Copy, Clone, Debug, PartialEq, Zeroable)]
// pub enum StackValue {
// 	Int(i32),
// 	Float(f32),
// 	Long(i64),
// 	Double(f64),
// 	Reference(*mut u8),
// }
//
// impl Display for StackValue {
// 	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
// 		match self {
// 			StackValue::Int(v) => write!(f, "{v}"),
// 			StackValue::Float(v) => write!(f, "{v}F"),
// 			StackValue::Long(v) => write!(f, "{v}L"),
// 			StackValue::Double(v) => write!(f, "{v}.0"),
// 			StackValue::Reference(v) => write!(f, "{v:?}"),
// 		}
// 	}
// }
//
// macro_rules! to_impl {
// 	($($METHOD_NAME:ident $KIND:ident $TY:ty),*) => {
// 		impl StackValue {
// 			$(
// 				pub fn $METHOD_NAME(self) -> Result<$TY, CastKindError> {
// 					match self {
// 						StackValue::$KIND(value) => Ok(value),
// 						_ => Err(CastKindError {
// 							expected: Kind::$KIND,
// 							found: self.kind().kind(),
// 						}),
// 					}
// 				}
// 			)*
// 		}
//
// 		$(
// 			impl TryInto<$TY> for StackValue {
// 				type Error = CastKindError;
//
// 				fn try_into(self) -> Result<$TY, Self::Error> {
// 					self.$METHOD_NAME()
// 				}
// 			}
// 		)*
// 	};
// }
//
// to_impl!(
// 	to_int Int i32,
// 	to_long Long i64,
// 	to_float Float f32,
// 	to_double Double f64,
// 	to_ref Reference *mut u8
// );
// impl StackValue {
// 	pub fn kind(&self) -> StackKind {
// 		match self {
// 			StackValue::Int(_) => StackKind::Int,
// 			StackValue::Float(_) => StackKind::Float,
// 			StackValue::Long(_) => StackKind::Long,
// 			StackValue::Double(_) => StackKind::Double,
// 			StackValue::Reference(_) => StackKind::Reference,
// 		}
// 	}
//
// 	pub fn category(&self) -> u8 {
// 		match self {
// 			StackValue::Int(_) => 1,
// 			StackValue::Float(_) => 1,
// 			StackValue::Reference(_) => 1,
// 			StackValue::Long(_) => 2,
// 			StackValue::Double(_) => 2,
// 		}
// 	}
//
// 	pub fn category_1(&self) -> bool {
// 		self.category() == 1
// 	}
//
// 	pub fn category_2(&self) -> bool {
// 		self.category() == 2
// 	}
// }
