use rvm_core::Kind;
use rvm_runtime::Reference;

pub trait ReturnValue {
	fn kind() -> Option<Kind>;
}

macro_rules! val {
	($TY:ty) => {
		impl ReturnValue for $TY {
			fn kind() -> Option<Kind> {
				Some(Self::ty())
			}
		}
	};
}
val!(i8);
val!(i16);
val!(i32);
val!(i64);
val!(u16);
val!(f32);
val!(f64);
val!(bool);
val!(Reference);

impl ReturnValue for () {
	fn kind() -> Option<Kind> {
		None
	}
}
