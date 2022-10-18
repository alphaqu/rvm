use crate::error::JError;
use crate::{Class, JResult, Runtime};
use base64::{Config, encode, encode_config};
use rvm_core::Id;
use std::alloc::{alloc_zeroed, Layout};
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Deref, Sub};
use std::ptr::{null_mut, read, write};
use base64::display::Base64Display;

mod field;
mod method;
mod value;

pub use field::Field;
pub use method::Method;
pub use method::MethodCode;
pub use method::MethodIdentifier;
pub use method::NativeCode;
pub use value::Type;
pub use value::Value;
pub use value::ValueType;

// 4 for class and 1 for flag
pub const HEADER_SIZE: usize = 5;
pub const ALIGN: usize = 32;

#[derive(Hash, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
#[repr(transparent)]
pub struct Ref(ObjectData);

impl Ref {
    pub fn null() -> Ref {
        Ref(ObjectData(null_mut()))
    }

    pub unsafe fn new_ptr(ptr: *mut u8) -> Ref {
        Ref(ObjectData(ptr))
    }

    /// # Safety
    /// 60% of the time
    pub unsafe fn new(mark: bool, class: u32, size: usize) -> Ref {
        let mut ptr = ObjectData::new(HEADER_SIZE + size);
        write(ptr.0, mark as u8);
        let bytes: [u8; 4] = class.to_le_bytes();
        write(ptr.0.offset(1), bytes[0]);
        write(ptr.0.offset(2), bytes[1]);
        write(ptr.0.offset(3), bytes[2]);
        write(ptr.0.offset(4), bytes[3]);

        // for future operations
        ptr.0 = ptr.0.add(HEADER_SIZE);
        Ref(ptr)
    }

    pub fn get_class(&self) -> Id<Class> {
        unsafe {
            let id = u32::from_le_bytes(ObjectData::read(self.0 .0.sub(HEADER_SIZE - 1)));
            Id::new(id as usize)
        }
    }

    pub fn set_mark(&self, flag: bool) {
        unsafe { write(self.0 .0.sub(HEADER_SIZE), flag as u8) }
    }

    pub fn get_mark(&self) -> bool {
        unsafe {
            let flag = read(self.0 .0.sub(HEADER_SIZE));
            flag != 0
        }
    }

    pub fn assert_matching_class(&self, target: Id<Class>, runtime: &Runtime) -> JResult<()> {
        let this = self.get_class();

        if this != target {
            let this = runtime.cl.get(this);
            let target = runtime.cl.get(target);
            Err(JError::new(format!(
                "Expected class {} but found {}",
                &this.binary_name, &target.binary_name
            )))
        } else {
            Ok(())
        }
    }
}

impl Deref for Ref {
    type Target = ObjectData;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Ref {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let bytes = (self.0.0 as usize).to_le_bytes();
        let display = Base64Display::with_config(&bytes, base64::URL_SAFE_NO_PAD);
        write!(f, "{}", display)
    }
}

#[derive(Hash, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
#[repr(transparent)]
pub struct ObjectData(*mut u8);

impl ObjectData {
    /// # Safety
    /// sometimes
    pub unsafe fn new(size: usize) -> ObjectData {
        ObjectData(if size == 0 {
            null_mut()
        } else {
            alloc_zeroed(ObjectData::layout(HEADER_SIZE + size))
        })
    }

    pub fn is_null(&self) -> bool {
        self.0.is_null()
    }

    /// # Safety
    /// not
    pub unsafe fn ptr(&self) -> *mut u8 {
        self.0
    }

    //  pub fn set_field(&self, field: &Field, value: Value) {
    //         unsafe {
    //             let field_ptr = self.0.add(field.offset as usize);
    //
    //             match value {
    //                 Value::Boolean(boolean) => {
    //                     write(field_ptr, boolean as u8);
    //                 }
    //                 Value::Byte(value) => Self::write(field_ptr, value.to_le_bytes()),
    //                 Value::Short(value) => Self::write(field_ptr, value.to_le_bytes()),
    //                 Value::Int(value) => Self::write(field_ptr, value.to_le_bytes()),
    //                 Value::Long(value) => Self::write(field_ptr, value.to_le_bytes()),
    //                 Value::Char(value) => Self::write(field_ptr, value.to_le_bytes()),
    //                 Value::Float(value) => Self::write(field_ptr, value.to_le_bytes()),
    //                 Value::Double(value) => Self::write(field_ptr, value.to_le_bytes()),
    //                 Value::Reference(value) => Self::write(field_ptr, {
    //                     let i = value.0 .0 as usize;
    //                     i.to_le_bytes()
    //                 }),
    //             }
    //         }
    //     }
    //
    //     pub fn get_field(&self, field: &Field) -> Value {
    //         unsafe {
    //             let field_ptr = self.0.add(field.offset as usize);
    //             match field.desc {
    //                 FieldDesc::Boolean => Value::Boolean(read(field_ptr) != 0),
    //                 FieldDesc::Byte => Value::Byte(i8::from_le_bytes(Self::read(field_ptr))),
    //                 FieldDesc::Short => Value::Short(i16::from_le_bytes(Self::read(field_ptr))),
    //                 FieldDesc::Int => Value::Int(i32::from_le_bytes(Self::read(field_ptr))),
    //                 FieldDesc::Long => Value::Long(i64::from_le_bytes(Self::read(field_ptr))),
    //                 FieldDesc::Char => Value::Char(u16::from_le_bytes(Self::read(field_ptr))),
    //                 FieldDesc::Float => Value::Float(f32::from_le_bytes(Self::read(field_ptr))),
    //                 FieldDesc::Double => Value::Double(f64::from_le_bytes(Self::read(field_ptr))),
    //                 FieldDesc::Object(_) | FieldDesc::Array(_) => Value::Reference(Object(ObjectData(
    //                     usize::from_le_bytes(Self::read(field_ptr)) as *mut u8,
    //                 ))),
    //             }
    //         }
    //     }

    #[inline(always)]
    pub fn layout(size: usize) -> Layout {
        Layout::from_size_align(size, ALIGN).unwrap()
    }

    #[inline(always)]
    unsafe fn read<const C: usize>(ptr: *mut u8) -> [u8; C] {
        let mut out = [0; C];
        for i in 0..C {
            *out.get_unchecked_mut(i) = read(ptr.add(i));
        }
        out
    }

    #[inline(always)]
    unsafe fn write<const C: usize>(ptr: *mut u8, value: [u8; C]) {
        for i in 0..C {
            write(ptr.add(i), *value.get_unchecked(i));
        }
    }
}
