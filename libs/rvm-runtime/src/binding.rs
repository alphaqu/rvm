use crate::{AnyValue, FromJavaMulti, JavaTypedMulti, ToJavaMulti, Vm};
use ahash::HashMap;
use parking_lot::RwLock;
use rvm_core::{MethodDescriptor, Type};
use std::collections::hash_map::Entry;
use std::sync::Arc;
use tracing::warn;

pub struct RustBinder {
	methods: RwLock<RustBinderInner>,
}

impl RustBinder {
	pub fn new() -> RustBinder {
		RustBinder {
			methods: RwLock::new(RustBinderInner {
				short_names: Default::default(),
				long_names: Default::default(),
			}),
		}
	}
	// https://docs.oracle.com/javase/1.5.0/docs/guide/jni/spec/design.html
	pub fn get_binding(
		&self,
		class_name: &str,
		method_name: &str,
		descriptor: &MethodDescriptor,
	) -> Option<Arc<MethodBinding>> {
		let inner = self.methods.read();

		let short_name = MethodDescriptor::jni_short_name(class_name, method_name);
		if let Some(Some(binding)) = inner.short_names.get(&short_name) {
			if &binding.signature == descriptor {
				return Some(binding.clone());
			}
		}

		let long_name =
			MethodDescriptor::jni_long_name(class_name, method_name, &descriptor.parameters);
		if let Some(binding) = inner.long_names.get(&long_name) {
			if &binding.signature == descriptor {
				return Some(binding.clone());
			}
		}

		None
	}
	pub fn bind(&self, class_name: &str, method_name: &str, binding: MethodBinding) {
		let mut inner = self.methods.write();

		let binding = Arc::new(binding);

		let short_name = MethodDescriptor::jni_short_name(class_name, method_name);
		let long_name =
			MethodDescriptor::jni_long_name(class_name, method_name, &binding.signature.parameters);

		match inner.short_names.entry(short_name) {
			Entry::Occupied(mut value) => {
				// Remove the short_name binding, (because there are overloading methods.)
				value.insert(None);
			}
			Entry::Vacant(value) => {
				value.insert(Some(binding.clone()));
			}
		}

		if inner
			.long_names
			.insert(long_name, binding.clone())
			.is_some()
		{
			let arguments: Vec<String> = binding
				.signature
				.parameters
				.iter()
				.map(|v| format!("{v:?}"))
				.collect();
			warn!(
				"Conflicting method binding! {class_name} {method_name} with ({})",
				arguments.join(", ")
			);
		}
	}
}

pub struct RustBinderInner {
	short_names: HashMap<String, Option<Arc<MethodBinding>>>,
	long_names: HashMap<String, Arc<MethodBinding>>,
}

pub struct MethodBinding {
	function: Box<dyn Fn(&Vm, Vec<AnyValue>) -> eyre::Result<Option<AnyValue>> + Send + Sync>,
	signature: MethodDescriptor,
}

fn single_or_none<V>(mut vec: Vec<V>) -> Option<V> {
	match vec.len() {
		0 => None,
		1 => vec.pop(),
		_ => {
			panic!("Trying to return more than 1 value");
		}
	}
}
impl MethodBinding {
	pub fn new<I, O, F>(function: F) -> Self
	where
		F: Fn(&Vm, I) -> O + Send + Sync + 'static,
		I: FromJavaMulti + JavaTypedMulti,
		O: ToJavaMulti + JavaTypedMulti,
	{
		let function =
			move |runtime: &Vm, values: Vec<AnyValue>| -> eyre::Result<Option<AnyValue>> {
				let input = I::from_vec(values, runtime)?;
				let output = function(runtime, input);
				let result = output.to_vec(runtime)?;
				if result.len() > 1 {
					panic!("Trying to return more than 1 value");
				}

				Ok(single_or_none(result))
			};

		let input_types = I::java_type_multi();
		let output_type = single_or_none(O::java_type_multi());

		MethodBinding {
			function: Box::new(function),
			signature: MethodDescriptor {
				parameters: input_types,
				returns: output_type,
			},
		}
	}

	pub fn call(&self, runtime: &Vm, parameters: Vec<AnyValue>) -> eyre::Result<Option<AnyValue>> {
		(self.function)(runtime, parameters)
	}
}
