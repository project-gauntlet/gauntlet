use proc_macro::TokenStream;

mod boundary_gen;
mod rusqlite;
mod widget_deserialization_gen;
mod widget_model_gen;

#[proc_macro_attribute]
pub fn boundary_gen(args: TokenStream, input: TokenStream) -> TokenStream {
    boundary_gen::boundary_gen(args, input)
}

#[proc_macro_derive(RusqliteFromRow, attributes(rusqlite))]
pub fn derive_rusqlite(item: TokenStream) -> TokenStream {
    rusqlite::derive_rusqlite(item)
}

#[proc_macro]
pub fn widget_model_gen(_item: TokenStream) -> TokenStream {
    widget_model_gen::widget_model_gen()
}

#[proc_macro]
pub fn widget_deserialization_gen(_item: TokenStream) -> TokenStream {
    widget_deserialization_gen::widget_deserialization_gen()
}
