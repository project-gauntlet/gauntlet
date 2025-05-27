use proc_macro::TokenStream;

mod boundary_gen;
mod rusqlite;

#[proc_macro_attribute]
pub fn boundary_gen(args: TokenStream, input: TokenStream) -> TokenStream {
    boundary_gen::boundary_gen(args, input)
}

#[proc_macro_derive(RusqliteFromRow, attributes(rusqlite))]
pub fn derive_rusqlite(item: TokenStream) -> TokenStream {
    rusqlite::derive_rusqlite(item)
}
