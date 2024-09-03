use crate::InstanceClass;
use parking_lot::Mutex;
use rvm_core::ObjectType;
use std::collections::HashMap;
use std::fs::read;
use std::io::{Cursor, Read};
use std::path::PathBuf;
use std::sync::Arc;
use zip::ZipArchive;

pub trait ClassSource: Send + Sync {
	fn try_load(&self, ty: &ObjectType) -> eyre::Result<Option<Vec<u8>>>;
}
impl<S: ClassSource> ClassSource for Arc<S> {
	fn try_load(&self, ty: &ObjectType) -> eyre::Result<Option<Vec<u8>>> {
		(**self).try_load(ty)
	}
}

pub struct DirectoryClassSource {
	dir: PathBuf,
}

impl DirectoryClassSource {
	pub fn new(dir: PathBuf) -> eyre::Result<DirectoryClassSource> {
		Ok(DirectoryClassSource { dir })
	}
}

impl ClassSource for DirectoryClassSource {
	fn try_load(&self, ty: &ObjectType) -> eyre::Result<Option<Vec<u8>>> {
		let mut path = self.dir.join(PathBuf::from(&**ty));
		path.set_extension("class");

		if path.exists() {
			let vec = read(path)?;
			return Ok(Some(vec));
		}

		Ok(None)
	}
}

pub struct JarClassSource {
	file_lookup: HashMap<String, usize>,
	archive: Mutex<ZipArchive<Cursor<Vec<u8>>>>,
}

impl JarClassSource {
	pub fn new(data: Vec<u8>) -> eyre::Result<JarClassSource> {
		let reader = Cursor::new(data);
		let mut archive: ZipArchive<Cursor<Vec<u8>>> = ZipArchive::new(reader)?;
		let mut map: Vec<String> = archive.file_names().map(|v| v.to_string()).collect();
		map.sort();
		let mut file_lookup = HashMap::new();
		for name in map {
			let file_index = archive.index_for_name(&name).unwrap();
			let file = archive.by_name(&name)?;

			let file_name = file.name();
			if file.is_file() && file_name.ends_with(".class") {
				file_lookup.insert(file_name.trim_end_matches(".class").to_string(), file_index);
				//let mut data = Vec::with_capacity(file.size() as usize);
				//file.read_to_end(&mut data)?;
				//self.load_class(&data)
				//    .wrap_err_with(|| format!("Failed to load {}", file_name))?;
			}
		}

		Ok(JarClassSource {
			file_lookup,
			archive: Mutex::new(archive),
		})
	}
}

impl ClassSource for JarClassSource {
	fn try_load(&self, ty: &ObjectType) -> eyre::Result<Option<Vec<u8>>> {
		if let Some(file_location) = self.file_lookup.get(&**ty) {
			let mut guard = self.archive.lock();
			let mut file = guard.by_index(*file_location)?;

			let mut data = Vec::with_capacity(file.size() as usize);
			file.read_to_end(&mut data)?;
			return Ok(Some(data));
		}

		Ok(None)
	}
}
