use std::cell::Cell;
use rvm_reader::{
	AttributeInfo, Code, ConstantPool, MethodInfo, NameAndTypeConst,
};
use crate::{JResult, Runtime};
use anyways::Result;
use either::Either;
use rvm_core::{MethodAccessFlags, MethodDesc};
use rvm_core::StorageValue;
use std::ffi::c_void;
use std::sync::Arc;

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
