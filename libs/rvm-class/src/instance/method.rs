use eyre::Context;
use rvm_core::StorageValue;
use rvm_core::{MethodAccessFlags, MethodDescriptor};
use rvm_core::{Storage, VecExt};
use rvm_reader::{AttributeInfo, Code, ConstantPool, MethodInfo, NameAndTypeConst};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

pub struct ClassMethods {
	storage: Storage<MethodIdentifier, Method>,
}

impl ClassMethods {
	pub fn empty() -> ClassMethods {
		ClassMethods {
			storage: Storage::new(),
		}
	}

	pub fn new(methods: Vec<Method>) -> ClassMethods {
		let mut storage = Storage::new();
		for method in methods {
			let key = MethodIdentifier {
				name: method.name.clone().into(),
				descriptor: method.desc.to_string().into(),
			};
			storage.insert(key, method);
		}

		ClassMethods { storage }
	}
	pub fn parse(methods: Vec<MethodInfo>, cp: &ConstantPool) -> eyre::Result<ClassMethods> {
		let mut output = Vec::new();
		for method in methods {
			let name = method.name_index.get(cp).unwrap().as_str();
			let method =
				Method::parse(method, cp).wrap_err_with(|| format!("in METHOD \"{}\"", name))?;
			output.push(method);
		}
		Ok(ClassMethods::new(output))
	}
}

impl Deref for ClassMethods {
	type Target = Storage<MethodIdentifier, Method>;

	fn deref(&self) -> &Self::Target {
		&self.storage
	}
}
impl DerefMut for ClassMethods {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.storage
	}
}

pub struct Method {
	pub name: String,
	pub desc: MethodDescriptor,
	pub flags: MethodAccessFlags,
	pub code: Option<Code>,
	pub attributes: Vec<AttributeInfo>,
}

impl Method {
	pub fn parse(mut info: MethodInfo, consts: &ConstantPool) -> eyre::Result<Method> {
		let desc_str = consts.get(info.descriptor_index).unwrap();
		let desc = MethodDescriptor::parse(desc_str).unwrap();

		let mut code = None;
		let ident = MethodIdentifier {
			name: consts.get(info.name_index).unwrap().to_string().into(),
			descriptor: desc_str.to_string().into(),
		};

		if info.access_flags.contains(MethodAccessFlags::NATIVE) {
			//	code = Some(MethodCode::Binding(ident.clone()));
		} else {
			// Get the code
			if let Some(code_attr) = info
				.attributes
				.find_and_remove(|v| matches!(v, AttributeInfo::CodeAttribute { .. }))
			{
				let AttributeInfo::CodeAttribute { code: c } = code_attr else {
					unreachable!();
				};

				code = Some(c);
			}
		}

		Ok(Method {
			name: ident.name.to_string(),
			desc,
			flags: info.access_flags,
			code,
			attributes: info.attributes,
		})
	}

	pub fn to_identifier(&self) -> MethodIdentifier {
		MethodIdentifier {
			name: Arc::from(&*self.name),
			descriptor: Arc::from(self.desc.to_java()),
		}
	}

	pub fn is_static(&self) -> bool {
		self.flags.contains(MethodAccessFlags::STATIC)
	}
}

impl StorageValue for Method {
	type Idx = u16;
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct MethodIdentifier {
	pub name: Arc<str>,
	pub descriptor: Arc<str>,
}

impl MethodIdentifier {
	pub fn new(nat: &NameAndTypeConst, cp: &ConstantPool) -> MethodIdentifier {
		MethodIdentifier {
			name: nat.name.get(cp).unwrap().to_string().into(),
			descriptor: nat.descriptor.get(cp).unwrap().to_string().into(),
		}
	}
}
