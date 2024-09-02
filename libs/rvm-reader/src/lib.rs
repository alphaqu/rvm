#![feature(let_chains)]
//! rvm-reader is responsible for parsing .class files.
#![allow(dead_code)]

pub use attribute::*;
pub use class::*;
pub use code::*;
pub use consts::*;
pub use field::*;
pub use method::*;

use crate::error::ParsingError;

mod attribute;
mod class;
mod code;
mod consts;
mod error;
mod field;
mod method;

pub type IResult<'a, O> = nom::IResult<&'a [u8], O, ParsingError<'a>>;
