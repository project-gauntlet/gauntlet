use convert_case::Case;
use convert_case::Casing;
use proc_macro::TokenStream;
use quote::quote;
use syn::FnArg;
use syn::ItemTrait;
use syn::Meta;
use syn::Pat;
use syn::PathArguments;
use syn::ReturnType;
use syn::TraitItem;
use syn::Type;
use syn::parse_macro_input;
use syn::punctuated::Punctuated;

#[proc_macro_attribute]
pub fn boundary_gen(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemTrait);
    let args = parse_macro_input!(args with Punctuated::<Meta, syn::Token![,]>::parse_terminated);

    let bincode_enabled = args.iter().any(|arg| {
        match arg {
            Meta::Path(path) => path.is_ident("bincode"),
            _ => false,
        }
    });

    let ItemTrait { ident, items, .. } = &input;

    let request_enum_name = syn::Ident::new(
        &format!("{}RequestData", ident.to_string()),
        proc_macro2::Span::call_site(),
    );
    let response_enum_name = syn::Ident::new(
        &format!("{}ResponseData", ident.to_string()),
        proc_macro2::Span::call_site(),
    );

    let proxy_struct_name = syn::Ident::new(&format!("{}Proxy", ident.to_string()), proc_macro2::Span::call_site());

    let request_enum_items: Vec<_> = items
        .into_iter()
        .filter_map(|item| {
            match item {
                TraitItem::Fn(item) => {
                    let enum_item_name = syn::Ident::new(
                        &item.sig.ident.to_string().to_case(Case::Pascal),
                        proc_macro2::Span::call_site(),
                    );

                    let inputs: Vec<_> = item
                        .sig
                        .inputs
                        .iter()
                        .filter_map(|item| {
                            match item {
                                FnArg::Receiver(_) => None,
                                FnArg::Typed(item) => {
                                    match item.pat.as_ref() {
                                        Pat::Ident(pat) => {
                                            let ident = pat.ident.clone();
                                            let ty = item.ty.clone();
                                            let result = quote!(#ident: #ty,);

                                            Some(result)
                                        }
                                        _ => None,
                                    }
                                }
                            }
                        })
                        .collect();

                    let result = quote!(
                        #enum_item_name {
                            #(#inputs)*
                        },
                    );

                    Some(result)
                }
                _ => None,
            }
        })
        .collect();

    let response_enum_items: Vec<_> = items
        .into_iter()
        .filter_map(|item| {
            match item {
                TraitItem::Fn(item) => {
                    let enum_item_name = syn::Ident::new(
                        &item.sig.ident.to_string().to_case(Case::Pascal),
                        proc_macro2::Span::call_site(),
                    );

                    let result = match &item.sig.output {
                        ReturnType::Default => {
                            panic!("Plain () return type is not supported, return anyhow::Result<()>")
                        }
                        ReturnType::Type(_, ty) => {
                            let ty = match ty.as_ref() {
                                Type::Path(item) => {
                                    let item = item.path.segments.last().unwrap();
                                    if &item.ident == "RequestResult" {
                                        match &item.arguments {
                                            PathArguments::AngleBracketed(item) => item.args.first(),
                                            _ => return None,
                                        }
                                    } else {
                                        return None;
                                    }
                                }
                                _ => return None,
                            };

                            quote!(
                                #enum_item_name {
                                    data: #ty
                                },
                            )
                        }
                    };

                    Some(result)
                }
                _ => None,
            }
        })
        .collect();

    let impl_fns: Vec<_> = items
        .into_iter()
        .filter_map(|item| {
            match item {
                TraitItem::Fn(item) => {
                    let sig = item.sig.clone();
                    let ident = item.sig.ident.clone();

                    let enum_item_name = syn::Ident::new(
                        &item.sig.ident.to_string().to_case(Case::Pascal),
                        proc_macro2::Span::call_site(),
                    );

                    let field_names: Vec<_> = item
                        .sig
                        .inputs
                        .iter()
                        .filter_map(|item| {
                            match item {
                                FnArg::Receiver(_) => None,
                                FnArg::Typed(item) => {
                                    match item.pat.as_ref() {
                                        Pat::Ident(pat) => {
                                            let ident = pat.ident.clone();
                                            let result = quote!(#ident,);
                                            Some(result)
                                        }
                                        _ => None,
                                    }
                                }
                            }
                        })
                        .collect();

                    let result = quote!(
                        #sig {
                            let request = #request_enum_name::#enum_item_name {
                                #(#field_names)*
                            };

                            match self.request(request).await? {
                                #response_enum_name::#enum_item_name { data } => Ok(data),
                                value @ _ => panic!("Unexpected {} return type: {:?}", stringify!(#ident), value),
                            }
                        }
                    );

                    Some(result)
                }
                _ => None,
            }
        })
        .collect();

    let handle_impl_fns: Vec<_> = items
        .into_iter()
        .filter_map(|item| {
            match item {
                TraitItem::Fn(item) => {
                    let sig = item.sig.clone();
                    let ident = item.sig.ident.clone();

                    let enum_item_name = syn::Ident::new(
                        &item.sig.ident.to_string().to_case(Case::Pascal),
                        proc_macro2::Span::call_site(),
                    );

                    let field_names: Vec<_> = item
                        .sig
                        .inputs
                        .iter()
                        .filter_map(|item| {
                            match item {
                                FnArg::Receiver(_) => None,
                                FnArg::Typed(item) => {
                                    match item.pat.as_ref() {
                                        Pat::Ident(pat) => {
                                            let ident = pat.ident.clone();
                                            let result = quote!(#ident,);
                                            Some(result)
                                        }
                                        _ => None,
                                    }
                                }
                            }
                        })
                        .collect();

                    let result = quote!(
                        #request_enum_name::#enum_item_name {
                            #(#field_names)*
                        } => {
                            let data = api.#ident(
                                #(#field_names)*
                            ).await?;

                            Ok(#response_enum_name::#enum_item_name { data })
                        }
                    );

                    Some(result)
                }
                _ => None,
            }
        })
        .collect();

    let bincode_derive = if bincode_enabled {
        Some(quote!(
           #[derive(bincode::Encode, bincode::Decode)]
        ))
    } else {
        None
    };

    quote!(
        #input

        #[derive(Debug)]
        #bincode_derive
        pub enum #request_enum_name {
            #(#request_enum_items)*
        }

        #[derive(Debug)]
        #bincode_derive
        pub enum #response_enum_name {
            #(#response_enum_items)*
        }

        #[derive(Clone)]
        pub struct #proxy_struct_name {
            request_sender: gauntlet_utils::channel::RequestSender<#request_enum_name, #response_enum_name>,
        }

        impl #proxy_struct_name {
            pub fn new(request_sender: gauntlet_utils::channel::RequestSender<#request_enum_name, #response_enum_name>) -> Self {
                Self { request_sender }
            }

            async fn request(&self, request: #request_enum_name) -> gauntlet_utils::channel::RequestResult<#response_enum_name> {
                self.request_sender.send_receive(request).await
            }
        }

        impl #ident for #proxy_struct_name {
            #(#impl_fns)*
        }

        pub async fn handle_proxy_message(message: #request_enum_name, api: &impl #ident) -> anyhow::Result<#response_enum_name> {
            match message {
                #(#handle_impl_fns)*
            }
        }
    )
        .into()
}
