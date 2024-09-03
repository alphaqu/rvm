use crate::{AnyValue, FromJavaMulti, JavaTypedMulti, Runtime, ToJavaMulti};
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

		let short_name = MethodBinding::jni_short_name(class_name, method_name);
		if let Some(Some(binding)) = inner.short_names.get(&short_name) {
			if &binding.signature == descriptor {
				return Some(binding.clone());
			}
		}

		let long_name =
			MethodBinding::jni_long_name(class_name, method_name, &descriptor.parameters);
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

		let short_name = MethodBinding::jni_short_name(class_name, method_name);
		let long_name =
			MethodBinding::jni_long_name(class_name, method_name, &binding.signature.parameters);

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
	function:
		Box<dyn Fn(&Arc<Runtime>, Vec<AnyValue>) -> eyre::Result<Option<AnyValue>> + Send + Sync>,
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
	pub fn jni_short_name(class_name: &str, method_name: &str) -> String {
		Self::jni_name(class_name, method_name, None)
	}

	pub fn jni_long_name(class_name: &str, method_name: &str, arguments: &[Type]) -> String {
		Self::jni_name(class_name, method_name, Some(arguments))
	}
	fn jni_name(class_name: &str, method_name: &str, signature: Option<&[Type]>) -> String {
		let mut out = String::new();
		out.push_str("Java_");

		let push_mangled = |out: &mut String, string: &str| {
			for char in string.chars() {
				if char == '/' {
					out.push('_')
				} else if char == '_' {
					out.push_str("_1")
				} else if char == ';' {
					out.push_str("_2")
				} else if char == '[' {
					out.push_str("_3")
				} else if char.is_ascii_alphanumeric() {
					out.push(char);
				} else {
					panic!("Unsupported rn {char}!!")
				}
			}
		};

		push_mangled(&mut out, class_name);
		out.push('_');
		push_mangled(&mut out, method_name);

		if let Some(arguments) = signature {
			out.push_str("__");
			for argument in arguments {
				push_mangled(&mut out, &format!("{argument}"));
			}
		}
		out
	}

	pub fn new<I, O, F>(function: F) -> Self
	where
		F: Fn(&Arc<Runtime>, I) -> O + Send + Sync + 'static,
		I: FromJavaMulti + JavaTypedMulti,
		O: ToJavaMulti + JavaTypedMulti,
	{
		let function = move |runtime: &Arc<Runtime>,
		                     values: Vec<AnyValue>|
		      -> eyre::Result<Option<AnyValue>> {
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

	pub fn call(
		&self,
		runtime: &Arc<Runtime>,
		parameters: Vec<AnyValue>,
	) -> eyre::Result<Option<AnyValue>> {
		(self.function)(runtime, parameters)
	}
}
