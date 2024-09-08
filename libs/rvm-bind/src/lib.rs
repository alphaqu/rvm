#![feature(iterator_try_collect)]
//! RVM bind creates rust bindings to the java classes

mod class;
mod package;

use crate::class::ClassFile;
use crate::package::Package;
use convert_case::{Case, Casing};
use eyre::Context;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, TokenStreamExt};
use rvm_class::{Class, ClassLoader, Field, LoadResult, Method, MethodIdentifier};
use rvm_core::{
	Id, Kind, MethodAccessFlags, MethodDescriptor, ObjectType, PrimitiveType, Type, VecExt,
};
use rvm_reader::{AttributeInfo, ConstantPool, Op};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Write;
use std::str::FromStr;
use syn::punctuated::Punctuated;
use syn::{parse, parse2, parse_file, parse_str, Path, PathSegment};

fn tokenize(str: &str) -> TokenStream {
	TokenStream::from_str(str).unwrap()
}

const BASE_FIELD_NAME: &str = "_base";
pub struct JavaBinder {
	pub loader: ClassLoader,
	pub to_bind: Vec<(Id<Class>, bool)>,
}

impl JavaBinder {
	pub fn bind(&mut self, ty: &ObjectType) -> eyre::Result<Id<Class>> {
		Self::bind_raw(&self.loader, ty, &mut self.to_bind)
	}

	fn bind_raw(
		loader: &ClassLoader,
		ty: &ObjectType,
		to_bind: &mut Vec<(Id<Class>, bool)>,
	) -> eyre::Result<Id<Class>> {
		Ok(match loader.load(ty) {
			LoadResult::New(value) => {
				let id = value?;
				let is_java = loader.get(id).ty.package().starts_with("java/lang");
				to_bind.push((id, !is_java));

				if !is_java {
					loader
						.resolve(id, &mut |ty| Self::bind_raw(loader, ty, to_bind))
						.wrap_err_with(|| format!("While resolving {ty}"))?;
				}

				id
			}
			LoadResult::Existing(value) => value,
		})
	}

	pub fn compile(&mut self) -> String {
		let mut root = Package::default();
		for (id, simple) in &self.to_bind {
			root.insert(ClassFile {
				data: self.loader.get(*id),
				full_binding: *simple,
			});
		}

		let stream = self.compile_module(&root);
		let mut text = stream.to_string();
		match parse_file(&text) {
			Ok(file) => prettyplease::unparse(&file),
			Err(err) => {
				text.push_str("//");
				text.push_str(&err.to_string());
				text.push('\n');
				text
			}
		}
	}

	fn compile_module(&mut self, package: &Package) -> TokenStream {
		let mut output = TokenStream::new();
		for file in &package.files {
			output.append_all(self.compile_class(file));
		}

		for (inner, package) in &package.packages {
			let module_name = Ident::new(inner, Span::call_site());
			let module_content = self.compile_module(package);
			output.append_all(quote! {
				pub mod #module_name {
					#module_content
				}
			});
		}

