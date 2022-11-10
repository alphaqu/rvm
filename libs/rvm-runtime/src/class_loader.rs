use crate::class::{ArrayClass, ObjectClass};
use crate::{Class, ClassInfo, ClassKind, MethodIdentifier, NativeCode};
use ahash::AHashMap;
use anyways::audit::Audit;
use anyways::ext::AuditExt;
use parking_lot::lock_api::MappedRwLockReadGuard;
use parking_lot::{RawRwLock, RwLock, RwLockReadGuard};
use rvm_core::{Id, Kind, ObjectType, Storage, Type};
use std::io::{Cursor, Read};
use tracing::{debug, info, instrument, warn};

pub struct ClassLoader {
	classes: RwLock<Storage<Type, Class>>,
	native_methods: AHashMap<(String, MethodIdentifier), NativeCode>,
}

impl ClassLoader {
	pub fn new() -> ClassLoader {
		ClassLoader {
			classes: RwLock::new(Storage::new()),
			native_methods: AHashMap::new(),
		}
	}

	pub fn get(&self, id: Id<Class>) -> MappedRwLockReadGuard<'_, RawRwLock, Class> {
		RwLockReadGuard::map(self.classes.read(), |v| v.get(id))
	}

	pub fn classes(&self) -> MappedRwLockReadGuard<'_, RawRwLock, [Class]> {
		RwLockReadGuard::map(self.classes.read(), |v| v.iter())
	}

	#[deprecated]
	pub fn get_obj_class(
		&self,
		id: Id<Class>,
	) -> MappedRwLockReadGuard<'_, RawRwLock, ObjectClass> {
		MappedRwLockReadGuard::map(self.get(id), |v| match &v.kind {
			ClassKind::Object(class) => class,
			_ => {
				panic!("why")
			}
		})
	}

	pub fn scope_class<R>(&self, id: Id<Class>, func: impl FnOnce(&ObjectClass) -> R) -> R {
		let guard = self.get(id);
		match &guard.kind {
			ClassKind::Object(class) => func(class),
			_ => {
				panic!("why")
			}
		}
	}

	pub fn get_class_id(&self, desc: &Type) -> Id<Class> {
		// if its in the match the lock wont get dropped
		let option = self.classes.read().get_id(desc);
		match option {
			Some(value) => value,
			None => {
				info!("defining class {desc}");
				let kind = match desc {
					Type::Primitive(_) => {
						panic!("primitive?!?!??!?!")
					}
					Type::Object(object) => {
						panic!("CLASS NOT LOADED {object:?}, Cannot load classes while running... yet.");
					}
					Type::Array(value) => {
						if let Kind::Reference = value.component.kind() {
							// ensure loaded
							self.get_class_id(&value.component);
						}

						ClassKind::Array(ArrayClass::new(value.component.clone()))
					}
				};

				self.define(
					desc.clone(),
					Class {
						name: desc.to_string(),
						kind,
					},
				)
				
			}
		}
	}

	/// Forcefully loads all classes in a jar. This is used only in bootstrapping the java standard library.
	pub fn load_jar(&self, data: &[u8], filter: impl Fn(&str) -> bool) -> anyways::Result<()> {
		let reader = Cursor::new(data);
		let mut archive = zip::read::ZipArchive::new(reader)?;
		let map: Vec<String> = archive.file_names().map(|v| v.to_string()).collect();
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
		let (_, info) =
			ClassInfo::parse(data).map_err(|_| Audit::new("Failed to parse classfile"))?;
		let class = ObjectClass::parse(info, self)?;

		debug!("Parsed class {}", class.name);

		Ok(self.define(Type::Object(ObjectType {
			name: class.name.replace(".", "/")
		}), class))
	}

	fn define(&self, ty: Type, class: Class) -> Id<Class> {
		debug!("Inject and defining new class {ty:?}");
		if self.classes.is_locked() {
			warn!("Classes are locked");
		}
		self.classes.try_write().unwrap().insert(ty, class)
	}

	pub fn is_locked(&self) -> bool {
		self.classes.is_locked()
	}

	pub fn register_native(
		&mut self,
		class_name: String,
		method: MethodIdentifier,
		code: NativeCode,
	) {
		self.native_methods.insert((class_name, method), code);
	}

	pub fn native_methods(&self) -> &AHashMap<(String, MethodIdentifier), NativeCode> {
		&self.native_methods
	}
}
