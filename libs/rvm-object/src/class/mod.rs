mod object;
mod array;

use rvm_core::{PrimitiveType, StorageValue};

pub use object::*;
pub use array::*;

pub struct Class {
    pub name: String,
    pub kind: ClassKind,
}

impl Class {}

pub enum ClassKind {
    Object(ObjectClass),
    Array(ArrayClass),
    Primitive(PrimitiveType),
}

impl ClassKind {
   ///// Gets the object size excluding the header
   //pub fn obj_size(&self, object: Ref) -> usize {
   //    match self {
   //        ClassKind::Object(class) => class.size(false),
   //        ClassKind::Array(class) => class.size(object),
   //        ClassKind::Primitive(desc) => {
   //            panic!("no")
   //        }
   //    }
   //}
}

impl StorageValue for Class {
    type Idx = u32;
}