		output
	}

	fn compile_class(&mut self, class: &ClassFile) -> TokenStream {
		let ctx = Ctx {
			class,
			classes: &self.loader,
		};
		let tokenized_class = TokenizedClass::new(&ctx);

		// Struct
		let struct_ts = Self::def_struct(&tokenized_class);
		let instance_binding_ts = Self::def_instance_binding(&tokenized_class);
		let java_typed_ts = Self::def_java_typed(&tokenized_class);
		let constants_ts = Self::def_constants(&tokenized_class);
		let methods_ts = Self::def_methods(&tokenized_class);
		let deref_ts = Self::def_deref(&tokenized_class);

		let ident = tokenized_class.ident.clone();
		quote! {
			#struct_ts

			impl #ident {
				#constants_ts
				#methods_ts
			}

			#instance_binding_ts
			#java_typed_ts
			#deref_ts
		}
	}

	fn def_struct(class: &TokenizedClass) -> TokenStream {
		let ident = &class.ident;
		let mut fields: Vec<TokenStream> = class
			.fields
			.iter()
			.map(|TokenizedField { ident, ty }| {
				quote! {
					pub #ident: rvm_runtime::TypedField<#ty>
				}
			})
			.collect();

		if let Some(BaseClass {
			field: TokenizedField { ident, .. },
			ident: binding,
		}) = &class.base_field
		{
			fields.insert(
				0,
				quote! {
					#ident: #binding
				},
			);
		}

		quote! {
			#[derive(Copy, Clone)]
			pub struct #ident {
				#(#fields),*
			}
		}
	}
	fn def_instance_binding(class: &TokenizedClass) -> TokenStream {
		let ident = &class.ident;

		let mut field_bindings: Vec<TokenStream> = class
			.fields
			.iter()
			.map(|TokenizedField { ident, .. }| {
				let field_lit = ident.to_string();
				quote! {
					#ident: fields.by_name_typed(#field_lit).unwrap()
				}
			})
			.collect();

		if let Some(BaseClass {
			field: TokenizedField { ident, .. },
			ident: binding,
		}) = &class.base_field
		{
			field_bindings.insert(
				0,
				quote! {
					#ident: #binding::bind(instance)
				},
			);
		}

		quote! {
			impl rvm_runtime::InstanceBinding for #ident {
				fn ty() -> rvm_core::ObjectType {
					rvm_core::ObjectType::new(Self::TY)
				}

				fn bind(instance: &rvm_runtime::AnyInstance) -> Self {
					let fields = instance.fields();
					Self {
						#(#field_bindings),*
					}
				}
			}
		}
	}
	fn def_constants(class: &TokenizedClass) -> TokenStream {
		let name = class.full_name.to_string();
		quote! {
			pub const TY: &'static str = #name;
		}
	}

	fn def_methods(class: &TokenizedClass) -> TokenStream {
		let mut methods = TokenStream::new();
		for method in &class.methods {
			let name = method.ident.clone();
			let name_descriptor = format_ident!("{}_descriptor", method.ident);

			if !method.flags.contains(MethodAccessFlags::STATIC) {
				continue;
			}

			let returns = match method.returns.as_ref() {
				Some(value) => value.clone(),
				None => tokenize("()"),
			};

			let arguments: Vec<TokenStream> = method
				.arguments
				.iter()
				.map(|MethodArgument { name, ty, .. }| {
					quote! {
						#name: #ty
					}
				})
				.collect();

			let argument_call: Vec<TokenStream> = method
				.arguments
				.iter()
				.map(|MethodArgument { name, .. }| {
					quote! {
						rvm_runtime::ToJava::to_java(#name, runtime)?
					}
				})
				.collect();

			let return_expr = match method.returns {
				Some(_) => quote! {
					let output = output.expect("expected return");
					rvm_runtime::FromJava::from_java(output, runtime)
				},
				None => quote! {
					if !output.is_none() {
						panic!("Returned on void");
					}
					Ok(())
				},
			};

			let i_name = method.method_ident.name.to_string();
			let i_descriptor = method.method_ident.descriptor.to_string();

			methods.append_all(quote! {
				pub fn #name_descriptor() -> rvm_runtime::MethodIdentifier {
					rvm_runtime::MethodIdentifier {
						name: std::sync::Arc::from(#i_name),
						descriptor: std::sync::Arc::from(#i_descriptor),
					}
				}

				pub fn #name(runtime: &rvm_runtime::Runtime, #(#arguments),*) -> eyre::Result<#returns> {
					let output = runtime.simple_run(
						<Self as rvm_runtime::InstanceBinding>::ty(),
						Self::#name_descriptor(),
						vec![
							#(#argument_call),*
						],
					)?;
					#return_expr
				}
			});
		}

		methods
	}

	fn def_deref(class: &TokenizedClass) -> TokenStream {
		let ident = &class.ident;
		let Some(BaseClass {
			ident: base_ident,
			field: TokenizedField {
				ident: base_field, ..
			},
		}) = &class.base_field
		else {
			return TokenStream::new();
		};

		quote! {
			impl std::ops::Deref for #ident {
				type Target = #base_ident;

				fn deref(&self) -> &Self::Target {
					&self.#base_field
				}
			}

			impl std::ops::DerefMut for #ident {
				fn deref_mut(&mut self) -> &mut Self::Target {
					&mut self.#base_field
				}
			}
		}
	}

	fn def_java_typed(class: &TokenizedClass) -> TokenStream {
		let ident = &class.ident;
		quote! {
			impl rvm_runtime::JavaTyped for #ident {
				fn java_type() -> rvm_core::Type {
					<Self as rvm_runtime::InstanceBinding>::ty().into()
				}
			}
		}
	}
}

