use crate::ClassMethods;
use eyre::ContextCompat;
use rvm_core::{FieldAccessFlags, Storage, StorageValue, Type};
use rvm_reader::{ConstantPool, FieldInfo};
use std::ops::{Deref, DerefMut};

pub struct ClassFields {
	storage: Storage<String, Field>,
}

impl ClassFields {
	//	fn resolve_fields(info: &ClassInfo, _cl: &mut ClassResolver) -> eyre::Result<Vec<FieldData>> {
	// 		info.fields
	// 			.iter()
	// 			.map(|v| FieldData::from_info(v, &info.cp).wrap_err("Failed to parse descriptor"))
	// 			.try_collect()
	// 	}
	pub fn empty() -> Self {
		Self {
			storage: Storage::new(),
		}
	}

	pub fn new(fields: Vec<Field>) -> Self {
		let mut storage = Storage::new();
		for field in fields {
			storage.insert(field.name.clone(), field);
		}

		Self { storage }
	}

	pub fn parse(fields: &[FieldInfo], cp: &ConstantPool) -> eyre::Result<Self> {
		let mut output = Vec::new();
		for info in fields {
			let name = info.name_index.get(cp).unwrap().as_str();
			let field =
				Field::parse(&info, cp).wrap_err_with(|| format!("in FIELD \"{}\"", name))?;
			output.push(field);
		}
		Ok(Self::new(output))
	}
}

impl Deref for ClassFields {
	type Target = Storage<String, Field>;

	fn deref(&self) -> &Self::Target {
		&self.storage
	}
}
impl DerefMut for ClassFields {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.storage
	}
}
pub struct Field {
	pub name: String,
	pub ty: Type,
	pub flags: FieldAccessFlags,
}

impl Field {
	pub fn parse(info: &FieldInfo, cp: &ConstantPool) -> Option<Field> {
		let name = cp[info.name_index].to_string();
		let desc = cp[info.descriptor_index].as_str();
		let field_type = Type::parse(desc)?;

		Some(Field {
			name,
			ty: field_type,
			flags: info.access_flags,
		})
	}

	pub fn is_static(&self) -> bool {
		self.flags.contains(FieldAccessFlags::STATIC)
	}
}

impl StorageValue for Field {
	type Idx = u16;
}
