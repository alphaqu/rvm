use proc_macro::TokenStream;
use syn::{FnArg, parse_macro_input, parse_quote, PatType};

#[proc_macro_attribute]
pub fn method(attr: TokenStream, item: TokenStream) -> TokenStream {



    TokenStream::from(item)
}