use std::ops::Deref;

use ahash::{HashMap, HashMapExt};

use rvm_core::{FieldAccessFlags, Type};
use rvm_core::{Id, Storage, StorageValue};
use rvm_reader::{ConstantPool, FieldInfo};

pub struct ObjectFieldLayoutBuilder {}

pub struct FieldData {
	pub name: String,
	pub ty: Type,
	pub flags: FieldAccessFlags,
}

impl FieldData {
	pub fn from_info(info: &FieldInfo, cp: &ConstantPool) -> Option<FieldData> {
		let name = info.name_index.get(cp).to_string();
		let desc = info.descriptor_index.get(cp).as_str();
		let field_type = Type::parse(desc)?;

		Some(FieldData {
			name,
			ty: field_type,
			flags: info.access_flags,
		})
	}
}

pub struct ObjectFieldLayout {
	pub size: u32,
	pub ref_fields: u16,
	fields: Storage<String, Field>,
}

impl ObjectFieldLayout {
	pub fn new(fields: &[FieldData], static_layout: bool) -> ObjectFieldLayout {
		let mut output = vec![];
		for field in fields {
			let static_field = field.flags.contains(FieldAccessFlags::STATIC);

			if static_field != static_layout {
				continue;
			}

			output.push((
				Field {
					offset: 0,
					flags: field.flags,
					ty: field.ty.clone(),
				},
				field.name.clone(),
			));
		}

		// Sort fields so that reference fields are first.
		output.sort_by(|(v0, _), (v1, _)| v1.ty.kind().is_ref().cmp(&v0.ty.kind().is_ref()));

		let mut size = 0;
		let mut ref_fields = 0;
		let mut storage = Storage::new();
		for (mut field, name) in output.into_iter() {
			field.offset = size;
			if field.ty.kind().is_ref() {
				ref_fields += 1;
			}
			size += field.ty.kind().size() as u32;
			storage.insert(name, field);
		}

		ObjectFieldLayout {
			size: size,
			ref_fields: ref_fields as u16,
			fields: storage,
		}
	}
}

//pub struct ClassFieldManager {
// 	instance: ObjectFieldLayout,
// 	static_fields
// 	object_size: u32,
// 	static_size: u32,
// 	fields: Storage<String, Field>,
// 	object_fields: Vec<Id<Field>>,
// 	static_fields: Vec<Id<Field>>,
// }
//
// impl ClassFieldManager {
// 	pub fn parse(
// 		fields: Vec<FieldInfo>,
// 		cp: &ConstantPool,
// 		//class_loader: &ClassLoader,
// 	) -> ClassFieldManager {
// 		let mut out = Storage::new();
//
// 		let mut object_fields = Vec::new();
// 		let mut object_size = 0;
// 		let mut static_fields = Vec::new();
// 		let mut static_size = 0;
// 		for field in fields {
// 			let name = field.name_index.get(cp).to_string();
//
// 			let desc = field.descriptor_index.get(cp).as_str();
// 			let field_type = Type::parse(desc).unwrap();
// 			let static_field = field.access_flags.contains(FieldAccessFlags::STATIC);
// 			let object_field = matches!(field_type, Type::Object(_));
//
// 			//if object_field {
// 			//	// ensure loaded
// 			//	class_loader.get_class_id(&field_type);
// 			//}
//
// 			let ty = field_type.kind();
// 			let field_size = if static_field {
// 				let value = static_size;
// 				static_size += ty.size() as u32;
// 				value
// 			} else {
// 				let value = object_size;
// 				object_size += ty.size() as u32;
// 				value
// 			};
//
// 			let pos = out.insert(
// 				name,
// 				Field {
// 					offset: field_size,
// 					flags: field.access_flags,
// 					ty: field_type,
// 				},
// 			);
//
// 			if object_field {
// 				if static_field {
// 					&mut static_fields
// 				} else {
// 					&mut object_fields
// 				}
// 				.push(pos);
// 			}
// 		}
//
// 		ClassFieldManager {
// 			fields: out,
// 			object_size,
// 			object_fields,
// 			static_size,
// 			static_fields,
// 		}
// 	}
//
// 	pub fn object_fields(&self, static_obj: bool) -> &[Id<Field>] {
// 		if static_obj {
// 			&self.static_fields
// 		} else {
// 			&self.object_fields
// 		}
// 	}
//
// 	pub fn size(&self, static_obj: bool) -> u32 {
// 		if static_obj {
// 			self.static_size
// 		} else {
// 			self.object_size
// 		}
// 	}
// }
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
