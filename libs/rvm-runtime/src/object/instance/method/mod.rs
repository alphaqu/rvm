use eyre::Context;
use rvm_core::Storage;
use rvm_core::StorageValue;
use rvm_core::{MethodAccessFlags, MethodDescriptor};
use rvm_reader::{AttributeInfo, Code, ConstantPool, MethodInfo, NameAndTypeConst};
use std::ops::Deref;
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
		let mut storage = Storage::new();
		for method in methods {
			let name = method.name_index.get(cp).unwrap().as_str();
			let (name, method) =
				Method::parse(method, cp).wrap_err_with(|| format!("in METHOD \"{}\"", name))?;
			storage.insert(name, method);
		}
		Ok(ClassMethods { storage })
	}
}

impl Deref for ClassMethods {
	type Target = Storage<MethodIdentifier, Method>;

	fn deref(&self) -> &Self::Target {
		&self.storage
	}
}

pub struct Method {
	pub name: String,
	pub desc: MethodDescriptor,
	pub flags: MethodAccessFlags,
	pub code: Option<Code>,
}

impl Method {
	//pub fn new(
	//	name: String,
	//	desc: String,
	//	flags: MethodAccessFlags,
	//	code: Code,
	//) -> (MethodIdentifier, Method) {
	//	(
	//		MethodIdentifier {
	//			name: name.to_string().into(),
	//			descriptor: desc.to_string().into(),
	//		},
	//		Method {
	//			name,
	//			desc: MethodDescriptor::parse(&desc).unwrap(),
	//			flags,
	//			code: Some(code),
	//		},
	//	)
	//}

	pub fn parse(
		info: MethodInfo,
		consts: &ConstantPool,
	) -> eyre::Result<(MethodIdentifier, Method)> {
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
			for attribute in info.attribute_info {
				if let AttributeInfo::CodeAttribute { code: c } = attribute {
					code = Some(c);
				}
			}
		}

		Ok((
			ident.clone(),
			Method {
				name: ident.name.to_string(),
				desc,
				flags: info.access_flags,
				code,
			},
		))
	}
}

impl StorageValue for Method {
	type Idx = u16;
}

pub enum MethodCode {
	Java(Code),
	Binding(MethodIdentifier),
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
