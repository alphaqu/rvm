use base64::display::Base64Display;
use rvm_core::Id;
use std::alloc::{alloc_zeroed, Layout};
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use std::ptr::{null_mut, read, write};
use anyways::audit::Audit;
use crate::class::Class;
use crate::class_loader::ClassLoader;

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
        write_arr(ptr.0.offset(1), bytes);

        // for future operations
        ptr.0 = ptr.0.add(HEADER_SIZE);
        Ref(ptr)
    }

    pub fn get_class(&self) -> Id<Class> {
        unsafe {
            let id = u32::from_le_bytes(read_arr(self.0 .0.sub(HEADER_SIZE - 1)));
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

    pub fn assert_matching_class(&self, target: Id<Class>, cl: &ClassLoader) -> anyways::Result<()> {
        let this = self.get_class();

        if this != target {
            let this = cl.get(this);
            let target = cl.get(target);
            Err(Audit::new(format!(
                "Expected class {} but found {}",
                &this.name, &target.name
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
        let bytes = (self.0 .0 as usize).to_le_bytes();
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

    #[inline(always)]
    pub fn layout(size: usize) -> Layout {
        Layout::from_size_align(size, ALIGN).unwrap()
    }
}

#[inline(always)]
pub(crate) unsafe fn read_arr<const C: usize>(ptr: *mut u8) -> [u8; C] {
    let mut out = [0; C];
    for i in 0..C {
        *out.get_unchecked_mut(i) = read(ptr.add(i));
    }
    out
}

#[inline(always)]
pub(crate) unsafe fn write_arr<const C: usize>(ptr: *mut u8, value: [u8; C]) {
    for i in 0..C {
        write(ptr.add(i), *value.get_unchecked(i));
    }
}
