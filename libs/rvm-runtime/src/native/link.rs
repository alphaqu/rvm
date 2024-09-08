use crate::MethodBinding;
use either::Either;
#[cfg(unix)]
use libloading::os::unix as imp;
#[cfg(windows)]
use libloading::os::windows as imp;
use libloading::Library;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::ffi::CString;
use std::panic::UnwindSafe;
use std::path::Path;
use tracing::{debug, trace};

pub struct JNILinker {
	libraries: HashMap<String, Library>,
	symbols: HashMap<String, JNISymbol>,
}

enum JNISymbol {
	Library(imp::Symbol<extern "C" fn()>),
	Rust(MethodBinding),
}

impl JNILinker {
	pub fn new() -> JNILinker {
		JNILinker {
			libraries: Default::default(),
			symbols: Default::default(),
		}
	}
	pub fn link_library<P: AsRef<Path>>(&mut self, file: P) {
		unsafe {
			let str = file.as_ref().to_str().unwrap().to_string();
			let library = Library::new(file.as_ref()).unwrap();
			self.libraries.insert(str, library);
		}
	}

	pub unsafe fn link(&mut self, name: String, func: MethodBinding) {
		self.symbols.insert(name, JNISymbol::Rust(func));
	}

	pub fn get<V>(
		&mut self,
		name: &str,
		func: impl FnOnce(Either<extern "C" fn(), &MethodBinding>) -> V,
	) -> Option<V> {
		let symbol = match self.symbols.entry(name.to_string()) {
			Entry::Occupied(entry) => entry.into_mut(),
			Entry::Vacant(entry) => unsafe {
				'load: {
					debug!("Linking native method {name}");
					for (lib_name, library) in &self.libraries {
						trace!("Checking {lib_name}");
						let string = CString::new(name).unwrap();

						if let Ok(value) = library.get::<extern "C" fn()>(string.as_bytes()) {
							let symbol = value.into_raw();
							break 'load entry.insert(JNISymbol::Library(symbol));
						}
					}

					return None;
				}
			},
		};

		Some(func(match symbol {
			JNISymbol::Library(symbol) => Either::Left(**symbol),
			JNISymbol::Rust(binding) => Either::Right(binding),
		}))
	}
}
