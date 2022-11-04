use crate::compiler::compiler::BlockCompiler;

use crate::compiler::resolver::BlockResolver;
use crate::executor::Inst;

use crate::compiler::{MethodReference, Reference};
use either::Either;
use inkwell::values::{BasicMetadataValueEnum, CallableValue};
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug)]
pub struct InvokeTask {
	pub kind: InvokeKind,
	pub method: MethodReference,
}

impl InvokeTask {
	pub fn resolve(inst: &Inst, resolver: &mut BlockResolver) -> InvokeTask {
		let (ptr, kind) = match inst {
			Inst::INVOKEINTERFACE(_, _) => todo!("interface"),
			Inst::INVOKEDYNAMIC(_) => todo!("dynamic"),
			Inst::INVOKESPECIAL(v) => (v, InvokeKind::Dynamic),
			Inst::INVOKESTATIC(v) => (v, InvokeKind::Static),
			Inst::INVOKEVIRTUAL(v) => (v, InvokeKind::Virtual),
			_ => panic!("what"),
		};

		let cp = resolver.cp();
		let method = cp.get(*ptr);
		let name_and_type = method.name_and_type.get(cp);

		let class_name = method.class.get(cp).name.get(cp).as_str().to_string();
		let method_name = name_and_type.name.get(cp).to_string();
		let desc_raw = name_and_type.descriptor.get(cp).as_str();
		resolver.add_ref(Reference::Method(MethodReference {
			class_name: method.class.get(cp).name.get(cp).0.clone(),
			method_name: method_name.clone(),
			desc: desc_raw.to_string(),
		}));

		InvokeTask {
			kind,
			method: MethodReference {
				class_name,
				method_name,
				desc: desc_raw.to_string(),
			},
		}
	}

	pub fn compile<'b, 'a>(&self, bc: &mut BlockCompiler<'b, 'a>) {
		// Create args
		let mut args = Vec::new();
		let desc = self.method.desc();
		let instance = !matches!(self.kind, InvokeKind::Static);

		for _parameter in desc.parameters.iter().rev() {
			// Todo check parameters
			let value = bc.pop();
			args.push(BasicMetadataValueEnum::from(value));
		}
		if instance {
			let value = bc.pop();
			args.push(BasicMetadataValueEnum::from(value));
		}

		args.reverse();

		// Resolve method

		let function = if let Some(value) = bc.module().get_function(&self.method.def_name()) {
			value
		} else {
			let string = self.method.call_name();
			bc.module().get_function(&string).unwrap()
		};

		let value: CallableValue = function.into();
		let name = bc.gen.next();
		match bc.build_call(value, &args, &name).try_as_basic_value() {
			Either::Left(value) => {
				bc.push(value);
			}
			Either::Right(_) => {}
		}
	}
}

impl Display for InvokeTask {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "invoke {}{}", self.method.method_name, self.method.desc)
	}
}

#[derive(Clone, Debug)]
pub enum InvokeKind {
	Static,
	Dynamic,
	Virtual,
}
