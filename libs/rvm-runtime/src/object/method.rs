use crate::executor::{LocalVariables, StackValue};
use crate::reader::{
	AttributeInfo, Code, ConstantPool, MethodDescriptor, MethodInfo, NameAndTypeConst,
};
use crate::{ClassLoader, JResult, Runtime, StrParse};
use anyways::audit::Audit;
use anyways::Result;
use rvm_consts::MethodAccessFlags;
use rvm_core::StorageValue;
use std::sync::Arc;

pub struct Method {
	pub name: String,
	pub desc: MethodDescriptor,
	pub flags: MethodAccessFlags,
	pub max_locals: u16,
	pub max_stack: u16,
	pub code: Option<MethodCode>,
}

impl Method {
	pub fn new(
		name: String,
		desc: String,
		flags: MethodAccessFlags,
		code: MethodCode,
	) -> (MethodIdentifier, Method) {
		(
			MethodIdentifier {
				name: name.to_string(),
				descriptor: desc.to_string(),
			},
			Method {
				name,
				desc: MethodDescriptor::parse(&desc).unwrap(),
				flags,
				max_locals: code.max_locals(),
				max_stack: code.max_stack(),
				code: Some(code),
			},
		)
	}

	pub fn parse(
		info: MethodInfo,
		class_name: &str,
		consts: &ConstantPool,
		class_loader: &ClassLoader,
	) -> Result<(MethodIdentifier, Method)> {
		let desc_str = consts.get(info.descriptor_index);
		let desc = MethodDescriptor::parse(desc_str).unwrap();

		let mut code = None;
		let ident = MethodIdentifier {
			name: consts.get(info.name_index).to_string(),
			descriptor: desc_str.to_string(),
		};
		if info.access_flags.contains(MethodAccessFlags::NATIVE) {
			match class_loader
				.native_methods()
				.get(&(class_name.to_string(), ident.clone()))
			{
				None => {
					return Err(Audit::new(format!(
						"Could not find native method {class_name} {ident:?}"
					)));
				}
				Some(method) => {
					code = Some(MethodCode::Native(*method));
				}
			}
		} else {
			for attribute in info.attribute_info {
				match attribute {
					AttributeInfo::CodeAttribute { code: c } => {
						code = Some(MethodCode::JVM(Arc::new(c)));
					}
					_ => {}
				}
			}
		}

		Ok((
			ident.clone(),
			Method {
				name: ident.name,
				desc,
				flags: info.access_flags,
				max_locals: code.as_ref().map(|v| v.max_locals()).unwrap_or(0),
				max_stack: code.as_ref().map(|v| v.max_stack()).unwrap_or(0),
				code: code,
			},
		))
	}
}

impl StorageValue for Method {
	type Idx = u16;
}

#[derive(Clone)]
pub enum MethodCode {
	JVM(Arc<Code>),
	Native(NativeCode),
}

impl MethodCode {
	pub fn max_locals(&self) -> u16 {
		match self {
			MethodCode::JVM(code) => code.max_locals,
			MethodCode::Native(code) => code.max_locals,
		}
	}

	pub fn max_stack(&self) -> u16 {
		match self {
			MethodCode::JVM(code) => code.max_stack,
			MethodCode::Native(code) => 0,
		}
	}
}

#[derive(Copy, Clone)]
pub struct NativeCode {
	pub func: fn(&mut LocalVariables, &Runtime) -> JResult<Option<StackValue>>,
	pub max_locals: u16,
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct MethodIdentifier {
	pub name: String,
	pub descriptor: String,
}

impl MethodIdentifier {
	pub fn new(nat: &NameAndTypeConst, cp: &ConstantPool) -> MethodIdentifier {
		MethodIdentifier {
			name: nat.name.get(cp).to_string(),
			descriptor: nat.descriptor.get(cp).to_string(),
		}
	}
}
