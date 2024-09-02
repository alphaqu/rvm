extern crate proc_macro;

use proc_macro::TokenStream;
use std::fmt::Write;

use syn::__private::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parenthesized, parse, token, Token, Type};

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

//enum PrimDesc {
// 	Void,
// 	Reference,
// 	Char,
// 	Bool,
// 	I8,
// 	I16,
// 	I32,
// 	I64,
// 	F32,
// 	F64,
// }
//
// impl Parse for PrimDesc {
// 	fn parse(input: ParseStream) -> syn::Result<Self> {
// 		let ident: Ident = input.parse()?;
// 		Ok(match ident.to_string().as_str() {
// 			"bool" => PrimDesc::Bool,
// 			"Reference" => PrimDesc::Reference,
// 			"char" => PrimDesc::Char,
// 			"i8" => PrimDesc::I8,
// 			"i16" => PrimDesc::I16,
// 			"i32" => PrimDesc::I32,
// 			"f32" => PrimDesc::F32,
// 			"i64" => PrimDesc::I64,
// 			"f64" => PrimDesc::F64,
// 			_ => panic!(),
// 		})
// 	}
// }
//
// impl PrimDesc {
// 	pub fn export(&self) -> String {
// 		match self {
// 			PrimDesc::Void => "V".to_string(),
// 			PrimDesc::Char => "C".to_string(),
// 			PrimDesc::Bool => "Z".to_string(),
// 			PrimDesc::I8 => "B".to_string(),
// 			PrimDesc::I16 => "S".to_string(),
// 			PrimDesc::I32 => "I".to_string(),
// 			PrimDesc::I64 => "J".to_string(),
// 			PrimDesc::F32 => "F".to_string(),
// 			PrimDesc::F64 => "D".to_string(),
// 			PrimDesc::Reference => "Ljava/lang/Object;".to_string(),
// 		}
// 	}
// }
//
// struct ArrayDesc {
// 	pub lt_token: Token![<],
// 	pub element: Box<Desc>,
// 	pub gt_token: Token![>],
// }
//
// impl Parse for ArrayDesc {
// 	fn parse(input: ParseStream) -> syn::Result<Self> {
// 		let ident: Ident = input.parse()?;
// 		if ident.to_string() != "Array" {
// 			panic!();
// 		}
//
// 		Ok(ArrayDesc {
// 			lt_token: input.parse()?,
// 			element: input.parse()?,
// 			gt_token: input.parse()?,
// 		})
// 	}
// }
//
// impl ArrayDesc {
// 	pub fn export(&self) -> String {
// 		format!("[{}", self.element.export())
// 	}
// }
struct TypedDesc {
	ty: Type,
}

impl TypedDesc {
	pub fn export(&self, out: &mut String) -> std::fmt::Result {
		let ty = self.ty.to_token_stream().to_string();
		write!(out, "<{ty} as rvm_core::Typed>::ty()")
	}
}
impl Parse for TypedDesc {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		Ok(TypedDesc { ty: input.parse()? })
	}
}

enum Desc {
	Typed(TypedDesc),
	Func(FuncDesc),
}

impl Parse for Desc {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		if input.peek(Token!(fn)) {
			Ok(Desc::Func(input.parse()?))
		} else {
			Ok(Desc::Typed(input.parse()?))
		}
	}
}

impl Desc {
	pub fn export(&self, out: &mut String) -> std::fmt::Result {
		match self {
			Desc::Typed(v) => v.export(out),
			Desc::Func(v) => v.export(out),
		}
	}
}
struct FuncDesc {
	pub _token: Token!(fn),
	pub _paren_token: token::Paren,
	pub parameters: Punctuated<Desc, Token![,]>,
	pub output: FuncReturn,
}

impl FuncDesc {
	pub fn export(&self, out: &mut String) -> std::fmt::Result {
		write!(out, "format!(\"")?;

		write!(out, "(")?;
		for _ in &self.parameters {
			write!(out, "{{}}")?;
		}
		write!(out, ")")?;

		if self.output.is_void() {
			write!(out, "V")?;
		} else {
			write!(out, "{{}}")?;
		}

		write!(out, "\",")?;
		// Resolve the types

		for desc in &self.parameters {
			desc.export(out)?;
			write!(out, ",")?;
		}

		if let FuncReturn::Type(_, value) = &self.output {
			value.export(out)?;
			write!(out, ",")?;
		}

		write!(out, ")")
	}
}

impl Parse for FuncDesc {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let content;
		Ok(FuncDesc {
			_token: input.parse()?,
			_paren_token: parenthesized!(content in input),
			parameters: content.parse_terminated(Desc::parse, Token![,])?,
			output: input.parse()?,
		})
	}
}

#[allow(dead_code)]
enum FuncReturn {
	Default,
	Type(Token![->], Box<Desc>),
}

impl FuncReturn {
	pub fn is_void(&self) -> bool {
		matches!(self, FuncReturn::Default)
	}
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
	let mut output = String::new();
	desc.export(&mut output).unwrap();
	output.parse().unwrap()
}
