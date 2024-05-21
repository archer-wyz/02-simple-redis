mod auto_deref;
use auto_deref::process_auto_deref;
use proc_macro::TokenStream;
use syn::DeriveInput;

#[proc_macro_derive(AutoDeref, attributes(deref))]
pub fn derive_auto_deref(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);

    process_auto_deref(input).into()
}
