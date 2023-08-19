use std::io::{Cursor, Read};
use std::sync::Arc;

use anyways::ext::AuditExt;
use nom::error::VerboseErrorKind;
use parking_lot::RwLock;
use tracing::{debug, info, instrument, warn};

use crate::object::class::Class;
use crate::{ArrayClass, InstanceClass};
use rvm_core::{Id, Kind, Storage, Type};
use rvm_reader::ClassInfo;

pub struct ClassLoader {
	classes: RwLock<Storage<Type, Class, Arc<Class>>>,
}

impl ClassLoader {
	pub fn new() -> ClassLoader {
		ClassLoader {
			classes: RwLock::new(Storage::new()),
		}
	}

	pub fn get(&self, id: Id<Class>) -> Arc<Class> {
		self.classes.read().get(id).clone()
	}

	pub fn get_class_id(&self, desc: &Type) -> Id<Class> {
		info!("getting class {desc:?}");
		// if its in the match the lock wont get dropped
		let option = self.classes.read().get_id(desc);
		match option {
			Some(value) => value,
			None => {
				info!("defining class {desc:?}");
				let class = match desc {
					Type::Primitive(_) => {
						panic!("primitive?!?!??!?!")
					}
					Type::Object(object) => {
						for x in self.classes.read().iter() {
							info!("{:?}", x.cloned_ty());
						}
						panic!("CLASS NOT LOADED {object:?}, Cannot load classes while running... yet.");
					}
					Type::Array(value) => {
						if let Kind::Reference = value.component.kind() {
							// ensure loaded
							self.get_class_id(&value.component);
						}

						Class::Array(ArrayClass::new((*value.component).clone()))
					}
				};

				self.define(class)
			}
		}
	}

	/// Forcefully loads all classes in a jar. This is used only in bootstrapping the java standard library.
	pub fn load_jar(&self, data: &[u8], filter: impl Fn(&str) -> bool) -> anyways::Result<()> {
		let reader = Cursor::new(data);
		let mut archive = zip::read::ZipArchive::new(reader)?;
		let mut map: Vec<String> = archive.file_names().map(|v| v.to_string()).collect();
		map.sort();
		for name in map {
			let mut file = archive.by_name(&name)?;
			if file.is_file() && file.name().ends_with(".class") && filter(file.name()) {
				let mut data = Vec::with_capacity(file.size() as usize);
				file.read_to_end(&mut data)?;
				self.load_class(&data)
					.wrap_err_with(|| format!("Failed to load {}", file.name()))?;
			}
		}
		Ok(())
	}

	/// Loads a java class to the JVM and injects it to the class table by locking it.
	#[instrument(skip_all)]
	pub fn load_class(&self, data: &[u8]) -> anyways::Result<Id<Class>> {
		let info = match ClassInfo::parse(data) {
			Ok((_, info)) => info,
			Err(error) => {
				match error {
					nom::Err::Incomplete(e) => {}
					nom::Err::Failure(e) | nom::Err::Error(e) => {
						for (input, error) in e.errors {
							match error {
								VerboseErrorKind::Context(ctx) => {}
								VerboseErrorKind::Char(char) => {}
								VerboseErrorKind::Nom(nom) => {}
							}
							println!("{:?}", error);
						}
					}
				}
				panic!();
			}
		};
		let class = InstanceClass::parse(info, self)?;

		debug!("Parsed class {}", class.ty);

		Ok(self.define(Class::Object(class)))
	}

	pub fn define(&self, class: Class) -> Id<Class> {
		let ty = class.cloned_ty();
		debug!("Inject and defining new class {ty:?}");
		if self.classes.is_locked() {
			warn!("Classes are locked");
		}
		let id = self.classes.write().insert(ty.clone(), Arc::new(class));
		info!("Loaded class {ty:?} at {id:?}");
		id
	}
}
