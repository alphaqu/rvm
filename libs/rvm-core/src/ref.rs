#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct Ref {
    data: *mut u8,
}

impl Ref {
    pub unsafe fn new(data: *mut u8) -> Ref {
        Ref {
            data
        }
    }

    pub fn ptr(&self) -> *mut u8 {
        self.data
    }
}