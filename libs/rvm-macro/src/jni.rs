use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use proc_macro_error::abort;
use std::fmt::Write;
use std::str::FromStr;
use syn::__private::ToTokens;
use syn::punctuated::Iter;
use syn::spanned::Spanned;
use syn::{parse, FnArg, ImplItem, ImplItemConst, ImplItemFn, ItemImpl, ReturnType};

pub fn jni_binding(attr: TokenStream, item: TokenStream) -> TokenStream {
	let package_name = attr.to_string();
	let mut item_impl: ItemImpl = parse(item).unwrap();

	let mut out = String::new();
	generate_link(&mut out, &package_name, &item_impl).unwrap();

	let token_stream = TokenStream::from_str(&out).expect(&out);
	let link_item_impl: ImplItemFn = parse(token_stream).unwrap();

	item_impl.items.push(ImplItem::Fn(link_item_impl));
	let stream = item_impl.to_token_stream();

	stream.into()
}
pub fn jni_method(attr: TokenStream, item: TokenStream) -> TokenStream {
	item
}

fn generate_link(out: &mut String, package_name: &str, item: &ItemImpl) -> std::fmt::Result {
	let item_name = item.self_ty.to_token_stream().to_string();
	writeln!(out, "fn link(runtime: &Runtime) {{ unsafe {{")?;
	for item in &item.items {
		let ImplItem::Fn(func) = item else {
			continue;
		};

		let Some(method_attr) = func
			.attrs
			.iter()
			.find(|attribute| attribute.meta.path().is_ident(&"jni_method"))
		else {
			continue;
		};

		let func_name = func.sig.ident.to_string();
		let jni_func_name = method_attr
			.meta
			.require_list()
			.ok()
			.map(|v| v.tokens.to_string())
			.unwrap_or(func_name.clone().to_case(Case::Camel));

		let mut inputs = func.sig.inputs.iter();

		ensure_first_input_runtime(func.sig.span(), &mut inputs);

		let mut output_type = None;
		if let ReturnType::Type(_, ty, ..) = &func.sig.output {
			output_type = Some(ty.to_token_stream().to_string());
		}

		let mut input_types = Vec::new();
		for arg in inputs {
			let FnArg::Typed(arg) = arg else { panic!() };

			input_types.push(arg.ty.to_token_stream().to_string());
		}

		{
			writeln!(
				out,
				"unsafe extern \"C\" fn {func_name}(runtime: *const Runtime,"
			)?;
			for (i, ty) in input_types.iter().enumerate() {
				writeln!(out, "v{i}: {ty},")?;
			}
			writeln!(out, ")")?;
			if let Some(ty) = output_type.as_ref() {
				writeln!(out, " -> {ty}")?;
			}
			writeln!(out, "{{")?;
			writeln!(out, "let runtime = Arc::from_raw(runtime);")?;
			write!(out, "let returns = {item_name}::{func_name}(&runtime, ")?;
			for (i, ty) in input_types.iter().enumerate() {
				write!(out, "v{i},")?;
			}
			writeln!(out, ");")?;
			writeln!(out, "let _ = Arc::into_raw(runtime);")?;
			writeln!(out, "returns")?;
			writeln!(out, "}}")?;
		}
		// 			runtime.linker.lock().link(
		// 				"Java_testing_rni_RniTests_testNative".to_string(),
		// 				MethodBinding::new(
		// 					transmute::<*const (), extern "C" fn()>(test as *const ()),
		// 					MethodDescriptor {
		// 						parameters: vec![i32::ty(), i64::ty(), i32::ty()],
		// 						returns: Some(i64::ty()),
		// 					},
		// 				),
		// 			);
		{
			writeln!(out, "let desc = ")?;
			writeln!(out, "rvm_core::MethodDescriptor {{ parameters: vec![")?;
			for ty in &input_types {
				write!(out, "<{ty} as rvm_core::Typed>::ty(),")?;
			}
			write!(out, "], returns: ")?;
			match output_type {
				Some(ty) => {
					writeln!(out, "Some(<{ty} as rvm_core::Typed>::ty())")?;
				}
				None => {
					writeln!(out, "None")?;
				}
			}
			writeln!(out, "}};")?;

			writeln!(out, "runtime.bindings.write().insert(")?;

			writeln!(out, "rvm_runtime::MethodIdentifier {{")?;
			//let string = package_name.replace("/", "_").replace(" ", "");
			//writeln!(
			//	out,
			//	"name: \"Java_{}_{jni_func_name}\".to_string(),",
			//	string
			//)?;
			writeln!(out, "name: \"{jni_func_name}\".to_string(),")?;
			writeln!(out, "descriptor: desc.to_string(),")?;
			writeln!(out, "}},")?;

			writeln!(out, "MethodBinding::new(std::mem::transmute::<*const (), extern \"C\" fn()>({func_name} as *const ()), desc)")?;
			writeln!(out, ");")?;
		}
	}

	writeln!(out, "}} }}")?;
	Ok(())
}

fn ensure_first_input_runtime(span: proc_macro2::Span, inputs: &mut Iter<FnArg>) {
	let first = inputs
		.next()
		.map(|v| v.to_token_stream().to_string())
		.unwrap_or_default()
		.replace(" ", "");
	if !first.contains("Arc<Runtime>") {
		if first.is_empty() {
			abort!(span, "Function needs to contain atleast a single argument.");
		} else {
			let message =
				format!("Function first argument needs to be Arc<Runtime> and not {first}");
			abort!(span, message);
		}
	}
}
