use crate::reference::Ref;
use crate::ty::Type;
use crate::ty::ValueEnum;
use rvm_consts::FieldAccessFlags;
use rvm_core::{Id, Ref, Storage, StorageValue, Type, ValueEnum};

pub struct ObjectDesc {
	size: usize,
	fields: Storage<String, ObjectField>,
}

impl ObjectDesc {
	pub fn new(values: Vec<(FieldAccessFlags, String, Type)>) -> ObjectDesc {
		let mut fields = Storage::new();
		let mut size = 0;
		for (flags, name, ty) in values {
			let field_size = ty.kind().size();
			fields.insert(
				name,
				ObjectField {
					offset: size,
					flags,
					ty,
				},
			);
			size += field_size;
		}

		ObjectDesc { size, fields }
	}
    
    pub fn size(&self) -> usize {
        self.size
    }
    
    pub fn fields(&self) -> &Storage<String, ObjectField> {
        &self.fields
    }
}

pub struct Object<'a> {
	reference: Ref,
	desc: &'a ObjectDesc,
}

impl<'a> Object<'a> {
	pub unsafe fn new(reference: Ref, desc: &'a ObjectDesc) -> Object<'a> {
		Object { reference, desc }
	}

	pub fn get_field(&self, id: Id<ObjectField>) -> ValueEnum {
		let field = self.desc.fields.get(id);
        unsafe {
            field.ty.kind().read(self.reference.ptr().add(field.offset))
        }
	}

    pub fn set_field(&self, id: Id<ObjectField>, value: ValueEnum) {
        let field = self.desc.fields.get(id);
        unsafe {
            field.ty.kind().write(self.reference.ptr().add(field.offset), value)
        }
    }
}

pub struct ObjectField {
	pub offset: usize,
	pub flags: FieldAccessFlags,
	pub ty: Type,
}

impl StorageValue for ObjectField {
	type Idx = u32;
}
