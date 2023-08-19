extern crate proc_macro;

use proc_macro::TokenStream;
use std::fmt::Write;

use syn::{Ident, parenthesized, parse, token, Token};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;

//macro_rules! java_descriptor {
// 	(()) => {"V"};
// 	(bool) => {"Z"};
// 	(i8) => {"B"};
// 	(i16) => {"S"};
// 	(i32) => {"I"};
// 	(f32) => {"F"};
// 	(i64) => {"J"};
// 	(f64) => {"D"};
// 	(Array<$param:tt>) => {
// 		::core::concat!("[", $crate::java_descriptor!($param))
// 	};
// 	(fn($($param:tt),*)) => {
// 		$crate::java_descriptor!(fn($($param),*) -> ())
// 	};
// 	(fn($($param:tt),*) -> $ret:tt) => {
// 		::core::concat!("(", $($crate::java_descriptor!($param),)* ")", $crate::java_descriptor!($ret))
// 	};

enum Desc {
	Prim(PrimDesc),
	Array(ArrayDesc),
	Func(FuncDesc),
}

impl Parse for Desc {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		if input.peek(Token!(fn)) {
			Ok(Desc::Func(input.parse()?))
		} else if input.peek2(Token!(<)) {
			Ok(Desc::Array(input.parse()?))
		} else {
			Ok(Desc::Prim(input.parse()?))
		}
	}
}

impl Desc {
	pub fn export(&self) -> String {
		match self {
			Desc::Prim(v) => v.export(),
			Desc::Array(v) => v.export(),
			Desc::Func(v) => v.export(),
		}
	}
}

enum PrimDesc {
	Void,
	Reference,
	Char,
	Bool,
	I8,
	I16,
	I32,
	I64,
	F32,
	F64,
}

impl Parse for PrimDesc {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let ident: Ident = input.parse()?;
		Ok(match ident.to_string().as_str() {
			"bool" => PrimDesc::Bool,
			"Reference" => PrimDesc::Reference,
			"char" => PrimDesc::Char,
			"i8" => PrimDesc::I8,
			"i16" => PrimDesc::I16,
			"i32" => PrimDesc::I32,
			"f32" => PrimDesc::I64,
			"i64" => PrimDesc::F32,
			"f64" => PrimDesc::F64,
			_ => panic!(),
		})
	}
}

impl PrimDesc {
	pub fn export(&self) -> String {
		match self {
			PrimDesc::Void => "V".to_string(),
			PrimDesc::Char => "C".to_string(),
			PrimDesc::Bool => "Z".to_string(),
			PrimDesc::I8 => "B".to_string(),
			PrimDesc::I16 => "S".to_string(),
			PrimDesc::I32 => "I".to_string(),
			PrimDesc::I64 => "J".to_string(),
			PrimDesc::F32 => "F".to_string(),
			PrimDesc::F64 => "D".to_string(),
			PrimDesc::Reference => "Ljava/lang/Object;".to_string()
		}
	}
}

struct ArrayDesc {
	pub lt_token: Token![<],
	pub element: Box<Desc>,
	pub gt_token: Token![>],
}

impl Parse for ArrayDesc {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let ident: Ident = input.parse()?;
		if ident.to_string() != "Array" {
			panic!();
		}

		Ok(ArrayDesc {
			lt_token: input.parse()?,
			element: input.parse()?,
			gt_token: input.parse()?,
		})
	}
}

impl ArrayDesc {
	pub fn export(&self) -> String {
		format!("[{}", self.element.export())
	}
}

struct FuncDesc {
	pub token: Token!(fn),
	pub paren_token: token::Paren,
	pub parameters: Punctuated<Desc, Token![,]>,
	pub output: FuncReturn,
}

impl FuncDesc {
	pub fn export(&self) -> String {
		let mut out = String::new();
		out.write_char('(').unwrap();
		for desc in &self.parameters {
			out.write_str(&desc.export()).unwrap();
		}
		out.write_char(')').unwrap();
		match &self.output {
			FuncReturn::Type(_, value) => {
				out.write_str(&value.export()).unwrap();
			}
			FuncReturn::Default => {
				out.write_char('V').unwrap();
			}
		}
		out
	}
}

impl Parse for FuncDesc {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let content;
		Ok(FuncDesc {
			token: input.parse()?,
			paren_token: parenthesized!(content in input),
			parameters: content.parse_terminated(Desc::parse, Token![,])?,
			output: input.parse()?,
		})
	}
}

enum FuncReturn {
	Default,
	Type(Token![->], Box<Desc>),
}

impl Parse for FuncReturn {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		if input.peek(Token![->]) {
			Ok(FuncReturn::Type(input.parse()?, input.parse()?))
		} else {
			Ok(FuncReturn::Default)
		}
	}
}

#[proc_macro]
pub fn java_desc(item: TokenStream) -> TokenStream {
	let desc: Desc = parse(item).unwrap();
	format!("\"{}\"", desc.export()).parse().unwrap()
}
