use proc_macro::TokenStream;
use quote::quote;
use syn::Attribute;
use syn::Data;
use syn::DeriveInput;
use syn::Expr;
use syn::Meta;
use syn::MetaNameValue;
use syn::Path;
use syn::parse_macro_input;

pub fn derive_rusqlite(item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as DeriveInput);

    let Data::Struct(data) = item.data else {
        panic!()
    };

    let struct_name = item.ident;

    let output: Vec<_> = data
        .fields
        .iter()
        .map(|field| {
            let field_name = field.ident.clone();

            let rename = field.attrs.iter().find_map(|attr| get_attr_rename(attr));
            let json = field.attrs.iter().any(|arg| is_attr_json(arg));

            let column_name = match rename {
                None => {
                    quote! {
                        stringify!(#field_name)
                    }
                }
                Some(rename) => {
                    quote! {
                        #rename
                    }
                }
            };

            let value = if json {
                quote! {
                    serde_json::from_value(row.get(#column_name)?).map_err(|err| rusqlite::Error::ToSqlConversionFailure(Box::new(err)))?
                }
            } else {
                quote! {
                    row.get(#column_name)?
                }
            };

            quote! {
                #field_name: #value,
            }
        })
        .collect();

    let result = quote! {
        impl crate::plugins::data_db_repository::RusqliteFromRow for #struct_name {
            fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
                Ok(Self {
                    #(#output)*
                })
            }
        }
    };

    result.into()
}

fn is_attr_json(attr: &Attribute) -> bool {
    match &attr.meta {
        Meta::List(list) => {
            let rusqlite = list.path.is_ident("rusqlite");
            if rusqlite {
                let Ok(path) = list.parse_args::<Path>() else {
                    return false;
                };

                path.is_ident("json")
            } else {
                false
            }
        }
        _ => false,
    }
}

fn get_attr_rename(attr: &Attribute) -> Option<Expr> {
    match &attr.meta {
        Meta::List(list) => {
            let rusqlite = list.path.is_ident("rusqlite");
            if rusqlite {
                let Ok(name_value) = list.parse_args::<MetaNameValue>() else {
                    return None;
                };

                if !name_value.path.is_ident("rename") {
                    return None;
                }

                Some(name_value.value)
            } else {
                None
            }
        }
        _ => None,
    }
}
