#![allow(dead_code)]

mod attribute;
mod class;
mod code;
mod consts;
mod field;
mod method;

pub use attribute::*;
pub use class::*;
pub use code::*;
pub use consts::*;
pub use field::*;
pub use method::*;

pub type IResult<'a, O> = nom::IResult<&'a [u8], O>;
