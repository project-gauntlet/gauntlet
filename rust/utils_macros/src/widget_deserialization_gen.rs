use convert_case::Case;
use convert_case::Casing;
use gauntlet_component_model::Arity;
use gauntlet_component_model::Children;
use gauntlet_component_model::Component;
use gauntlet_component_model::ComponentRef;
use gauntlet_component_model::OptionalKind;
use gauntlet_component_model::PropertyKind;
use gauntlet_component_model::PropertyType;
use gauntlet_component_model::SharedType;
use gauntlet_component_model::create_component_model;
use indexmap::IndexMap;
use itertools::Itertools;
use proc_macro::TokenStream;
use quote::quote;

pub fn widget_deserialization_gen() -> TokenStream {
    let components = create_component_model();

    let mut output_components = vec![];

    for component in &components {
        match component {
            Component::Standard {
                internal_name,
                name,
                props,
                children,
                ..
            } => {
                let name = name.to_string();
                let props_has_content = props
                    .iter()
                    .any(|prop| matches!(prop.property_type.kind(), PropertyKind::Component));

                let has_props = props
                    .iter()
                    .any(|prop| !matches!(prop.property_type.kind(), PropertyKind::Component));

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

                let deserialize_method = format!("deserialize_{}_widget", internal_name);
                let deserialize_method = syn::Ident::new(&deserialize_method, proc_macro2::Span::call_site());

                let deserialize_content_method = format!("deserialize_{}_widget_content", internal_name);
                let deserialize_content_method =
                    syn::Ident::new(&deserialize_content_method, proc_macro2::Span::call_site());

                let (props_names, widget_properties) = if has_props {
                    let props_names: Vec<_> = props
                        .iter()
                        .filter(|prop| matches!(prop.property_type.kind(), PropertyKind::Property))
                        .map(|prop| {
                            let name = syn::Ident::new(&prop.name.to_case(Case::Snake), proc_macro2::Span::call_site());
                            quote! {
                                #name,
                            }
                        })
                        .collect();

                    let props_deserialization: Vec<_> = props
                        .iter()
                        .map(|prop| {
                            let prop_name = &prop.name;

                            let prop_var_name = prop_name.to_case(Case::Snake);
                            let prop_var_name = syn::Ident::new(&prop_var_name, proc_macro2::Span::call_site());

                            match prop.property_type.kind() {
                                PropertyKind::Event | PropertyKind::Component => {
                                    quote! {
                                        keys.retain(|item| *item != #prop_name);
                                    }
                                }
                                PropertyKind::Property => {
                                    let deserialize_prop = match &prop.property_type {
                                        PropertyType::String => {
                                            quote! {
                                                deserialize_string(scope, property_value)?
                                            }
                                        }
                                        PropertyType::Number => {
                                            quote! {
                                                deserialize_number(property_value)?
                                            }
                                        }
                                        PropertyType::Boolean => {
                                            quote! {
                                                deserialize_boolean(property_value)?
                                            }
                                        }
                                        PropertyType::Component { .. } => panic!(),
                                        PropertyType::Function { .. } => panic!(),
                                        PropertyType::SharedTypeRef { name } => {
                                            let method_name = format!("deserialize_shared_type_{}", name.to_case(Case::Snake));
                                            let method_name = syn::Ident::new(&method_name, proc_macro2::Span::call_site());

                                            quote! {
                                                #method_name(scope, property_value)?
                                            }
                                        }
                                        PropertyType::Union { .. } | PropertyType::Array { .. } => {
                                            panic!() // only property kind property is used here
                                        }
                                    };

                                    match prop.optional {
                                        OptionalKind::No => {
                                            quote! {
                                                keys.retain(|item| *item != #prop_name);
                                                let #prop_var_name = match extract_object_value(scope, widget_properties, #prop_name) {
                                                    Some(property_value) => #deserialize_prop,
                                                    None => {
                                                        return Err(Error::required_prop(#prop_name))
                                                    },
                                                };
                                            }
                                        }
                                        OptionalKind::Yes => {
                                            quote! {
                                                keys.retain(|item| *item != #prop_name);
                                                let #prop_var_name = match extract_object_value(scope, widget_properties, #prop_name) {
                                                    Some(property_value) => Some(#deserialize_prop),
                                                    None => None,
                                                };
                                            }
                                        }
                                        OptionalKind::YesButComplicated => {
                                            quote! {
                                                keys.retain(|item| *item != #prop_name);

                                                let key = v8::String::new(scope, #prop_name).unwrap();
                                                let value = widget_properties.get(scope, key.into());

                                                let #prop_var_name = match value {
                                                    Some(property_value) => {
                                                        if property_value.is_undefined() {
                                                            JsOption::Undefined
                                                        } else if property_value.is_null() {
                                                            JsOption::Null
                                                        } else {
                                                            JsOption::Value(#deserialize_prop)
                                                        }
                                                    }
                                                    None => JsOption::Undefined
                                                };
                                            }
                                        }
                                    }
                                }
                            }
                        })
                        .collect();

                    let widget_properties = quote! {
                        let widget_properties = extract_object_value(scope, container, "widgetProperties")
                            .ok_or(error_internal!("'widgetProperties' field is not present on widget object"))?;

                        let widget_properties: v8::Local<v8::Object> = widget_properties
                            .try_into()
                            .map_err(|_| error_internal!("invalid 'widget_properties', expected 'object', got: '{}'", widget_properties.type_repr()))?;

                        let mut keys = extract_object_keys(scope, widget_properties)
                            .map_err(|err| error_internal_source!(err, "unable to get keys of 'widget_properties' object"))?;

                        #(#props_deserialization)*

                        match keys.len() {
                            0 => {}
                            _ => return Err(Error::unknown_prop(keys)),
                        }
                    };

                    (props_names, Some(widget_properties))
                } else {
                    (vec![], None)
                };

                let (widget_children, content) = if has_content {
                    let widget_children = quote! {
                        let widget_children = extract_object_value(scope, container, "widgetChildren")
                            .ok_or(error_internal!("'widgetChildren' field is not present on widget object"))?;

                        let widget_children: v8::Local<v8::Array> = widget_children
                            .try_into()
                            .map_err(|_| error_internal!("invalid 'widgetChildren', expected 'array', got: '{}'", widget_children.type_repr()))?;
                    };

                    let content = quote! {
                        content: #deserialize_content_method(scope, widget_children)
                            .map_err(Error::inside(#name))?,
                    };

                    (Some(widget_children), Some(content))
                } else {
                    (None, None)
                };

                let widget_struct_name = format!("{}Widget", name);
                let widget_struct_name = syn::Ident::new(&widget_struct_name, proc_macro2::Span::call_site());

                output_components.push(quote! {
                    fn #deserialize_method(
                        scope: &mut v8::HandleScope,
                        container: v8::Local<v8::Object>,
                    ) -> Result<#widget_struct_name> {
                        let widget_id = extract_object_value(scope, container, "widgetId")
                            .ok_or(error_internal!("'widgetId' field is not present on widget object"))?;

                        let widget_id: v8::Local<v8::Number> = widget_id
                            .try_into()
                            .map_err(|_| error_internal!("invalid 'widgetId', expected 'string', got: '{}'", widget_id.type_repr()))?;

                        let widget_id = widget_id.value() as usize;

                        #widget_children

                        #widget_properties

                        Ok(#widget_struct_name {
                            __id__: widget_id,
                            #(#props_names)*
                            #content
                        })
                    }
                });

                if has_content {
                    let ordered_component_refs = ordered_members
                        .iter()
                        .map(|(_member_name, component_ref)| component_ref)
                        .unique_by(|component_ref| component_ref.component_name.clone())
                        .collect::<Vec<_>>();

                    let per_type_component_refs = per_type_members
                        .iter()
                        .map(|(_member_name, component_ref)| component_ref)
                        .collect::<Vec<_>>();

                    let mut prop_union_component_refs = IndexMap::new();
                    let mut prop_other_component_refs = IndexMap::new();

                    for prop in props {
                        let is_union = match &prop.property_type {
                            PropertyType::Union { .. } => true,
                            PropertyType::Array { item } => matches!(item.as_ref(), PropertyType::Union { .. }),
                            _ => false,
                        };

                        let prop_name = prop.name.to_case(Case::Snake);

                        if is_union {
                            fn all_component_refs(property_type: &PropertyType) -> Vec<&ComponentRef> {
                                match property_type {
                                    PropertyType::String => vec![],
                                    PropertyType::Number => vec![],
                                    PropertyType::Boolean => vec![],
                                    PropertyType::Component { reference } => vec![reference],
                                    PropertyType::Function { .. } => vec![],
                                    PropertyType::SharedTypeRef { .. } => vec![],
                                    PropertyType::Union { items } => {
                                        items.iter().flat_map(|prop| all_component_refs(prop)).collect()
                                    }
                                    PropertyType::Array { item } => all_component_refs(item),
                                }
                            }

                            prop_union_component_refs.insert(prop_name, all_component_refs(&prop.property_type));
                        } else {
                            match &prop.property_type {
                                PropertyType::Component { reference } => {
                                    prop_other_component_refs.insert(prop_name, reference);
                                }
                                PropertyType::Array { item, .. } => {
                                    match item.as_ref() {
                                        PropertyType::Component { reference } => {
                                            prop_other_component_refs.insert(prop_name, reference);
                                        }
                                        _ => {}
                                    }
                                }
                                _ => {}
                            }
                        }
                    }

                    let union_component_props: Vec<_> = prop_union_component_refs
                        .iter()
                        .map(|(prop_name, _)| {
                            let prop_name = syn::Ident::new(&prop_name, proc_macro2::Span::call_site());

                            quote! {
                                let mut #prop_name: Vec<_> = vec![];
                            }
                        })
                        .collect();

                    let union_component_props_match_arms: Vec<_> = prop_union_component_refs
                        .iter()
                        .flat_map(|(prop_name, prop_union_component_refs)| {
                            prop_union_component_refs
                                .iter()
                                .enumerate()
                                .map(|(index, prop_union_component_ref)| {
                                    let id = &prop_union_component_ref.component_internal_name;
                                    let react_id = format!("gauntlet:{}", id);

                                    let deserialize_method_name = format!("deserialize_{}_widget", id);
                                    let deserialize_method_name =
                                        syn::Ident::new(&deserialize_method_name, proc_macro2::Span::call_site());

                                    let union_enum_name = format!("{}{}", name, prop_name.to_case(Case::Pascal));
                                    let union_enum_name =
                                        syn::Ident::new(&union_enum_name, proc_macro2::Span::call_site());

                                    let union_enum_item_name = format!("_{}", index);
                                    let union_enum_item_name =
                                        syn::Ident::new(&union_enum_item_name, proc_macro2::Span::call_site());

                                    let prop_name = syn::Ident::new(&prop_name, proc_macro2::Span::call_site());

                                    quote! {
                                        #react_id => {
                                            let widget = #deserialize_method_name(scope, container)?;

                                            #prop_name.push(#union_enum_name::#union_enum_item_name(widget))
                                        }
                                    }
                                })
                                .collect::<Vec<_>>()
                        })
                        .collect();

                    let prop_union_results: Vec<_> = prop_union_component_refs
                        .iter()
                        .map(|(prop_name, _)| {
                            let prop_name = syn::Ident::new(&prop_name, proc_macro2::Span::call_site());
                            quote! {
                                #prop_name,
                            }
                        })
                        .collect();

                    let other_component_props: Vec<_> = prop_other_component_refs
                        .iter()
                        .map(|(prop_name, _)| {
                            let prop_name = syn::Ident::new(&prop_name, proc_macro2::Span::call_site());

                            quote! {
                                let mut #prop_name: Option<_> = None;
                            }
                        })
                        .collect();

                    let other_component_props_match_arms: Vec<_> = prop_other_component_refs
                        .iter()
                        .map(|(prop_name, prop_other_component_ref)| {
                            let name = &prop_other_component_ref.component_name.to_string();
                            let id = &prop_other_component_ref.component_internal_name;
                            let react_id = format!("gauntlet:{}", id);
                            let prop_name = syn::Ident::new(&prop_name, proc_macro2::Span::call_site());

                            let deserialize_method_name = format!("deserialize_{}_widget", id);
                            let deserialize_method_name =
                                syn::Ident::new(&deserialize_method_name, proc_macro2::Span::call_site());

                            quote! {
                                #react_id => {
                                    if let Some(_) = #prop_name {
                                        return Err(Error::single_component(#name));
                                    }

                                    let widget = #deserialize_method_name(scope, container)?;

                                    #prop_name = Some(widget);
                                }
                            }
                        })
                        .collect();

                    let prop_other_results: Vec<_> = prop_other_component_refs
                        .iter()
                        .map(|(prop_name, prop_other_component_ref)| {
                            let prop_name = syn::Ident::new(&prop_name, proc_macro2::Span::call_site());
                            match prop_other_component_ref.arity {
                                Arity::ZeroOrOne => {
                                    quote! {
                                        #prop_name,
                                    }
                                }
                                Arity::One => {
                                    let component_name = &prop_other_component_ref.component_name.to_string();
                                    quote! {
                                        #prop_name: #prop_name.ok_or(Error::required_component(#component_name))?,
                                    }
                                }
                                Arity::ZeroOrMore => {
                                    unimplemented!()
                                }
                            }
                        })
                        .collect();

                    let per_type_component_props: Vec<_> = per_type_component_refs
                        .iter()
                        .map(|per_type_component_ref| {
                            let prop_name = &per_type_component_ref.component_internal_name;
                            let prop_name = syn::Ident::new(prop_name, proc_macro2::Span::call_site());

                            quote! {
                                let mut #prop_name: Option<_> = None;
                            }
                        })
                        .collect();

                    let per_type_component_props_match_arms: Vec<_> = per_type_component_refs
                        .iter()
                        .map(|prop_other_component_ref| {
                            let name = &prop_other_component_ref.component_name.to_string();
                            let id = &prop_other_component_ref.component_internal_name;
                            let react_id = format!("gauntlet:{}", id);
                            let prop_name = syn::Ident::new(&id, proc_macro2::Span::call_site());

                            let deserialize_method_name = format!("deserialize_{}_widget", id);
                            let deserialize_method_name =
                                syn::Ident::new(&deserialize_method_name, proc_macro2::Span::call_site());

                            quote! {
                                #react_id => {
                                    if let Some(_) = #prop_name {
                                        return Err(Error::single_component(#name));
                                    }

                                    let widget = #deserialize_method_name(scope, container)?;

                                    #prop_name = Some(widget);
                                }
                            }
                        })
                        .collect();

                    let per_type_prop_results: Vec<_> = per_type_component_refs
                        .iter()
                        .map(|per_type_component_ref| {
                            let prop_name = &per_type_component_ref.component_internal_name;
                            let prop_name = syn::Ident::new(prop_name, proc_macro2::Span::call_site());
                            match per_type_component_ref.arity {
                                Arity::ZeroOrOne => {
                                    quote! {
                                        #prop_name,
                                    }
                                }
                                Arity::One => {
                                    let component_name = &per_type_component_ref.component_name.to_string();
                                    quote! {
                                        #prop_name: #prop_name.ok_or(Error::required_component(#component_name))?,
                                    }
                                }
                                Arity::ZeroOrMore => {
                                    unimplemented!()
                                }
                            }
                        })
                        .collect();

                    let (ordered_members_prop, ordered_members_prop_match_arms, ordered_members_prop_result) =
                        if !ordered_members.is_empty() {
                            let ordered_members_prop = quote! {
                                let mut ordered_members = vec![];
                            };

                            let ordered_members_prop_match_arms: Vec<_> = ordered_component_refs
                                .iter()
                                .map(|ordered_component_ref| {
                                    let id = &ordered_component_ref.component_internal_name;
                                    let react_id = format!("gauntlet:{}", id);

                                    let ordered_members_enum_name = format!("{}WidgetOrderedMembers", name);
                                    let ordered_members_enum_name =
                                        syn::Ident::new(&ordered_members_enum_name, proc_macro2::Span::call_site());

                                    let ordered_members_enum_item_name = &ordered_component_ref.component_name.to_string();
                                    let ordered_members_enum_item_name =
                                        syn::Ident::new(&ordered_members_enum_item_name, proc_macro2::Span::call_site());

                                    let deserialize_method_name = format!("deserialize_{}_widget", id);
                                    let deserialize_method_name =
                                        syn::Ident::new(&deserialize_method_name, proc_macro2::Span::call_site());

                                    quote! {
                                        #react_id => {
                                            let widget = #deserialize_method_name(scope, container)?;

                                            ordered_members.push(#ordered_members_enum_name::#ordered_members_enum_item_name(widget));
                                        }
                                    }
                                })
                                .collect();

                            let ordered_members_prop_result = quote! {
                                ordered_members,
                            };

                            (
                                Some(ordered_members_prop),
                                ordered_members_prop_match_arms,
                                Some(ordered_members_prop_result),
                            )
                        } else {
                            (None, vec![], None)
                        };

                    let (text_prop, text_prop_match_arm, text_prop_result) = if has_text {
                        let text_prop = quote! {
                            let mut text = vec![];
                        };

                        let text_react_id = "gauntlet:text_part";
                        let text_prop_match_arm = quote! {
                            #text_react_id => {
                                let widget_text = deserialize_text_widget(scope, container)?;

                                text.push(widget_text);
                            }
                        };

                        let text_prop_result = quote! {
                            text,
                        };

                        (Some(text_prop), Some(text_prop_match_arm), Some(text_prop_result))
                    } else {
                        (None, None, None)
                    };

                    let mut expected_types = vec![];

                    for (_, prop_union_component_refs) in prop_union_component_refs {
                        for prop_union_component_ref in prop_union_component_refs {
                            expected_types.push(prop_union_component_ref.component_internal_name.to_string());
                        }
                    }

                    for (_, prop_other_component_refs) in prop_other_component_refs {
                        expected_types.push(prop_other_component_refs.component_internal_name.to_string());
                    }

                    for per_type_component_ref in per_type_component_refs {
                        expected_types.push(per_type_component_ref.component_internal_name.to_string());
                    }

                    for ordered_component_ref in ordered_component_refs {
                        expected_types.push(ordered_component_ref.component_internal_name.to_string());
                    }

                    if has_text {
                        expected_types.push("gauntlet:text_part".to_string())
                    }
                    let expected_types = expected_types.join(", ");

                    let widget_content_struct_name = format!("{}WidgetContent", name);
                    let widget_content_struct_name =
                        syn::Ident::new(&widget_content_struct_name, proc_macro2::Span::call_site());

                    output_components.push(quote! {
                        fn #deserialize_content_method(
                            scope: &mut v8::HandleScope,
                            widget_children: v8::Local<v8::Array>,
                        ) -> Result<#widget_content_struct_name> {
                            #(#union_component_props)*
                            #(#other_component_props)*
                            #(#per_type_component_props)*
                            #ordered_members_prop
                            #text_prop

                            for index in 0..widget_children.length() {
                                let container = widget_children
                                    .get_index(scope, index)
                                    .ok_or(error_internal!("unable to get item from 'widget_children' array at index {}", index))?;

                                let container: v8::Local<v8::Object> = container
                                    .try_into()
                                    .map_err(|_| error_internal!("invalid widget container, expected 'object', got: '{}'", container.type_repr()))?;

                                let widget_type = deserialize_widget_type(scope, container)?;

                                match widget_type.as_str() {
                                    #(#union_component_props_match_arms)*
                                    #(#other_component_props_match_arms)*
                                    #(#per_type_component_props_match_arms)*
                                    #(#ordered_members_prop_match_arms)*
                                    #text_prop_match_arm
                                    unexpected_type @ _ => {
                                         let unexpected_type = if let Some(pos) = unexpected_type.find(':') {
                                             &unexpected_type[pos + 1..]
                                         } else {
                                             unexpected_type
                                         };

                                        return Err(Error::unexpected_component(&unexpected_type, #expected_types));
                                    }
                                }
                            }

                            Ok(#widget_content_struct_name {
                                #(#prop_union_results)*
                                #(#prop_other_results)*
                                #(#per_type_prop_results)*
                                #ordered_members_prop_result
                                #text_prop_result
                            })
                        }
                    });
                }
            }
            Component::Root {
                children, shared_types, ..
            } => {
                let children_names = children.iter().map(|item| &item.component_name).join(", ");

                let match_arm: Vec<_> = children
                    .iter()
                    .map(|component| {
                        let name = &component.component_name.to_string();
                        let name_ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
                        let internal_name = &component.component_internal_name;
                        let method_name = format!("deserialize_{}_widget", internal_name);
                        let method_name = syn::Ident::new(&method_name, proc_macro2::Span::call_site());

                        let id = format!("gauntlet:{}", internal_name);

                        quote! {
                            #id => {
                                let widget = #method_name(scope, container)?;

                                RootWidgetMembers::#name_ident(widget)
                            }
                        }
                    })
                    .collect();

                output_components.push(quote! {
                    pub fn deserialize_root_widget(
                        scope: &mut v8::HandleScope,
                        container: v8::Local<v8::Value>,
                    ) -> Result<RootWidget> {
                        let container: v8::Local<v8::Object> = container.try_into()
                            .map_err(|_| error_internal!("root container is not an object"))?;

                        let content = extract_object_value(scope, container, "content")
                            .ok_or(error_internal!("'content' key is not present on root object"))?;

                        let content: v8::Local<v8::Array> = content.try_into()
                            .map_err(|_| error_internal!("'content' in root object is not an array"))?;

                        let content = match content.length() {
                            0 => None,
                            1 => {
                                let container = content
                                    .get_index(scope, 0)
                                    .ok_or(error_internal!("unable to get item from root content array at index {}", 0))?;

                                let container: v8::Local<v8::Object> = container
                                    .try_into()
                                    .map_err(|_| error_internal!("item in root content array at index {} is not an object, got: '{}'", 0, container.type_repr()))?;

                                let members = deserialize_root_widget_members(scope, container)?;

                                Some(members)
                            }
                            _ => {
                                return Err(error_internal!(
                                    "root component can only contain a single child, got: '{}'",
                                    content.length()
                                ));
                            }
                        };

                        Ok(RootWidget { content })
                    }

                    fn deserialize_root_widget_members(
                        scope: &mut v8::HandleScope,
                        container: v8::Local<v8::Object>,
                    ) -> Result<RootWidgetMembers> {
                        let widget_type = deserialize_widget_type(scope, container)?;

                        let members = match widget_type.as_str() {
                            #(#match_arm)*
                            _ => {
                                return Err(Error::unexpected_component(&widget_type, #children_names));
                            }
                        };

                        Ok(members)
                    }
                });

                let shared_types = shared_types
                    .iter()
                    .map(|(type_name, shared_type)| {
                        let method_name = format!("deserialize_shared_type_{}", type_name.to_case(Case::Snake));
                        let method_name = syn::Ident::new(&method_name, proc_macro2::Span::call_site());
                        let type_name_ident = syn::Ident::new(&type_name, proc_macro2::Span::call_site());

                        match shared_type {
                            SharedType::Enum { .. } => {
                                quote! {
                                    fn #method_name(scope: &mut v8::HandleScope, value: v8::Local<v8::Value>) -> Result<#type_name_ident> {
                                        let value: v8::Local<v8::String> = value.try_into()
                                            .map_err(|_| Error::unexpected_type(value.type_repr(), "enum (string)"))?;

                                        let value = value.to_rust_string_lossy(scope);

                                        let value = <#type_name_ident as std::str::FromStr>::from_str(&value)
                                            .map_err(|_| Error::invalid_enum_value(#type_name, &value))?;

                                        Ok(value)
                                    }
                                }
                            }
                            SharedType::Object { items } => {
                                let field_names: Vec<_> = items.iter()
                                    .map(|(name, _)| {
                                        let name = syn::Ident::new(&name, proc_macro2::Span::call_site());
                                        quote! {
                                            #name,
                                        }
                                    })
                                    .collect();

                                let fields: Vec<_> = items.iter()
                                    .map(|(name, property_type)| {
                                        let name_ident = syn::Ident::new(&name, proc_macro2::Span::call_site());

                                        let deserialize_method = match &property_type {
                                            PropertyType::String => {
                                                quote! {
                                                    deserialize_string(scope, #name_ident)?
                                                }
                                            }
                                            PropertyType::Number => {
                                                quote! {
                                                    deserialize_number(#name_ident)?
                                                }
                                            }
                                            PropertyType::Boolean => {
                                                quote! {
                                                    deserialize_boolean(#name_ident)?
                                                }
                                            }
                                            _ => {
                                                todo!()
                                            }
                                        };

                                        quote! {
                                            let #name_ident = extract_object_value(scope, value, #name)
                                                .ok_or(Error::required_prop(#name))?;

                                            let #name_ident = #deserialize_method;
                                        }
                                    })
                                    .collect();

                                quote! {
                                    fn #method_name(scope: &mut v8::HandleScope, value: v8::Local<v8::Value>) -> Result<#type_name_ident> {
                                        let value: v8::Local<v8::Object> = value
                                            .try_into()
                                            .map_err(|_| Error::unexpected_type(value.type_repr(), "object"))?;

                                        #(#fields)*

                                        Ok(#type_name_ident {
                                            #(#field_names)*
                                        })
                                    }
                                }
                            }
                            SharedType::Union { items } => {
                                let deserializers: Vec<_> = items.iter()
                                    .enumerate()
                                    .map(|(index, property_type)| {
                                        let name = match property_type {
                                            PropertyType::SharedTypeRef { name } => name,
                                            _ => todo!()
                                        };
                                        let name_ident = syn::Ident::new(&name, proc_macro2::Span::call_site());

                                        let method_name = format!("deserialize_shared_type_{}", name.to_case(Case::Snake));
                                        let method_name = syn::Ident::new(&method_name, proc_macro2::Span::call_site());

                                        if index == 0 {
                                            quote! {
                                                let result = #method_name(scope, value).map(#type_name_ident::#name_ident)
                                            }
                                        } else {
                                            quote! {
                                                .or_else(|_| #method_name(scope, value).map(#type_name_ident::#name_ident));
                                            }
                                        }
                                    })
                                    .collect();

                                quote! {
                                    fn #method_name(scope: &mut v8::HandleScope, value: v8::Local<v8::Value>) -> Result<#type_name_ident> {
                                        #(#deserializers)*

                                        Ok(result?)
                                    }
                                }
                            }
                        }
                    })
                    .collect();

                output_components.push(shared_types);
            }
            Component::TextPart { .. } => {}
        }
    }

    let result = quote! {
        #(#output_components)*
    };

    result.into()
}
