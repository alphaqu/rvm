use crate::reader::{ConstantPool, FieldInfo, ValueDesc};
use crate::{BinaryName, ClassLoader, Field, StrParse};
use rvm_consts::FieldAccessFlags;
use rvm_core::{Id, Storage, StorageValue};
use std::ops::Deref;

pub struct ClassFieldManager {
    object_size: u32,
    static_size: u32,
    fields: Storage<String, Field>,
    object_fields: Vec<Id<Field>>,
    static_fields: Vec<Id<Field>>,
}

impl ClassFieldManager {
    pub fn parse(
        fields: Vec<FieldInfo>,
        cp: &ConstantPool,
        class_loader: &ClassLoader,
    ) -> ClassFieldManager {
        let mut out = Storage::new();

        let mut object_fields = Vec::new();
        let mut object_size = 0;
        let mut static_fields = Vec::new();
        let mut static_size = 0;
        for field in fields {
            let name = field.name_index.get(cp).to_string();

            let desc = field.descriptor_index.get(cp).as_str();
            let field_type = ValueDesc::parse(desc).unwrap();
            let static_field = field.access_flags.contains(FieldAccessFlags::STATIC);
            let object_field = matches!(field_type, ValueDesc::Object(_));

            if object_field {
                // ensure loaded
                class_loader.get_class_id(&BinaryName::parse(desc));
            }

            let ty = field_type.ty();
            let field_size = if static_field {
                let value = static_size;
                static_size += ty.size() as u32;
                value
            } else {
                let value = object_size;
                object_size += ty.size() as u32;
                value
            };

            let pos = out.insert(
                name,
                Field {
                    offset: field_size,
                    flags: field.access_flags,
                    desc: field_type,
                    ty,
                },
            );

            if object_field {
                if static_field {
                    &mut static_fields
                } else {
                    &mut object_fields
                }
                    .push(pos);
            }
        }

        ClassFieldManager {
            fields: out,
            object_size,
            object_fields,
            static_size,
            static_fields,
        }
    }

    pub fn object_fields(&self, static_obj: bool) -> &[Id<Field>] {
        if static_obj {
            &self.static_fields
        } else {
            &self.object_fields
        }
    }

    pub fn size(&self, static_obj: bool) -> u32 {
        if static_obj {
            self.static_size
        } else {
            self.object_size
        }
    }
}

impl Deref for ClassFieldManager {
    type Target = Storage<String, Field>;

    fn deref(&self) -> &Self::Target {
        &self.fields
    }
}

impl StorageValue for Field {
    type Idx = u16;
}
