use std::ops::Deref;

use ahash::HashMapExt;
use nom::ToUsize;

use rvm_core::{FieldAccessFlags, Type};
use rvm_core::{Storage, StorageValue};
use rvm_reader::{ConstantPool, FieldInfo};

pub struct FieldData {
	pub name: String,
	pub ty: Type,
	pub flags: FieldAccessFlags,
}

impl FieldData {
	pub fn from_info(info: &FieldInfo, cp: &ConstantPool) -> Option<FieldData> {
		let name = info.name_index.get(cp).unwrap().to_string();
		let desc = info.descriptor_index.get(cp).unwrap().as_str();
		let field_type = Type::parse(desc)?;

		Some(FieldData {
			name,
			ty: field_type,
			flags: info.access_flags,
		})
	}
}

pub struct ObjectFieldLayout {
	pub fields_size: u32,
	pub ref_fields: u16,
	fields: Storage<String, Field>,
}

impl ObjectFieldLayout {
	pub fn new(
		fields: &[FieldData],
		super_fields: Option<&ObjectFieldLayout>,
		static_layout: bool,
	) -> ObjectFieldLayout {
		let mut output: Vec<(usize, Field, String)> = vec![];
		if let Some(fields) = super_fields {
			for (id, name, field) in fields.fields.iter_keys_unordered() {
				output.push((
					id.idx().to_usize(),
					Field {
						offset: 0,
						flags: field.flags,
						ty: field.ty.clone(),
					},
					name.clone(),
				));
			}
		}

		for field in fields {
			let static_field = field.flags.contains(FieldAccessFlags::STATIC);
			if static_field != static_layout {
				continue;
			}

			let i = output.len();
			output.push((
				i + 1,
				Field {
					offset: 0,
					flags: field.flags,
					ty: field.ty.clone(),
				},
				field.name.clone(),
			));
		}

		// Sort it so that it follows the order of super field ids
		output.sort_by(|(v0, _, _), (v1, _, _)| v0.cmp(v1));

		// Create offsets, ensure that all reference fields are first.
		let mut ref_fields = 0;
		let mut fields_size = 0;
		{
			let mut fields: Vec<&mut Field> = output.iter_mut().map(|(_, f, _)| f).collect();
			fields.sort_by(|v0, v1| v1.ty.kind().is_ref().cmp(&v0.ty.kind().is_ref()));

			for field in fields {
				field.offset = fields_size;

				let kind = field.ty.kind();
				if kind.is_ref() {
					ref_fields += 1;
				}
				fields_size += kind.size() as u32;
			}
		}

		let mut storage = Storage::new();
		for (_, field, name) in output.into_iter() {
			storage.insert(name, field);
		}

		ObjectFieldLayout {
			fields_size,
			ref_fields: ref_fields as u16,
			fields: storage,
		}
	}
}

impl Deref for ObjectFieldLayout {
	type Target = Storage<String, Field>;

	fn deref(&self) -> &Self::Target {
		&self.fields
	}
}

impl StorageValue for Field {
	type Idx = u16;
}

pub struct Field {
	pub offset: u32,
	pub flags: FieldAccessFlags,
	pub ty: Type,
}

impl Field {
	pub fn is_static(&self) -> bool {
		self.flags.contains(FieldAccessFlags::STATIC)
	}
}
