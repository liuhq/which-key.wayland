use proc_macro::TokenStream;

#[proc_macro_derive(KdlParse, attributes(kdl))]
pub fn derive_kdl_parse(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}
