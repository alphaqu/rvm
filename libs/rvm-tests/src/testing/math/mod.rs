use num_traits::{Num, PrimInt};
use std::fmt::{Debug, Display, Write};
use std::fs::read;
use std::str::FromStr;

use rvm_runtime::java_bind_method;

use crate::launch;
use crate::testing::consts::Samples;
use crate::testing::quick_compile;

macro_rules! test_op {
    ($TY:ty: $METHOD:ident $RUST_METHOD:ident) => {
		let runtime = launch(1024, vec!["testing/math/MathTests.class"]);
		let func =
			java_bind_method!(runtime fn testing::math::MathTests:$METHOD(left: $TY, right: $TY) -> $TY);
		for v0 in <$TY>::samples() {
			for v1 in <$TY>::samples() {
				assert_eq!(func(v0, v1), v0.$RUST_METHOD(v1));
			}
		}
	};
}

pub fn cmp_integer<I: PrimInt>(v0: I, v1: I) {}

#[test]
fn add_int() {
	test_op!(i32: add wrapping_add);
	test_op!(i32: sub wrapping_sub);
	test_op!(i32: mul wrapping_mul);
	test_op!(i32: div wrapping_div);
}

#[test]
fn add_long() {
	test_op!(i64: add wrapping_add);
}

#[test]
fn add_floats() {
	let runtime = launch(1024, vec!["testing/math/MathTests.class"]);
	let func =
		java_bind_method!(runtime fn testing::math::MathTests:add(left: f32, right: f32) -> f32);
	for v0 in f32::samples() {
		for v1 in f32::samples() {
			let rust = v0 + v1;
			let java = func(v0, v1);
			let is_nan = rust.is_nan();
			if is_nan {
				assert!(java.is_nan());
			} else {
				assert_eq!(java, rust);
			}
		}
	}
}

#[test]
fn add_doubles() {
	let runtime = launch(1024, vec!["testing/math/MathTests.class"]);
	let func =
		java_bind_method!(runtime fn testing::math::MathTests:add(left: f64, right: f64) -> f64);
	for v0 in f64::samples() {
		for v1 in f64::samples() {
			let rust = v0 + v1;
			let java = func(v0, v1);
			let is_nan = rust.is_nan();
			if is_nan {
				assert!(java.is_nan());
			} else {
				assert_eq!(java, rust);
			}
		}
	}
}
//fn test_op<V: Clone + Debug + PartialEq + Copy + Display + FromStr>(
// 	values: Vec<V>,
// 	name: &str,
// 	convert: fn(V, V) -> String,
// 	mut func: impl FnMut(V, V) -> V,
// ) where
// 	V::Err: Debug,
// {
// 	let map: Vec<(V, V)> = values
// 		.clone()
// 		.into_iter()
// 		.flat_map(|v0| values.clone().into_iter().map(move |v1| (v0, v1)))
// 		.collect();
//
// 	let string = generate_results(name, map.clone(), |(v0, v1)| {
// 		format!("System.out.println({});", convert(v0, v1))
// 	});
// 	let results: Vec<V> = parse_results(&string);
// 	for ((v0, v1), result) in map.into_iter().zip(results) {
// 		assert_eq!(result, func(v0, v1));
// 	}
// }

fn generate_results<V>(
	name: &str,
	values: Vec<V>,
	mut java_mapper: impl FnMut(V) -> String,
) -> String {
	let mut output = String::new();
	output.write_str("public class Main {").unwrap();
	output
		.write_str("\tpublic static void main(String[] args) {")
		.unwrap();
	for x in values {
		output.write_str("\t\t").unwrap();
		output.write_str(&java_mapper(x)).unwrap();
		output.write_str("\n").unwrap();
	}
	output.write_str("\t}").unwrap();
	output.write_str("}").unwrap();

	quick_compile(name, &output)
}

fn parse_results<F: FromStr>(text: &str) -> Vec<F>
where
	F::Err: Debug,
{
	let mut output = vec![];
	for value in text.split('\n') {
		if value.trim().is_empty() {
			continue;
		}
		output.push(value.parse().unwrap());
	}

	output
}
