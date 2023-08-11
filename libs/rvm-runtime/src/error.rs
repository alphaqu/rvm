use crate::Runtime;
use rvm_core::Id;
use std::fmt::Debug;

pub type JResult<V> = Result<V, JError>;
use rvm_object::{Class, ClassKind, Method, MethodData};
use std::fmt::Write;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JError {
	// thread: Id<Thread>
	// TODO object
	pub message: String,
	pub stacktrace: Vec<TraceEntry>,
}

impl JError {
	pub fn new(message: impl ToString) -> JError {
		JError {
			message: message.to_string(),
			stacktrace: vec![],
		}
	}
}

impl JError {
	pub fn fmt(&self, f: &mut String, runtime: &Runtime) -> std::fmt::Result {
		writeln!(
			f,
			"Exception in thread \"main\" java.lang.FuckThisShitException: {}",
			self.message
		)?;
		for trace in &self.stacktrace {
			trace.fmt(f, runtime)?;
		}
		Ok(())
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TraceEntry {
	pub class: Id<Class>,
	pub method: Id<Method>,
	pub line: u32,
}

impl TraceEntry {
	fn fmt(&self, f: &mut String, runtime: &Runtime) -> std::fmt::Result {
		let class = runtime.class_loader.get(self.class);
		match &class.kind {
			ClassKind::Object(object) => {
				let method = object.methods.get(self.method);
				writeln!(
					f,
					"\tat {full_class_name}.{method_name}({class_name}.java:{line})",
					full_class_name = class.name,
					class_name = class.name,
					method_name = method.name,
					line = self.line,
				)
			}
			ClassKind::Array(_) => {
				writeln!(f, "\tat array garbage")
			}
			ClassKind::Primitive(_) => {
				writeln!(f, "\tits executing inside a primitive lmao")
			}
		}
	}
}
