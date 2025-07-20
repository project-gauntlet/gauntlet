use std::ops::Deref;

use convert_case::Case;
use convert_case::Casing;
use gauntlet_component_model::Arity;
use gauntlet_component_model::Children;
use gauntlet_component_model::Component;
use gauntlet_component_model::ComponentName;
use gauntlet_component_model::Property;
use gauntlet_component_model::PropertyKind;
use gauntlet_component_model::PropertyType;
use gauntlet_component_model::SharedType;
use gauntlet_component_model::create_component_model;
use indexmap::IndexMap;
use itertools::Itertools;
use proc_macro::TokenStream;
use quote::ToTokens;
use quote::quote;

pub fn widget_model_gen() -> TokenStream {
    let components = create_component_model();

    let mut output_components = vec![];

    for component in &components {
        match component {
            Component::Standard {
                name, props, children, ..
            } => {
                let props_has_content = props
                    .iter()
                    .any(|prop| matches!(prop.property_type.kind(), PropertyKind::Component));

                let children_has_content = match children {
                    Children::Members {
                        ordered_members,
                        per_type_members,
                        ..
                    }
                    | Children::StringOrMembers {
                        ordered_members,
                        per_type_members,
                        ..
                    } => !ordered_members.is_empty() || !per_type_members.is_empty(),
                    _ => false,
                };

                let has_text = matches!(children, Children::StringOrMembers { .. } | Children::String { .. });

                let has_content = children_has_content || props_has_content || has_text;

                let default = IndexMap::new();

                let (ordered_members, per_type_members) = match children {
                    Children::Members {
                        ordered_members,
                        per_type_members,
                        ..
                    }
                    | Children::StringOrMembers {
                        ordered_members,
                        per_type_members,
                        ..
                    } => (ordered_members, per_type_members),
                    _ => (&default, &default),
                };

                if !ordered_members.is_empty() {
                    let component_refs = ordered_members
                        .iter()
                        .map(|(_member_name, component_ref)| component_ref)
                        .unique_by(|component_ref| component_ref.component_name.clone())
                        .map(|component_ref| {
                            let enum_item_name = component_ref.component_name.to_string();
                            let enum_item_name = syn::Ident::new(&enum_item_name, proc_macro2::Span::call_site());

                            let widget_name = format!("{}Widget", component_ref.component_name);
                            let widget_name = syn::Ident::new(&widget_name, proc_macro2::Span::call_site());

                            quote! {
                                #enum_item_name(#widget_name)
                            }
                        })
                        .collect::<Vec<_>>();

                    let enum_name = &format!("{}WidgetOrderedMembers", name);
                    let enum_name = syn::Ident::new(enum_name, proc_macro2::Span::call_site());

                    output_components.push(quote! {
                        #[derive(Debug, Encode, Decode)]
                        pub enum #enum_name {
                            #(#component_refs),*
                        }
                    })
                }

                if has_content {
                    {
                        let props: Vec<_> = props
                            .iter()
                            .map(|prop| {
                                if matches!(prop.property_type.kind(), PropertyKind::Component) {
                                    let is_union = match &prop.property_type {
                                        PropertyType::Union { .. } => true,
                                        PropertyType::Array { item } => {
                                            matches!(item.as_ref(), PropertyType::Union { .. })
                                        }
                                        _ => false,
                                    };

                                    if is_union {
                                        let field_name = prop.name.to_case(Case::Snake);
                                        let field_name = syn::Ident::new(&field_name, proc_macro2::Span::call_site());

                                        let type_name = generate_required_type(
                                            &prop.property_type,
                                            Some(format!("{}{}", name, &prop.name.to_case(Case::Pascal))),
                                        );

                                        Some(quote! {
                                            pub #field_name: #type_name,
                                        })
                                    } else {
                                        let field_name = prop.name.to_case(Case::Snake);
                                        let field_name = syn::Ident::new(&field_name, proc_macro2::Span::call_site());

                                        let type_name = generate_type(&prop, name);

                                        Some(quote! {
                                            pub #field_name: #type_name,
                                        })
                                    }
                                } else {
                                    None
                                }
                            })
                            .collect();

                        let per_type_members: Vec<_> = per_type_members
                            .iter()
                            .map(|(member_name, component_ref)| {
                                match component_ref.arity {
                                    Arity::ZeroOrOne => {
                                        let field_name = member_name.to_case(Case::Snake);
                                        let field_name = syn::Ident::new(&field_name, proc_macro2::Span::call_site());

                                        let widget_name = &format!("{}Widget", component_ref.component_name);
                                        let widget_name = syn::Ident::new(&widget_name, proc_macro2::Span::call_site());

                                        quote! {
                                            pub #field_name: Option<#widget_name>,
                                        }
                                    }
                                    Arity::One => {
                                        let field_name = member_name.to_case(Case::Snake);
                                        let field_name = syn::Ident::new(&field_name, proc_macro2::Span::call_site());

                                        let widget_name = &format!("{}Widget", component_ref.component_name);
                                        let widget_name = syn::Ident::new(&widget_name, proc_macro2::Span::call_site());

                                        quote! {
                                            pub #field_name: #widget_name,
                                        }
                                    }
                                    Arity::ZeroOrMore => {
                                        todo!()
                                    }
                                }
                            })
                            .collect();

                        let ordered_members = if !ordered_members.is_empty() {
                            let ordered_members_name = &format!("{}WidgetOrderedMembers", name);
                            let ordered_members_name =
                                syn::Ident::new(&ordered_members_name, proc_macro2::Span::call_site());

                            Some(quote! {
                                pub ordered_members: Vec<#ordered_members_name>,
                            })
                        } else {
                            None
                        };

                        let text = if has_text {
                            Some(quote! {
                                pub text: Vec<String>,
                            })
                        } else {
                            None
                        };

                        let widget_content_name = &format!("{}WidgetContent", name);
                        let widget_content_name = syn::Ident::new(&widget_content_name, proc_macro2::Span::call_site());

                        output_components.push(quote! {
                            #[derive(Debug, Encode, Decode)]
                            pub struct #widget_content_name {
                                #(#props)*
                                #(#per_type_members)*
                                #ordered_members
                                #text
                            }
                        })
                    }
                }

                let prop_fields: Vec<_> = props
                    .iter()
                    .filter(|prop| matches!(prop.property_type.kind(), PropertyKind::Property))
                    .map(|prop| {
                        let field_name = prop.name.to_case(Case::Snake);
                        let field_name = syn::Ident::new(&field_name, proc_macro2::Span::call_site());

                        let type_name = generate_type(&prop, name);

                        quote! {
                            pub #field_name: #type_name,
                        }
                    })
                    .collect();

                let content = if has_content {
                    let widget_content_name = &format!("{}WidgetContent", name);
                    let widget_content_name = syn::Ident::new(&widget_content_name, proc_macro2::Span::call_site());

                    Some(quote! {
                        pub content: #widget_content_name,
                    })
                } else {
                    None
                };

                let widget_name = &format!("{}Widget", name);
                let widget_name = syn::Ident::new(&widget_name, proc_macro2::Span::call_site());

                output_components.push(quote! {
                    #[derive(Debug, Encode, Decode)]
                    pub struct #widget_name {
                        pub __id__: UiWidgetId,
                        #(#prop_fields)*
                        #content
                    }
                });

                fn generate_union(
                    name: &ComponentName,
                    items: &Vec<PropertyType>,
                    prop_name: &String,
                ) -> proc_macro2::TokenStream {
                    let enum_items: Vec<_> = items
                        .iter()
                        .enumerate()
                        .map(|(index, property_type)| {
                            let item_name = format!("_{}", index);
                            let item_name = syn::Ident::new(&item_name, proc_macro2::Span::call_site());
                            let item_type = generate_required_type(&property_type, None);
                            quote! {
                                #item_name(#item_type)
                            }
                        })
                        .collect();

                    let enum_name = format!("{}{}", name, prop_name.to_case(Case::Pascal));
                    let enum_name = syn::Ident::new(&enum_name, proc_macro2::Span::call_site());

                    quote! {
                        #[derive(Debug, Encode, Decode)]
                        pub enum #enum_name {
                            #(#enum_items),*
                        }
                    }
                }

                let unions: Vec<_> = props
                    .iter()
                    .map(|prop| {
                        match &prop.property_type {
                            PropertyType::Union { items } => Some(generate_union(&name, &items, &prop.name)),
                            PropertyType::Array { item } => {
                                match item.deref() {
                                    PropertyType::Union { items } => Some(generate_union(&name, &items, &prop.name)),
                                    _ => None,
                                }
                            }
                            _ => None,
                        }
                    })
                    .collect();

                output_components.push(quote! {
                    #(#unions)*
                })
            }
            Component::Root {
                children, shared_types, ..
            } => {
                let shared_types = shared_types
                    .iter()
                    .map(|(type_name, shared_type)| {
                        match shared_type {
                            SharedType::Enum { items } => {
                                let items: Vec<_> = items
                                    .iter()
                                    .map(|item| {
                                        let item_ident = syn::Ident::new(&item, proc_macro2::Span::call_site());
                                        quote! {
                                            #item_ident
                                        }
                                    })
                                    .collect();

                                let type_name = syn::Ident::new(&type_name, proc_macro2::Span::call_site());

                                quote! {
                                    #[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode, strum::EnumString)]
                                    pub enum #type_name {
                                        #(#items),*
                                    }
                                }
                            }
                            SharedType::Object { items } => {
                                let items: Vec<_> = items
                                    .iter()
                                    .map(|(property_name, property_type)| {
                                        let field_name = property_name;
                                        let field_name = syn::Ident::new(&field_name, proc_macro2::Span::call_site());

                                        let field_type = generate_required_type(
                                            &property_type,
                                            Some(format!("{}{}", type_name, property_name)),
                                        );
                                        quote! {
                                            pub #field_name: #field_type
                                        }
                                    })
                                    .collect();

                                let type_name = syn::Ident::new(&type_name, proc_macro2::Span::call_site());

                                quote! {
                                    #[derive(Debug, Encode, Decode)]
                                    pub struct #type_name {
                                        #(#items),*
                                    }
                                }
                            }
                            SharedType::Union { items } => {
                                let type_name = syn::Ident::new(&type_name, proc_macro2::Span::call_site());

                                let items: Vec<_> = items
                                    .iter()
                                    .map(|property_type| {
                                        match property_type {
                                            PropertyType::SharedTypeRef { name } => {
                                                let enum_item = name;
                                                let enum_item =
                                                    syn::Ident::new(&enum_item, proc_macro2::Span::call_site());

                                                quote! {
                                                    #enum_item(#enum_item)
                                                }
                                            }
                                            _ => {
                                                todo!()
                                            }
                                        }
                                    })
                                    .collect();

                                quote! {
                                    #[derive(Debug, Encode, Decode)]
                                    pub enum #type_name {
                                        #(#items),*
                                    }
                                }
                            }
                        }
                    })
                    .collect();

                output_components.push(shared_types);

                let children: Vec<_> = children
                    .iter()
                    .map(|component_ref| {
                        let enum_item = &component_ref.component_name;
                        let enum_item = syn::Ident::new(&enum_item.to_string(), proc_macro2::Span::call_site());

                        let enum_item_type = format!("{}Widget", component_ref.component_name);
                        let enum_item_type = syn::Ident::new(&enum_item_type, proc_macro2::Span::call_site());

                        quote! {
                            #enum_item(#enum_item_type)
                        }
                    })
                    .collect();

                output_components.push(quote! {
                    #[derive(Debug, Encode, Decode)]
                    pub enum RootWidgetMembers {
                        #(#children),*
                    }
                });

                output_components.push(quote! {
                    #[derive(Debug, Encode, Decode)]
                    pub struct RootWidget {
                        pub content: Option<RootWidgetMembers>
                    }
                })
            }
            Component::TextPart { .. } => {}
        }
    }

    let result = quote! {
        #(#output_components)*
    };

    result.into()
}

