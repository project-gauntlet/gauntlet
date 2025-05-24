use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use convert_case::Case;
use convert_case::Casing;
use gauntlet_component_model::Component;
use gauntlet_component_model::ComponentName;
use gauntlet_component_model::Property;
use gauntlet_component_model::PropertyType;
use gauntlet_component_model::create_component_model;

fn main() -> anyhow::Result<()> {
    let out_dir = env::var("OUT_DIR")?;
    let dest_path = Path::new(&out_dir).join("components.rs");

    let mut output = String::new();

    let components = create_component_model();

    for component in &components {
        match component {
            Component::Standard { name, props, .. } => {
                for prop in props {
                    let PropertyType::Function { arguments } = &prop.property_type else {
                        continue;
                    };

                    output.push_str(&format!(
                        "fn create_{}_{}_event(\n",
                        name.to_string().to_case(Case::Snake),
                        prop.name.to_case(Case::Snake)
                    ));
                    output.push_str("    widget_id: UiWidgetId,\n");

                    for arg in arguments {
                        output.push_str(&"    #[allow(non_snake_case)]\n".to_string());
                        output.push_str(&format!("    {}: {}\n", arg.name, generate_type(&arg, name)));
                    }

                    output.push_str(") -> crate::model::UiViewEvent {\n");
                    output.push_str("    crate::model::UiViewEvent::View {\n");
                    output.push_str("        widget_id,\n");
                    output.push_str(&format!("        event_name: \"{}\".to_owned(),\n", prop.name));
                    output.push_str("        event_arguments: vec![\n");

                    for arg in arguments {
                        match arg.property_type {
                            PropertyType::String => {
                                if arg.optional {
                                    output.push_str(&format!("            {}.map(|#[allow(non_snake_case)] {}| gauntlet_common::model::UiPropertyValue::String({})).unwrap_or_else(|| gauntlet_common::model::UiPropertyValue::Undefined),\n", arg.name, arg.name, arg.name));
                                } else {
                                    output.push_str(&format!(
                                        "            gauntlet_common::model::UiPropertyValue::String({}),\n",
                                        arg.name
                                    ));
                                }
                            }
                            PropertyType::Number => {
                                if arg.optional {
                                    output.push_str(&format!("            {}.map(|{}| gauntlet_common::model::UiPropertyValue::Number({})).unwrap_or_else(|| gauntlet_common::model::UiPropertyValue::Undefined),\n", arg.name, arg.name, arg.name));
                                } else {
                                    output.push_str(&format!(
                                        "            gauntlet_common::model::UiPropertyValue::Number({}),\n",
                                        arg.name
                                    ));
                                }
                            }
                            PropertyType::Boolean => {
                                if arg.optional {
                                    output.push_str(&format!("            {}.map(|{}| gauntlet_common::model::UiPropertyValue::Bool({})).unwrap_or_else(|| gauntlet_common::model::UiPropertyValue::Undefined),\n", arg.name, arg.name, arg.name));
                                } else {
                                    output.push_str(&format!(
                                        "            gauntlet_common::model::UiPropertyValue::Bool({}),\n",
                                        arg.name
                                    ));
                                }
                            }
                            _ => {
                                panic!("not yet supported")
                            }
                        }
                    }

                    output.push_str("        ]\n");
                    output.push_str("    }\n");
                    output.push_str("}\n");
                    output.push_str("\n");
                }
            }
            _ => {}
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
                Some(union_name) => union_name,
            }
        }
        PropertyType::Array { item } => format!("Vec<{}>", generate_required_type(item, union_name)),
    }
}
