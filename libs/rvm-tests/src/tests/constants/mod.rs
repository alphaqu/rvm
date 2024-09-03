use crate::{simple_launch, SimpleClassTest, SimpleMethodTest};

#[test]
fn test() {
	simple_launch(vec![SimpleClassTest {
		name: "tests/constants/ConstantTests".to_string(),
		methods: vec![SimpleMethodTest::void("test")],
	}]);
}

macro_rules! java_bind {
	(package $PACKAGE:path; pub struct $TY:ident {
		$(
			pub fn $FUNC_NAME:ident($($PARAM_NAME:ident: $PARAM_TYPE: ty),*) $(-> $RETURNS:ty)? $FUNC_BLOCK:block
		)*
	}) => {

		pub struct $TY;

		#[allow(unused)]
		impl $TY {
			$(

				pub fn $FUNC_NAME($($PARAM_NAME: $PARAM_TYPE),*) $(-> $RETURNS)? {
					$FUNC_BLOCK
				}
			)*
		}
	};
}

java_bind! {
	package tests::constants;

	pub struct ConstantTests {
		pub fn native_function(input: i32) -> i32 {
			input
		}
	}
}
