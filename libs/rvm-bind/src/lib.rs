use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::__private::TokenStreamExt;
use syn::spanned::Spanned;
use syn::{parse_macro_input, FnArg, ItemFn, LitStr};

#[proc_macro_error]
#[proc_macro_attribute]
pub fn method(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let name = attr.to_string();
    let (class_name, descriptor) = name.split_once(',').unwrap();

    let func = parse_macro_input!(item as ItemFn);

    let mut args = TokenStream::new();
    let args_count = func.sig.inputs.len() as u16;
    for (i, arg) in func.sig.inputs.iter().enumerate() {
        let i = i as u16;
        match arg {
            FnArg::Receiver(receiver) => {
                abort!(receiver.span(), "Cannot have receiver parameter. yet")
            }
            FnArg::Typed(ty) => {
                let ty = &ty.ty;

                args.append_all(quote!(
               #ty::from_java(local_table.get(#i), runtime)?
            ))
            },
        }
    }

    let func_name = &func.sig.ident;
    let str = LitStr::new(&func.sig.ident.to_string(), Span::call_site());
    let call_method = Ident::new(&("bind".to_string() + &func_name.to_string()), Span::call_site());

    quote!(
        fn #call_method(runtime: &mut Runtime) {
            runtime.load_native(
                #class_name.to_string(),
                #str.to_string(),
                #descriptor.to_string(),
                NativeCode {
                    func: |local_table, runtime| {
                        Ok(Some(StackValue::from(
                            #func_name(#args)?.to_java(runtime)?,
                        )))
                    },
                    max_locals: #args_count,
                },
            );
        }

        #func
    ).into()
}
