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
use syn::TraitItemFn;
use syn::Type;
use syn::parse_macro_input;
use syn::punctuated::Punctuated;

pub fn boundary_gen(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemTrait);
    let args = parse_macro_input!(args with Punctuated::<Meta, syn::Token![,]>::parse_terminated);

    let bincode_enabled = args.iter().any(|arg| {
        match arg {
            Meta::Path(path) => path.is_ident("bincode"),
            _ => false,
        }
    });

    let in_process_enabled = args.iter().any(|arg| {
        match arg {
            Meta::Path(path) => path.is_ident("in_process"),
            _ => false,
        }
    });

    let grpc_enabled = args.iter().any(|arg| {
        match arg {
            Meta::Path(path) => path.is_ident("grpc"),
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

                    let inputs = get_inputs_as_ident_type_list(item);

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

                    let ty = get_output_as_type(&item);

                    Some(quote! {
                        #enum_item_name {
                            data: #ty
                        },
                    })
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

                    let field_names = get_inputs_as_idents_list(item);

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
                    let ident = item.sig.ident.clone();

                    let enum_item_name = syn::Ident::new(
                        &item.sig.ident.to_string().to_case(Case::Pascal),
                        proc_macro2::Span::call_site(),
                    );

                    let field_names = get_inputs_as_idents_list(item);

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

    let proxy = if in_process_enabled {
        Some(quote! {
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
        })
    } else {
        None
    };

    let grpc_wrap_functions: Vec<_> = items
        .into_iter()
        .filter_map(|item| {
            match item {
                TraitItem::Fn(item) => {
                    let fn_name = syn::Ident::new(
                        &ident.to_string().to_case(Case::Snake),
                        proc_macro2::Span::call_site(),
                    );

                    let ident = item.sig.ident.clone();
                    let enum_item_name = syn::Ident::new(
                        &item.sig.ident.to_string().to_case(Case::Pascal),
                        proc_macro2::Span::call_site(),
                    );


                    let field_names = get_inputs_as_idents_list(item);
                    let inputs = get_inputs_as_ident_type_list(item);
                    let ty = get_output_as_type(&item);

                    Some(quote! {
                        async fn #ident(
                            &self,
                            #(#inputs)*
                        ) -> RequestResult<#ty> {
                            let request = #request_enum_name::#enum_item_name {
                                #(#field_names)*
                            };

                            let encoded: Vec<u8> = bincode::encode_to_vec(&request, bincode::config::standard())
                                .map_err::<gauntlet_utils::channel::RequestError, _>(|err| anyhow::anyhow!("Unable to deserialize message with id: {:#}", err).into())?;

                            let response = self.client.#fn_name(encoded).await
                                .map_err::<gauntlet_utils::channel::RequestError, _>(|err| anyhow::anyhow!("{:#}", err).into())?;

                            let (decoded, _): (#response_enum_name, _) = bincode::decode_from_slice(&response[..], bincode::config::standard())
                                .map_err::<gauntlet_utils::channel::RequestError, _>(|err| anyhow::anyhow!("Unable to deserialize message with id: {:#}", err).into())?;

                            let #response_enum_name::#enum_item_name { data } = decoded else {
                                return Err(anyhow::anyhow!("Invalid return type from server: {:?}", decoded).into());
                            };

                            Ok(data)
                        }
                    })
                }
                _ => None,
            }
        })
        .collect();

    let handle_grpc_match_arms: Vec<_> = items
        .into_iter()
        .filter_map(|item| {
            match item {
                TraitItem::Fn(item) => {
                    let ident = item.sig.ident.clone();
                    let enum_item_name = syn::Ident::new(
                        &item.sig.ident.to_string().to_case(Case::Pascal),
                        proc_macro2::Span::call_site(),
                    );

                    let field_names = get_inputs_as_idents_list(item);

                    let result = quote! {
                        #request_enum_name::#enum_item_name {
                            #(#field_names)*
                        } => {
                            let result = handler.#ident(
                                #(#field_names)*
                            ).await;

                            match result {
                                Ok(result) => #response_enum_name::#enum_item_name { data: result },
                                Err(err) => {
                                    match err {
                                        gauntlet_utils::channel::RequestError::Timeout => panic!("unsupported return error: {}", err),
                                        gauntlet_utils::channel::RequestError::OtherSideWasDropped => panic!("unsupported return error: {}", err),
                                        gauntlet_utils::channel::RequestError::Other { display } => Err(tonic::Status::internal(display))?,
                                    }
                                }
                            }
                        }
                    };

                    Some(result)
                }
                _ => None,
            }
        })
        .collect();

    let grpc = if grpc_enabled {
        let fn_name = syn::Ident::new(
            &format!("handle_grpc_request_{}", ident.to_string().to_case(Case::Snake)),
            proc_macro2::Span::call_site(),
        );

        let result = quote! {
            #[derive(Debug, Clone)]
            pub struct #proxy_struct_name {
                client: GrpcBackendApi
            }

            impl #proxy_struct_name {
                pub fn new(
                    client: GrpcBackendApi
                ) -> Self {
                    Self {
                        client
                    }
                }
            }

            #[tonic::async_trait]
            impl #ident for #proxy_struct_name {
                #(#grpc_wrap_functions)*
            }

            pub async fn #fn_name(handler: &(dyn #ident + Send + Sync), data: Vec<u8>) -> Result<Vec<u8>, tonic::Status> {
                let (decoded, _): (#request_enum_name, _) = bincode::decode_from_slice(&data[..], bincode::config::standard())
                        .map_err(|err| tonic::Status::unknown(format!("Unable to deserialize message with id: {:#}", err)))?;

                let response = match decoded {
                    #(#handle_grpc_match_arms)*
                };

                let encoded: Vec<u8> = bincode::encode_to_vec(&response, bincode::config::standard())
                    .map_err(|err| tonic::Status::unknown(format!("Unable to deserialize message with id: {:#}", err)))?;

                Ok(encoded)
            }

        };

        Some(result)
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

        #proxy

        #grpc
    )
    .into()
}

fn get_inputs_as_idents_list(item: &TraitItemFn) -> Vec<proc_macro2::TokenStream> {
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

    field_names
}

fn get_inputs_as_ident_type_list(item: &TraitItemFn) -> Vec<proc_macro2::TokenStream> {
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

    inputs
}

fn get_output_as_type(item: &TraitItemFn) -> Option<proc_macro2::TokenStream> {
    let ty = match &item.sig.output {
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
                #ty
            )
        }
    };

    Some(ty)
}
