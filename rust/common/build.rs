use std::env;
use std::fs::File;
use std::io::Write;
use std::ops::Deref;
use std::path::Path;
use gauntlet_component_model::{create_component_model, Arity, Children, Component, ComponentName, ComponentRef, Property, PropertyKind, PropertyType, SharedType};
use itertools::Itertools;

use convert_case::{Case, Casing};
use indexmap::IndexMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile_protos(
            &["./../../schema/backend.proto"],
            &["./../../schema/"],
        )?;

    component_model_generator()?;

    Ok(())
}

fn component_model_generator() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = env::var("OUT_DIR")?;
    let dest_path = Path::new(&out_dir).join("components.rs");

    let mut output = String::new();

    let components = create_component_model();

    for component in &components {
        match component {
            Component::Standard { name, props, children, .. } => {
                let props_has_content = props.iter().any(|prop| matches!(prop.property_type.kind(), PropertyKind::Component));

                let children_has_content = match children {
                    Children::Members { ordered_members, per_type_members, .. } | Children::StringOrMembers { ordered_members, per_type_members, .. } => {
                        !ordered_members.is_empty() || !per_type_members.is_empty()
                    }
                    _ => false
                };

                let has_text = matches!(children, Children::StringOrMembers { .. } | Children::String { .. });

                let has_content = children_has_content || props_has_content || has_text;

                let default = IndexMap::new();

                let (ordered_members, per_type_members) = match children {
                    Children::Members { ordered_members, per_type_members, .. } | Children::StringOrMembers { ordered_members, per_type_members, .. } => {
                        (ordered_members, per_type_members)
                    }
                    _ => (&default, &default)
                };

                if !ordered_members.is_empty() {
                    output.push_str("#[derive(Debug, Encode, Decode)]\n");
                    output.push_str(&format!("pub enum {}WidgetOrderedMembers {{\n", name));

                    let unique_component_refs = ordered_members
                        .iter()
                        .map(|(_member_name, component_ref)| component_ref)
                        .unique_by(|component_ref| component_ref.component_name.clone())
                        .collect::<Vec<_>>();

                    for component_ref in &unique_component_refs {
                        output.push_str(&format!("    {}({}Widget),\n", component_ref.component_name, component_ref.component_name));
                    }

                    output.push_str("}\n");
                }

                if has_content {
                    {
                        output.push_str("#[derive(Debug, Encode, Decode)]\n");
                        output.push_str(&format!("pub struct {}WidgetContent {{\n", name));

                        for prop in props {
                            if matches!(prop.property_type.kind(), PropertyKind::Component) {
                                let is_union = match &prop.property_type {
                                    PropertyType::Union { .. } => true,
                                    PropertyType::Array { item } => matches!(item.as_ref(), PropertyType::Union { .. }),
                                    _ => false
                                };

                                if is_union {
                                    output.push_str(&format!("    pub {}: {},\n", prop.name.to_case(Case::Snake), generate_required_type(&prop.property_type, Some(format!("{}{}", name, &prop.name.to_case(Case::Pascal))))));
                                } else {
                                    output.push_str(&format!("    pub {}: {},\n", prop.name.to_case(Case::Snake), generate_type(&prop, name)));
                                }
                            }
                        }

                        for (member_name, component_ref) in per_type_members {
                            match component_ref.arity {
                                Arity::ZeroOrOne => {
                                    output.push_str(&format!("    pub {}: Option<{}Widget>,\n", member_name.to_case(Case::Snake), component_ref.component_name));
                                }
                                Arity::One => {
                                    output.push_str(&format!("    pub {}: {}Widget,\n", member_name.to_case(Case::Snake), component_ref.component_name));
                                }
                                Arity::ZeroOrMore => {
                                    todo!()
                                }
                            }
                        }

                        if !ordered_members.is_empty() {
                            output.push_str(&format!("    pub ordered_members: Vec<{}WidgetOrderedMembers>,\n", name));
                        }

                        if has_text {
                            output.push_str("    pub text: Vec<String>,\n");
                        }

                        output.push_str("}\n");
                    }

                    {
                        let unique_ordered_component_refs = ordered_members
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
                                _ => false
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
                                        PropertyType::Array { item } => all_component_refs(item)
                                    }
                                }

                                prop_union_component_refs.insert(prop_name, all_component_refs(&prop.property_type));
                            } else {
                                match &prop.property_type {
                                    PropertyType::Component { reference } => {
                                        prop_other_component_refs.insert(prop_name, reference);
                                    },
                                    PropertyType::Array { item, .. } => {
                                        match item.as_ref() {
                                            PropertyType::Component { reference } => {
                                                prop_other_component_refs.insert(prop_name, reference);

                                            },
                                            _ => {}
                                        }
                                    },
                                    _ => {}
                                }
                            }
                        }

                        let mut component_refs = vec![];
                        component_refs.extend(unique_ordered_component_refs.clone());
                        component_refs.extend(per_type_component_refs.clone());
                        component_refs.extend(prop_other_component_refs.values());

                        {
                            output.push_str(&format!("impl<'de> Deserialize<'de> for {}WidgetContent {{\n", name));
                            output.push_str(&format!("    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>\n"));
                            output.push_str(&format!("    where\n"));
                            output.push_str(&format!("        D: Deserializer<'de>,\n"));
                            output.push_str(&format!("    {{\n"));

                            {
                                output.push_str("        #[derive(Debug, Deserialize)]\n");
                                output.push_str("        #[serde(tag = \"__type__\")]\n");
                                output.push_str(&format!("        enum {}WidgetMembersOwned {{\n", name));

                                for (_, prop_union_component_refs) in &prop_union_component_refs {
                                    for prop_union_component_ref in prop_union_component_refs {
                                        output.push_str(&format!("            #[serde(rename = \"gauntlet:{}\")]\n", prop_union_component_ref.component_internal_name));
                                        output.push_str(&format!("            {}({}Widget),\n", prop_union_component_ref.component_name, prop_union_component_ref.component_name));
                                    }
                                }

                                for component_ref in &component_refs {
                                    output.push_str(&format!("            #[serde(rename = \"gauntlet:{}\")]\n", component_ref.component_internal_name));
                                    output.push_str(&format!("            {}({}Widget),\n", component_ref.component_name, component_ref.component_name));
                                }

                                if has_text {
                                    output.push_str(&format!("            #[serde(rename = \"gauntlet:text_part\")]\n"));
                                    output.push_str(&format!("            Text {{\n"));
                                    output.push_str(&format!("                value: String\n"));
                                    output.push_str(&format!("            }},\n"));
                                }

                                output.push_str("        }\n");
                            }

                            output.push_str(&format!("        let mut members = Vec::<{}WidgetMembersOwned>::deserialize(deserializer)?;\n", name));
                            output.push_str("\n");

                            for (prop_name, _) in &prop_other_component_refs {
                                output.push_str(&format!("        let mut {}: Option<_> = None;\n", prop_name));
                            }

                            for (prop_name, _) in &prop_union_component_refs {
                                output.push_str(&format!("        let mut {}: Vec<_> = vec![];\n", prop_name));
                            }

                            for per_type_component_ref in &per_type_component_refs {
                                output.push_str(&format!("        let mut {}: Option<_> = None;\n", per_type_component_ref.component_internal_name));
                            }

                            if !ordered_members.is_empty() {
                                output.push_str("        let mut ordered_members = vec![];\n");
                            }

                            if has_text {
                                output.push_str("        let mut text = vec![];\n");
                            }

                            output.push_str("\n");

                            if has_content {
                                output.push_str("        while let Some(member) = members.pop() {\n");
                                output.push_str("            match member {\n");

                                for (prop_name, prop_union_component_refs) in &prop_union_component_refs {
                                    for (index, prop_union_component_ref) in prop_union_component_refs.iter().enumerate() {
                                        output.push_str(&format!("                {}WidgetMembersOwned::{}(widget) => {{\n", name, prop_union_component_ref.component_name));
                                        output.push_str(&format!("                    {}.push({}{}::_{}(widget));\n", prop_name, name, prop_name.to_case(Case::Pascal), index));
                                        output.push_str(&format!("                }}\n"));
                                    }
                                }

                                for (prop_name, prop_other_component_refs) in &prop_other_component_refs {
                                    output.push_str(&format!("                {}WidgetMembersOwned::{}(widget) => {{\n", name, prop_other_component_refs.component_name));
                                    output.push_str(&format!("                    if let Some(_) = {} {{\n", prop_name));
                                    output.push_str(&format!("                        return Err(Error::custom(\"Only one {} is allowed\"))\n", prop_other_component_refs.component_name));
                                    output.push_str(&format!("                    }}\n"));
                                    output.push_str(&format!("                    {} = Some(widget);\n", prop_name));
                                    output.push_str(&format!("                }}\n"));
                                }

                                for per_type_component_ref in &per_type_component_refs {
                                    output.push_str(&format!("                {}WidgetMembersOwned::{}(widget) => {{\n", name, per_type_component_ref.component_name));
                                    output.push_str(&format!("                    if let Some(_) = {} {{\n", per_type_component_ref.component_internal_name));
                                    output.push_str(&format!("                        return Err(Error::custom(\"Only one {} is allowed\"))\n", per_type_component_ref.component_name));
                                    output.push_str(&format!("                    }}\n"));
                                    output.push_str(&format!("                    {} = Some(widget);\n", per_type_component_ref.component_internal_name));
                                    output.push_str(&format!("                }}\n"));
                                }

                                for ordered_component_ref in &unique_ordered_component_refs {
                                    output.push_str(&format!("                {}WidgetMembersOwned::{}(widget) => {{\n", name, ordered_component_ref.component_name));
                                    output.push_str(&format!("                    ordered_members.insert(0, {}WidgetOrderedMembers::{}(widget));\n", name, ordered_component_ref.component_name));
                                    output.push_str(&format!("                }}\n"));
                                }

                                if has_text {
                                    output.push_str(&format!("                {}WidgetMembersOwned::Text {{ value }} => {{\n", name));
                                    output.push_str(&format!("                    text.insert(0, value);\n"));
                                    output.push_str(&format!("                }}\n"));
                                }

                                output.push_str("            }\n");
                                output.push_str("        }\n");
                            }

                            output.push_str("\n");
                            output.push_str(&format!("        Ok({}WidgetContent {{\n", name));

                            for (prop_name, _) in &prop_union_component_refs {
                                output.push_str(&format!("            {},\n", prop_name));
                            }

                            for per_type_component_ref in &per_type_component_refs {
                                match per_type_component_ref.arity {
                                    Arity::ZeroOrOne => {
                                        output.push_str(&format!("            {},\n", per_type_component_ref.component_internal_name));
                                    }
                                    Arity::One => {
                                        output.push_str(&format!("            {}: {}.ok_or(Error::custom(\"{} is required\"))?,\n", per_type_component_ref.component_internal_name, per_type_component_ref.component_internal_name, per_type_component_ref.component_name));
                                    }
                                    Arity::ZeroOrMore => {
                                        todo!()
                                    }
                                }
                            }

                            for (prop_name, prop_other_component_ref) in &prop_other_component_refs {
                                match prop_other_component_ref.arity {
                                    Arity::ZeroOrOne => {
                                        output.push_str(&format!("            {},\n", prop_name));
                                    }
                                    Arity::One => {
                                        output.push_str(&format!("            {}: {}.ok_or(Error::custom(\"{} is required\"))?,\n", prop_name, prop_other_component_ref.component_internal_name, prop_other_component_ref.component_name));
                                    }
                                    Arity::ZeroOrMore => {
                                        todo!()
                                    }
                                }
                            }

                            if !ordered_members.is_empty() {
                                output.push_str("            ordered_members\n");
                            }

                            if has_text {
                                output.push_str("            text\n");
                            }

                            output.push_str(&format!("        }})\n"));
                            output.push_str(&format!("    }}\n"));
                            output.push_str(&format!("}}\n"));
                        }

                        {

                            output.push_str(&format!("impl Serialize for {}WidgetContent {{\n", name));
                            output.push_str(&format!("    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>\n"));
                            output.push_str(&format!("    where\n"));
                            output.push_str(&format!("        S: Serializer\n"));
                            output.push_str(&format!("    {{\n"));

                            {
                                output.push_str("        #[derive(Debug, Serialize)]\n");
                                output.push_str("        #[serde(tag = \"__type__\")]\n");
                                output.push_str(&format!("        enum {}WidgetMembersRef<'a> {{\n", name));

                                for (_, prop_union_component_refs) in &prop_union_component_refs {
                                    for prop_union_component_ref in prop_union_component_refs {
                                        output.push_str(&format!("            #[serde(rename = \"gauntlet:{}\")]\n", prop_union_component_ref.component_internal_name));
                                        output.push_str(&format!("            {}(&'a {}Widget),\n", prop_union_component_ref.component_name, prop_union_component_ref.component_name));
                                    }
                                }

                                for component_ref in &component_refs {
                                    output.push_str(&format!("            #[serde(rename = \"gauntlet:{}\")]\n", component_ref.component_internal_name));
                                    output.push_str(&format!("            {}(&'a {}Widget),\n", component_ref.component_name, component_ref.component_name));
                                }

                                if has_text {
                                    output.push_str(&format!("            #[serde(rename = \"gauntlet:text_part\")]\n"));
                                    output.push_str(&format!("            Text {{\n"));
                                    output.push_str(&format!("                value: &'a String\n"));
                                    output.push_str(&format!("            }},\n"));
                                }

                                output.push_str("        }\n");
                            }

                            output.push_str(&format!("        let mut members = Vec::<{}WidgetMembersRef>::new();\n", name));
                            output.push_str("\n");


                            for (prop_name, prop_union_component_refs) in &prop_union_component_refs {
                                output.push_str(&format!("        for item in &self.{} {{\n", prop_name));
                                output.push_str(&format!("            match item {{\n"));

                                for (index, prop_union_component_ref) in prop_union_component_refs.iter().enumerate() {
                                    output.push_str(&format!("                {}{}::_{}(widget) => {{\n", name, prop_name.to_case(Case::Pascal), index));
                                    output.push_str(&format!("                    members.push({}WidgetMembersRef::{}(widget));\n", name, prop_union_component_ref.component_name));
                                    output.push_str(&format!("                }}\n"));
                                }

                                output.push_str(&format!("            }}\n"));
                                output.push_str(&format!("        }}\n"));
                            }

                            for (prop_name, prop_other_component_refs) in &prop_other_component_refs {
                                match prop_other_component_refs.arity {
                                    Arity::ZeroOrOne => {
                                        output.push_str(&format!("        if let Some({}) = &self.{} {{\n", prop_name, prop_name));
                                        output.push_str(&format!("            members.push({}WidgetMembersRef::{}({}))\n", name, prop_other_component_refs.component_name, prop_name));
                                        output.push_str(&format!("        }}\n"));
                                    }
                                    Arity::One => {
                                        output.push_str(&format!("        members.push({}WidgetMembersRef::{}(&self.{}))\n", name, prop_other_component_refs.component_name, prop_name));
                                    }
                                    Arity::ZeroOrMore => {
                                        todo!()
                                    }
                                }
                            }

                            for per_type_component_ref in &per_type_component_refs {
                                match per_type_component_ref.arity {
                                    Arity::ZeroOrOne => {
                                        output.push_str(&format!("        if let Some(item) = &self.{} {{\n", per_type_component_ref.component_internal_name));
                                        output.push_str(&format!("            members.push({}WidgetMembersRef::{}(item))\n", name, per_type_component_ref.component_name));
                                        output.push_str(&format!("        }}\n"));
                                    }
                                    Arity::One => {
                                        output.push_str(&format!("        members.push({}WidgetMembersRef::{}(&self.{}));\n", name, per_type_component_ref.component_name, per_type_component_ref.component_internal_name));
                                    }
                                    Arity::ZeroOrMore => {
                                        todo!()
                                    }
                                }
                            }

                            if !unique_ordered_component_refs.is_empty() {
                                output.push_str(&format!("        for member in &self.ordered_members {{\n"));
                                output.push_str(&format!("            match member {{\n"));

                                for ordered_component_ref in &unique_ordered_component_refs {
                                    output.push_str(&format!("                {}WidgetOrderedMembers::{}(widget) => {{\n", name, ordered_component_ref.component_name));
                                    output.push_str(&format!("                    members.push({}WidgetMembersRef::{}(widget))\n", name, ordered_component_ref.component_name));
                                    output.push_str(&format!("                }}\n"));
                                }

                                output.push_str(&format!("            }}\n"));
                                output.push_str(&format!("        }}\n"));
                            }

                            if has_text {
                                output.push_str(&format!("        for value in &self.text {{\n"));
                                output.push_str(&format!("            members.push({}WidgetMembersRef::Text {{ value }});\n", name));
                                output.push_str(&format!("        }}\n"));
                            }

                            output.push_str("\n");
                            output.push_str(&format!("        Vec::<{}WidgetMembersRef>::serialize(&members, serializer)\n", name));

                            output.push_str(&format!("    }}\n"));
                            output.push_str(&format!("}}\n"));
                        }
                    }
                }

                output.push_str("#[derive(Debug, Serialize, Deserialize, Encode, Decode)]\n");
                output.push_str(&format!("pub struct {}Widget {{\n", name));
                output.push_str("    #[serde(rename = \"__id__\")]\n");
                output.push_str("    pub __id__: UiWidgetId,\n");

                for prop in props {
                    if matches!(prop.property_type.kind(), PropertyKind::Property) {
                        output.push_str(&format!("    #[serde(rename = \"{}\")]\n", prop.name));
                        output.push_str(&format!("    pub {}: {},\n", prop.name.to_case(Case::Snake), generate_type(&prop, name)));
                    }
                }

                if has_content {
                    output.push_str(&format!("    pub content: {}WidgetContent,\n", name));
                }

                output.push_str("}\n");

                let generate_union = |output: &mut String, items: &Vec<PropertyType>, prop_name: &String| {
                    output.push_str("#[derive(Debug, Encode, Decode)]\n");
                    output.push_str(&format!("pub enum {}{} {{\n", name, prop_name.to_case(Case::Pascal)));

                    for (index, property_type) in items.iter().enumerate() {
                        output.push_str(&format!("    _{}({}),\n", index, generate_required_type(&property_type, None)));
                    }

                    output.push_str("}\n");
                    output.push_str("\n");
                };

                for prop in props {
                    match &prop.property_type {
                        PropertyType::Union { items } => {
                            generate_union(&mut output, items, &prop.name)
                        }
                        PropertyType::Array { item} => {
                            match item.deref() {
                                PropertyType::Union { items } => {
                                    generate_union(&mut output, items, &prop.name)
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
            }
            Component::Root { children, shared_types, .. } => {
                for (type_name, shared_type) in shared_types {
                    match shared_type {
                        SharedType::Enum { items } => {
                            output.push_str("#[derive(Debug, Serialize, Deserialize, Encode, Decode)]\n");
                            output.push_str(&format!("pub enum {} {{\n", type_name));

                            for item in items {
                                output.push_str(&format!("    #[serde(rename = \"{}\")]\n", &item));
                                output.push_str(&format!("    {},\n", &item));
                            }

                            output.push_str("}\n");
                            output.push_str("\n");
                        }
                        SharedType::Object { items } => {
                            output.push_str("#[derive(Debug, Serialize, Deserialize, Encode, Decode)]\n");
                            output.push_str(&format!("pub struct {} {{\n", type_name));

                            for (property_name, property_type) in items {
                                output.push_str(&format!("    pub {}: {},\n", &property_name, generate_required_type(&property_type, Some(format!("{}{}", type_name, property_name)))));
                            }

                            output.push_str("}\n");
                            output.push_str("\n");
                        }
                        SharedType::Union { items } => {
                            output.push_str("#[derive(Debug, Serialize, Deserialize, Encode, Decode)]\n");
                            output.push_str("#[serde(untagged)]\n");
                            output.push_str(&format!("pub enum {} {{\n", type_name));

                            for property_type in items {
                                match property_type {
                                    PropertyType::SharedTypeRef { name } => {
                                        output.push_str(&format!("    {}({}),\n", &name, name));
                                    }
                                    _ => {
                                        todo!()
                                    }
                                }
                            }

                            output.push_str("}\n");
                            output.push_str("\n");
                        }
                    }
                }

                output.push_str("#[derive(Debug, Serialize, Deserialize, Encode, Decode)]\n");
                output.push_str("#[serde(tag = \"__type__\")]\n");
                output.push_str("pub enum RootWidgetMembers {\n");

                for component_ref in children {
                    output.push_str(&format!("    #[serde(rename = \"gauntlet:{}\")]\n", component_ref.component_internal_name));
                    output.push_str(&format!("    {}({}Widget),\n", component_ref.component_name, component_ref.component_name));
                }

                output.push_str("}\n");

                output.push_str("#[derive(Debug, Serialize, Deserialize, Encode, Decode)]\n");
                output.push_str("pub struct RootWidget {\n");
                output.push_str("    #[serde(default, deserialize_with = \"array_to_option\")]\n");
                output.push_str("    pub content: Option<RootWidgetMembers>\n");
                output.push_str("}\n");
            }
            Component::TextPart { .. } => {}
        }
    }

    generate_file(&dest_path, &output)?;

    Ok(())
}

fn generate_file<P: AsRef<Path>>(path: P, text: &str) -> std::io::Result<()> {
    let mut f = File::create(path)?;
    f.write_all(text.as_bytes())
}

fn generate_type(property: &Property, name: &ComponentName) -> String {
    match property.optional {
        true => generate_optional_type(&property.property_type, format!("{}{}", name, &property.name.to_case(Case::Pascal))),
        false => generate_required_type(&property.property_type, Some(format!("{}{}", name, &property.name.to_case(Case::Pascal))))
    }
}

fn generate_optional_type(property_type: &PropertyType, union_name: String) -> String {
    format!("Option<{}>", generate_required_type(property_type, Some(union_name)))
}

fn generate_required_type(property_type: &PropertyType, union_name: Option<String>) -> String {
    match property_type {
        PropertyType::String => "String".to_owned(),
        PropertyType::Number => "f64".to_owned(),
        PropertyType::Boolean => "bool".to_owned(),
        PropertyType::Function { .. } => panic!("client doesn't know about functions in properties"),
        PropertyType::Component { reference } => format!("{}Widget", reference.component_name.to_string()),
        PropertyType::SharedTypeRef { name } => name.to_owned(),
        PropertyType::Union { .. } => {
            match union_name {
                None => panic!("should not be used"),
                Some(union_name) => union_name
            }
        },
        PropertyType::Array { item } => format!("Vec<{}>", generate_required_type(item, union_name))
    }
}
