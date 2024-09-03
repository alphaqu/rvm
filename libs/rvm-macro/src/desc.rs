use proc_macro::TokenStream;
use std::fmt::Write;

use syn::__private::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parenthesized, parse, token, Token, Type};

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

pub fn java_desc(item: TokenStream) -> TokenStream {
	let desc: Desc = parse(item).unwrap();
	let mut output = String::new();
	desc.export(&mut output).unwrap();
	output.parse().unwrap()
}
