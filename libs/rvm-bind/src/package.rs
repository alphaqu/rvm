use crate::class::ClassFile;
use std::collections::HashMap;

#[derive(Default)]
pub struct Package {
	pub full_name: String,
	pub packages: HashMap<String, Package>,
	pub files: Vec<ClassFile>,
}

impl Package {
	pub fn insert(&mut self, data: ClassFile) {
		let name = (*data.data.ty).to_string();

		let path_parts: Vec<String> = name.split("/").map(|v| v.to_string()).collect();

		self.insert_raw(path_parts, data);
	}

	fn insert_raw(&mut self, mut parts: Vec<String>, data: ClassFile) {
		if parts.len() == 1 {
			self.files.push(data);
		} else {
			let package = parts.remove(0);
			let package_name = format!("{}/{package}", self.full_name)
				.trim_start_matches("/")
				.to_string();
			let node = self.packages.entry(package).or_default();
			node.full_name = package_name;
			node.insert_raw(parts, data);
		}
	}
}

pub enum PackageEntry {
	Package(Package),
	File(ClassFile),
}
