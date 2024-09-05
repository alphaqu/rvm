use std::ops::Deref;

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
		let name = cp[info.name_index].to_string();
		let desc = cp[info.descriptor_index].as_str();
		let field_type = Type::parse(desc)?;

		Some(FieldData {
			name,
			ty: field_type,
			flags: info.access_flags,
		})
	}
}

#[derive(Clone)]
pub struct FieldLayout {
	pub fields_size: u32,
	pub reference_count: u16,
	pub statics: bool,
	fields: Storage<String, Field>,
}

impl FieldLayout {
	pub fn new_instance(fields: &[FieldData], super_fields: Option<&FieldLayout>) -> FieldLayout {
		FieldLayout::new(fields, super_fields, false)
	}

	pub fn new_static(fields: &[FieldData]) -> FieldLayout {
		FieldLayout::new(fields, None, true)
	}

	fn new(
		fields: &[FieldData],
		super_fields: Option<&FieldLayout>,
		is_static: bool,
	) -> FieldLayout {
		let mut output: Vec<(usize, Field, String)> = vec![];
		if let Some(fields) = super_fields {
			assert_eq!(fields.statics, is_static);
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
			if static_field != is_static {
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

		FieldLayout {
			fields_size,
			reference_count: ref_fields as u16,
			statics: is_static,
			fields: storage,
		}
	}

	pub fn len(&self) -> usize {
		self.fields.len()
	}
}

impl Deref for FieldLayout {
	type Target = Storage<String, Field>;

	fn deref(&self) -> &Self::Target {
		&self.fields
	}
}

impl StorageValue for Field {
	type Idx = u16;
}

#[derive(Clone)]
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