fn generate_type(property: &Property, name: &ComponentName) -> proc_macro2::TokenStream {
    match property.optional {
        true => {
            generate_optional_type(
                &property.property_type,
                format!("{}{}", name, &property.name.to_case(Case::Pascal)),
            )
        }
        false => {
            generate_required_type(
                &property.property_type,
                Some(format!("{}{}", name, &property.name.to_case(Case::Pascal))),
            )
        }
    }
}

fn generate_optional_type(property_type: &PropertyType, union_name: String) -> proc_macro2::TokenStream {
    let inner_type = generate_required_type(property_type, Some(union_name));
    quote!(Option<#inner_type>)
}

fn generate_required_type(property_type: &PropertyType, union_name: Option<String>) -> proc_macro2::TokenStream {
    // let enum_item_type = syn::Ident::new(&enum_item_type, proc_macro2::Span::call_site());
    match property_type {
        PropertyType::String => syn::Ident::new("String", proc_macro2::Span::call_site()).into_token_stream(),
        PropertyType::Number => syn::Ident::new("f64", proc_macro2::Span::call_site()).into_token_stream(),
        PropertyType::Boolean => syn::Ident::new("bool", proc_macro2::Span::call_site()).into_token_stream(),
        PropertyType::Function { .. } => panic!("client doesn't know about functions in properties"),
        PropertyType::Component { reference } => {
            let name = format!("{}Widget", reference.component_name.to_string());
            syn::Ident::new(&name, proc_macro2::Span::call_site()).into_token_stream()
        }
        PropertyType::SharedTypeRef { name } => {
            syn::Ident::new(name, proc_macro2::Span::call_site()).into_token_stream()
        }
        PropertyType::Union { .. } => {
            match union_name {
                None => panic!("should not be used"),
                Some(union_name) => syn::Ident::new(&union_name, proc_macro2::Span::call_site()).into_token_stream(),
            }
        }
        PropertyType::Array { item } => {
            let inner_type = generate_required_type(item, union_name);
            quote!(Vec<#inner_type>)
        }
    }
}