pub struct TokenizedClass {
	full_name: String,
	ident: Ident,
	// Fields
	base_field: Option<BaseClass>,
	fields: Vec<TokenizedField>,
	static_fields: Vec<TokenizedField>,
	//
	methods: Vec<TokenizedMethod>,
}

impl TokenizedClass {
	pub fn new(ctx: &Ctx) -> TokenizedClass {
		let class = ctx.class;
		let ident = rust_ident(&class.ty.name(), Span::call_site());

		let mut base_field = None;
		let mut fields = Vec::new();
		let mut static_fields = Vec::new();
		for field in class.fields.iter() {
			let Some(tokenized) = TokenizedField::from_class(ctx, field) else {
				continue;
			};
			if !field.is_static() {
				fields.push(tokenized);
			} else {
				static_fields.push(tokenized);
			}
		}

		base_field = class.superface.superclass.as_ref().and_then(|superclass| {
			let field =
				TokenizedField::new(ctx, BASE_FIELD_NAME, &Type::Object(superclass.ty.clone()))?;
			let string = ctx.class_ident_to_rust(&superclass.ty)?;
			Some(BaseClass {
				ident: string,
				field,
			})
		});

		let mut methods = vec![];

		let mut method_namer = MethodNamer {
			names: Default::default(),
		};
		for method in class.methods.iter() {
			//if !class.full_binding && !method.flags.contains(MethodAccessFlags::PUBLIC) {
			//	continue;
			//}
			method_namer.add_method(method.to_identifier());
		}
		let method_names = method_namer.compile();
		for method in class.methods.iter() {
			let Some(ident) = method_names.get(&method.to_identifier()) else {
				continue;
			};

			let Some(method) =
				TokenizedMethod::from_class(ctx, ident.clone(), &ctx.class.cp, method)
			else {
				continue;
			};

			methods.push(method);
		}

		TokenizedClass {
			full_name: (*class.ty).to_string(),
			ident,
			base_field,
			fields,
			static_fields,
			methods,
		}
	}
	//pub fn all_fields(&self) -> impl Iterator<Item = &TokenizedField> {
	//	self.fields
	//		.iter()
	//		.chain(self.base_field.as_ref().map(|v| &v.field))
	//}
}

pub struct BaseClass {
	ident: syn::Path,
	field: TokenizedField,
}

pub struct TokenizedField {
	ident: Ident,
	ty: TokenStream,
}

impl TokenizedField {
	pub fn new(ctx: &Ctx, name: &str, ty: &Type) -> Option<Self> {
		let ident = rust_ident(name, Span::call_site());
		let ty = ctx.ty_to_rust(ty)?;
		Some(Self { ident, ty })
	}
	pub fn from_class(ctx: &Ctx, field: &Field) -> Option<Self> {
		Self::new(ctx, &field.name, &field.ty)
	}
}

pub struct TokenizedMethod {
	ident: Ident,
	flags: MethodAccessFlags,
	method_ident: MethodIdentifier,
	arguments: Vec<MethodArgument>,
	returns: Option<TokenStream>,
}

pub struct MethodNamer {
	names: BTreeMap<String, (MethodNamingStrategy, Vec<MethodIdentifier>)>,
}

impl MethodNamer {
	pub fn add_method(&mut self, identifier: MethodIdentifier) {
		self.names
			.entry(identifier.name.to_string())
			.or_insert_with(|| (MethodNamingStrategy::Basic, vec![]))
			.1
			.push(identifier);
	}

