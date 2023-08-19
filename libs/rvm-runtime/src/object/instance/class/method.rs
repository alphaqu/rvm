use std::cell::Cell;
use std::ffi::c_void;
use std::ops::Deref;
use std::sync::Arc;

use anyways::ext::AuditExt;
use anyways::Result;
use either::Either;

use rvm_core::{MethodAccessFlags, MethodDesc};
use rvm_core::Storage;
use rvm_core::StorageValue;
use rvm_reader::{AttributeInfo, Code, ConstantPool, MethodInfo, NameAndTypeConst};

pub struct ClassMethodManager {
	storage: Storage<MethodIdentifier, Method>,
}

impl ClassMethodManager {
	pub fn empty() -> ClassMethodManager {
		ClassMethodManager {
			storage: Storage::new(),
		}
	}
	pub fn parse(
		methods: Vec<MethodInfo>,
		class_name: &str,
		cp: &ConstantPool,
	) -> Result<ClassMethodManager> {
		let mut storage = Storage::new();
		for method in methods {
			let name = method.name_index.get(cp).unwrap().as_str();
			let (name, method) = MethodData::parse(method, class_name, cp)
				.wrap_err_with(|| format!("in METHOD \"{}\"", name))?;
			storage.insert(
				name,
				Method {
					data: method,
					compiled: Cell::new(None),
				},
			);
		}
		Ok(ClassMethodManager { storage })
	}
}

impl Deref for ClassMethodManager {
	type Target = Storage<MethodIdentifier, Method>;

	fn deref(&self) -> &Self::Target {
		&self.storage
	}
}

pub struct Method {
	data: MethodData,
	pub compiled: Cell<Option<*const c_void>>,
}

impl Deref for Method {
	type Target = MethodData;

	fn deref(&self) -> &Self::Target {
		&self.data
	}
}

#[derive(Clone)]
pub struct MethodData {
	pub name: String,
	pub desc: MethodDesc,
	pub flags: MethodAccessFlags,
	pub code: Option<Arc<MethodCode>>,
}

impl MethodData {
	pub fn new(
		name: String,
		desc: String,
		flags: MethodAccessFlags,
		code: MethodCode,
	) -> (MethodIdentifier, MethodData) {
		(
			MethodIdentifier {
				name: name.to_string(),
				descriptor: desc.to_string(),
			},
			MethodData {
				name,
				desc: MethodDesc::parse(&desc).unwrap(),
				flags,
				code: Some(Arc::new(code)),
			},
		)
	}

	pub fn parse(
		info: MethodInfo,
		class_name: &str,
		consts: &ConstantPool,
	) -> Result<(MethodIdentifier, MethodData)> {
		let desc_str = consts.get(info.descriptor_index).unwrap();
		let desc = MethodDesc::parse(desc_str).unwrap();

		let mut code = None;
		let ident = MethodIdentifier {
			name: consts.get(info.name_index).unwrap().to_string(),
			descriptor: desc_str.to_string(),
		};

		if info.access_flags.contains(MethodAccessFlags::NATIVE) {
			code = Some(Arc::new(MethodCode::Native(Either::Left((
				class_name.to_string(),
				ident.clone(),
			)))));
		} else {
			for attribute in info.attribute_info {
				if let AttributeInfo::CodeAttribute { code: c } = attribute {
					code = Some(Arc::new(MethodCode::Java(c)));
				}
			}
		}

		Ok((
			ident.clone(),
			MethodData {
				name: ident.name,
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
	Native(Either<(String, MethodIdentifier), NativeCode>),
}

#[derive(Copy, Clone)]
pub struct NativeCode {
	pub func: unsafe extern "C" fn(),
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct MethodIdentifier {
	pub name: String,
	pub descriptor: String,
}

impl MethodIdentifier {
	pub fn new(nat: &NameAndTypeConst, cp: &ConstantPool) -> MethodIdentifier {
		MethodIdentifier {
			name: nat.name.get(cp).unwrap().to_string(),
			descriptor: nat.descriptor.get(cp).unwrap().to_string(),
		}
	}
}
