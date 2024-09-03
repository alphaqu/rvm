use crate::{AnyValue, FromJava, JavaTyped, Runtime, ToJava};
use eyre::{bail, Context};
use rvm_core::Type;
use std::sync::Arc;

pub trait FromJavaMulti: Sized {
	fn from_vec(vec: Vec<AnyValue>, runtime: &Runtime) -> eyre::Result<Self>;
}
pub trait ToJavaMulti: Sized {
	fn to_vec(self, runtime: &Runtime) -> eyre::Result<Vec<AnyValue>>;
}

pub trait JavaTypedMulti {
	fn java_type_multi() -> Vec<Type>;
}

fn pop_counted<V: FromJava>(
	vec: &mut Vec<AnyValue>,
	runtime: &Runtime,
	i: &mut usize,
) -> eyre::Result<V> {
	let value = vec.pop().unwrap();
	let value = V::from_java(value, runtime)
		.wrap_err_with(|| format!("Parameter {i} failed to convert!"))?;
	*i -= 1;
	Ok(value)
}

// This is to select $V in the macro, else the loop will shit itself.
fn count<V>() -> usize {
	1
}

impl<V: ToJavaMulti> ToJavaMulti for eyre::Result<V> {
	fn to_vec(self, runtime: &Runtime) -> eyre::Result<Vec<AnyValue>> {
		let value = self?;
		V::to_vec(value, runtime)
	}
}

macro_rules! impl_from_java_multi {
    ($($V:ident),*) => {
		impl<$($V: JavaTyped),*> JavaTypedMulti for ($($V),*) {
			#[allow(non_snake_case)]
			fn java_type_multi() -> Vec<Type> {
				let mut out = Vec::new();
				$(out.push($V::java_type());)*
				out
			}
		}
		impl<$($V: ToJava),*> ToJavaMulti for ($($V),*) {
			#[allow(non_snake_case)]
			fn to_vec(self, runtime: &Runtime) -> eyre::Result<Vec<AnyValue>> {
				let mut out = Vec::new();
				let ($($V),*) = self;
				$(out.push($V::to_java($V, runtime)?);)*
				Ok(out)
			}
		}
		impl<$($V: FromJava),*> FromJavaMulti for ($($V),*) {
			fn from_vec(mut vec: Vec<AnyValue>, runtime: &Runtime) -> eyre::Result<Self> {
				let mut count = 0 $(+ count::<$V>())*;
				if vec.len() != count {
					bail!("Expected {count} parameters, but found {}", vec.len());
				}

				// This should prob be done on the macro level... but im lazy.
				vec.reverse();
				Ok(($(pop_counted::<$V>(&mut vec, runtime, &mut count)?),*))
			}
		}
	};
}

impl_from_java_multi!();
impl_from_java_multi!(V0);
impl_from_java_multi!(V0, V1);
impl_from_java_multi!(V0, V1, V2);
impl_from_java_multi!(V0, V1, V2, V3);
