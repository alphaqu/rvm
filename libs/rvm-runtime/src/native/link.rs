use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::ffi::{CString, OsStr};
use std::path::Path;

#[cfg(unix)]
use libloading::os::unix as imp;
#[cfg(windows)]
use libloading::os::windows as imp;
use libloading::Library;
use tracing::{debug, trace};

pub struct JNILinker {
	libraries: HashMap<String, Library>,
	cache: HashMap<String, imp::Symbol<extern "system" fn()>>,
}

impl JNILinker {
	pub fn new() -> JNILinker {
		JNILinker {
			libraries: Default::default(),
			cache: Default::default(),
		}
	}
	pub fn link<P: AsRef<Path>>(&mut self, file: P) {
		unsafe {
			let str = file.as_ref().to_str().unwrap().to_string();
			let library = Library::new(file.as_ref()).unwrap();
			self.libraries.insert(str, library);
		}
	}

	pub fn get<V>(&mut self, name: &str, func: impl FnOnce(extern "system" fn()) -> V) -> V {
		let symbol = match self.cache.entry(name.to_string()) {
			Entry::Occupied(entry) => entry.into_mut(),
			Entry::Vacant(entry) => unsafe {
				'load: {
					debug!("Linking native method {name}");
					for (lib_name, library) in &self.libraries {
						trace!("Checking {lib_name}");
						let string = CString::new(name).unwrap();

						if let Ok(value) = library.get::<extern "system" fn()>(string.as_bytes()) {
							let symbol = value.into_raw();
							break 'load entry.insert(symbol);
						}
					}

					panic!()
				}
			},
		};

		func(**symbol)
	}
}
