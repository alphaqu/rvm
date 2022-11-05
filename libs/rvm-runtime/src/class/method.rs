use std::ops::Deref;

use anyways::ext::AuditExt;
use anyways::Result;

use rvm_core::Storage;

use crate::reader::{ConstantPool, MethodInfo};
use crate::{Method, MethodIdentifier};

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
