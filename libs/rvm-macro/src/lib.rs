mod desc;
mod jni;

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use rvm_reader::ClassInfo;
use std::env::vars;
use std::fs::read;
use std::path::PathBuf;
use syn::parse::Parse;
use syn::{parse, ItemStruct};
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

#[proc_macro]
pub fn java_desc(item: TokenStream) -> TokenStream {
	desc::java_desc(item)
}

#[proc_macro]
pub fn jni_method_name(item: TokenStream) -> TokenStream {
	let mut out = String::new();
	out.push_str("Java_");
	desc::java_desc(item)
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn jni_binding(attr: TokenStream, item: TokenStream) -> TokenStream {
	item
	//jni::jni_binding(attr, item)
}
#[proc_macro_attribute]
pub fn jni_method(attr: TokenStream, item: TokenStream) -> TokenStream {
	item
	//jni::jni_method(attr, item)
}
