use std::ops::Deref;

use anyways::ext::AuditExt;
use anyways::Result;

use rvm_core::Storage;

use either::Either;
use rvm_core::StorageValue;
use rvm_core::{MethodAccessFlags, MethodDesc};
use rvm_reader::{AttributeInfo, Code, ConstantPool, MethodInfo, NameAndTypeConst};
use std::cell::Cell;
use std::ffi::c_void;
use std::sync::Arc;


pub struct ClassMethodManager {
	storage: Storage<MethodIdentifier, Method>,
}

impl ClassMethodManager {
	pub fn parse(
		methods: Vec<MethodInfo>,
		class_name: &str,
		cp: &ConstantPool,
	) -> Result<ClassMethodManager> {
		let mut storage = Storage::new();
		for method in methods {
			let name = method.name_index.get(cp).as_str();
			let (name, method) = Method::parse(method, class_name, cp)
				.wrap_err_with(|| format!("in METHOD \"{}\"", name))?;
			storage.insert(name, method);
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
	pub name: String,
	pub desc: MethodDesc,
	pub flags: MethodAccessFlags,
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
				desc: MethodDesc::parse(&desc).unwrap(),
				flags,
				code: Some(code),
			},
		)
	}

	pub fn parse(
		info: MethodInfo,
		class_name: &str,
		consts: &ConstantPool,
	) -> Result<(MethodIdentifier, Method)> {
		let desc_str = consts.get(info.descriptor_index);
		let desc = MethodDesc::parse(desc_str).unwrap();

		let mut code = None;
		let ident = MethodIdentifier {
			name: consts.get(info.name_index).to_string(),
			descriptor: desc_str.to_string(),
		};

		if info.access_flags.contains(MethodAccessFlags::NATIVE) {
			code = Some(MethodCode::Native(Either::Left((
				class_name.to_string(),
				ident.clone(),
			))));
		} else {
			for attribute in info.attribute_info {
				if let AttributeInfo::CodeAttribute { code: c } = attribute {
					code = Some(MethodCode::Java(c, Cell::new(None)));
				}
			}
		}

		Ok((
			ident.clone(),
			Method {
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
	Java(Code, Cell<Option<*const c_void>>),
	Native(Either<(String, MethodIdentifier), NativeCode>),
}

#[derive(Copy, Clone)]
pub struct NativeCode {
	pub func: *const c_void,
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