	pub fn compile(self) -> HashMap<MethodIdentifier, Ident> {
		let mut values: Vec<(MethodNamingStrategy, Vec<MethodIdentifier>)> =
			self.names.into_values().collect();

		// Biggest to smallest, so that larger overloads don't make smaller methods use a longer name
		// (when stage 2 of the big one collides with a smaller method).
		values.sort_by_key(|(_, v)| v.len());
		values.reverse();

		'retry: loop {
			let mut output = HashMap::new();
			let mut conflicting_names_checker = HashSet::new();

			for (naming_strategy, signatures) in &mut values {
				for identifier in signatures.as_mut_slice() {
					let rust_name =
						rust_ident(&naming_strategy.to_rust(identifier), Span::call_site());
					if !conflicting_names_checker.insert(rust_name.clone()) {
						if let Some(next_strategy) = naming_strategy.get_next() {
							*naming_strategy = next_strategy;
						} else {
							println!(
								"Could not find a way to not have a conflict in {identifier:?}"
							);
							signatures.clear();
						}

						continue 'retry;
					}

					if output.insert(identifier.clone(), rust_name).is_some() {
						panic!("Conflicting method identifier {identifier:?}");
					}
				}
			}

			return output;
		}
	}
}

#[derive(Ord, PartialOrd, Eq, PartialEq)]
pub enum MethodNamingStrategy {
	Basic,
	Extended,
}

impl MethodNamingStrategy {
	pub fn get_next(&self) -> Option<MethodNamingStrategy> {
		match self {
			MethodNamingStrategy::Basic => Some(MethodNamingStrategy::Extended),
			MethodNamingStrategy::Extended => None,
		}
	}
	pub fn to_rust(&self, identifier: &MethodIdentifier) -> String {
		let name = identifier.name.to_string();
		let mut name = if name == "<init>" {
			"new".to_string()
		} else if name == "<clinit>" {
			"class_init".to_string()
		} else {
			name
		};

		match self {
			MethodNamingStrategy::Basic => name,
			MethodNamingStrategy::Extended => {
				let desc = MethodDescriptor::parse(&identifier.descriptor).unwrap();

				for parameter in &desc.parameters {
					write!(
						&mut name,
						"_{}",
						match parameter.kind() {
							Kind::Reference => "ref",
							Kind::Boolean => "bool",
							Kind::Char => "char",
							Kind::Float => "f32",
							Kind::Double => "f64",
							Kind::Byte => "i8",
							Kind::Short => "i16",
							Kind::Int => "i32",
							Kind::Long => "i64",
						}
					)
					.unwrap();
				}

				name
			}
		}
	}
}

pub struct MethodArgument {
	name: Ident,
	ty: TokenStream,
}

impl TokenizedMethod {
	pub fn from_class(ctx: &Ctx, ident: Ident, cp: &ConstantPool, method: &Method) -> Option<Self> {
		let range_start = if method.is_static() { 0 } else { 1 };
		let range_end = range_start + method.desc.parameters.len();

		let argument_range = range_start..range_end;
		let argument_names = method
			.attributes
			.first_where(|v| {
				if let AttributeInfo::LocalVariableTable { variables } = v {
					let mut names = Vec::new();
					for index in argument_range.clone() {
						names.push(variables.first_where(|v| {
							(v.index == index as u16).then(|| cp[v.name_index].to_string())
						}));
					}
					Some(names)
				} else {
					None
				}
			})
			.unwrap_or_else(|| vec![None; argument_range.len()]);

		// This ensures that all the argument names are unique, and filled.
		let mut i = 0;
		let argument_names: Vec<String> = argument_names
			.iter()
			.map(|v| {
				v.clone().unwrap_or_else(|| loop {
					let possible_name = format!("var{i}");
					i += 1;
					if !argument_names.contains(&Some(possible_name.clone())) {
						return possible_name;
					}
				})
			})
			.collect();

		let arguments: Vec<MethodArgument> = method
			.desc
			.parameters
			.iter()
			.zip(argument_names)
			.map(|(ty, name)| {
				Some(MethodArgument {
					name: Ident::new(&name, Span::call_site()),
					ty: ctx.ty_to_rust(ty)?,
				})
			})
			.try_collect()?;

		let returns = match method.desc.returns.as_ref() {
			Some(ty) => Some(ctx.ty_to_rust(ty)?),
			None => None,
		};
		Some(Self {
			ident,
			method_ident: method.to_identifier(),
			arguments,
			returns,
			flags: method.flags,
		})
	}
}

