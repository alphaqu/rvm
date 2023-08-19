#![allow(dead_code)]

use nom::error::VerboseError;

pub use attribute::*;
pub use class::*;
pub use code::*;
pub use consts::*;
pub use field::*;
pub use method::*;

mod attribute;
mod class;
mod code;
mod consts;
mod field;
mod method;

pub type IResult<'a, O> = nom::IResult<&'a [u8], O, VerboseError<&'a [u8]>>;