pub struct Ctx<'a> {
	class: &'a ClassFile,
	classes: &'a ClassLoader,
}

impl<'a> Ctx<'a> {
	pub fn ty_to_rust(&self, ty: &Type) -> Option<TokenStream> {
		Some(match ty {
			Type::Primitive(primitive) => match primitive {
				PrimitiveType::Boolean => tokenize("bool"),
				PrimitiveType::Byte => tokenize("i8"),
				PrimitiveType::Short => tokenize("i16"),
				PrimitiveType::Int => tokenize("i32"),
				PrimitiveType::Long => tokenize("i64"),
				PrimitiveType::Char =>
				/*tokenize("char")*/
				{
					return None;
				}
				PrimitiveType::Float => tokenize("f32"),
				PrimitiveType::Double => tokenize("f64"),
			},
			Type::Object(object) => {
				if object == &ObjectType::Object() {
					tokenize("rvm_runtime::Reference")
				} else {
					let binding = self.class_ident_to_rust(object)?;
					quote! {
						rvm_runtime::Instance<#binding>
					}
				}
			}
			Type::Array(array) => {
				let component = self.ty_to_rust(array.component())?;
				quote! {
					rvm_runtime::Array<#component>
				}
			}
		})
	}

	/// Converts a fully qualified java (java/lang/Object) name to rust (java::lang::Object)
	pub fn class_ident_to_rust(&self, ident: &ObjectType) -> Option<syn::Path> {
		let mut navigations = Vec::new();

		// Ensure the class exists
		_ = self.classes.get_named(ident)?;

		let target_package = ident.package_path();
		let current_package = self.class.ty.package_path();

		// TARGET: java/lang
		// CURRENT: java/testing/thing/another
		let leading_count = count_leading_matches(&target_package, &current_package);

		// navigate back until we reach common ground
		for _ in 0..(current_package.len() - leading_count) {
			navigations.push("super".to_string());
		}

		// Navigate forward to our destination
		for part in &target_package[leading_count..] {
			navigations.push(part.to_string());
		}

		navigations.push(ident.name());
		Some(rust_path(&navigations.join("::"), Span::call_site()))
	}

	pub fn qualified_name(&self) -> String {
		(*self.class.ty).to_string()
	}

	/// Returns the current java package we are in (separated by slashes)
	pub fn package(&self) -> String {
		let full_name = self.qualified_name();
		full_name
			.rsplit_once("/")
			.map(|(package, _)| package.to_string())
			// This is for unpackaged classes
			.unwrap_or(full_name)
	}
}

fn rust_ident(str: &str, span: Span) -> Ident {
	let mut string = str.replace("$", "_");

	let reserved = [
		"as", "break", "const", "continue", "else", "enum", "extern", "false", "fn", "for", "if",
		"impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return",
		"static", "struct", "trait", "true", "type", "unsafe", "use", "where", "while", "async",
		"await", "dyn", "abstract", "become", "box", "do", "final", "macro", "override", "priv",
		"typeof", "unsized", "virtual", "yield", "try",
	];

	for reserved_ident in reserved {
		if reserved_ident == string {
			return Ident::new_raw(&string, span);
		}
	}

	if string == "crate" || string == "self" || string == "super" || string == "Self" {
		string = format!("{string}_");
	}

	Ident::new(&string, span)
}
fn rust_path(str: &str, span: Span) -> syn::Path {
	let mut punctuated = Punctuated::new();
	let mut super_allowed = true;
	for value in str.split("::") {
		if value == "super" && super_allowed {
			punctuated.push(Ident::new("super", span).into());
		} else {
			super_allowed = false;
			punctuated.push(rust_ident(value, span).into());
		}
	}

	syn::Path {
		leading_colon: None,
		segments: punctuated,
	}
}
fn count_leading_matches(vec1: &[String], vec2: &[String]) -> usize {
	let mut count = 0;
	let min_len = vec1.len().min(vec2.len());

	for i in 0..min_len {
		if vec1[i] == vec2[i] {
			count += 1;
		} else {
			break;
		}
	}

	count
}
#[cfg(test)]
mod tests {
	#[test]
	fn compile() {}
}
