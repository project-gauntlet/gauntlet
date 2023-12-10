use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use component_model::{Children, create_component_model, PropertyType};

fn main() -> anyhow::Result<()> {
    let out_dir = env::var("OUT_DIR")?;
    let dest_path = Path::new(&out_dir).join("components.rs");

    let mut output = String::new();

    let components = create_component_model();

    output.push_str("#[derive(Debug)]\n");
    output.push_str("enum ComponentWidget {\n");
    output.push_str("    TextPart(String),\n");

    for component in &components {

        let name = component.name();

        output.push_str(&format!("    {}", name));

        let props = component.props();
        let has_children = !matches!(component.children(), Children::None);

        if !props.is_empty() || has_children {
            output.push_str(" {\n");

            if has_children {
                output.push_str(&format!("        children: {},\n", generate_children_type(&component.children())));
            }

            for prop in props {
                match prop.property_type() {
                    PropertyType::String | PropertyType::Number | PropertyType::Boolean if prop.optional() => {
                        output.push_str(&format!("        {}: {},\n", prop.name(), generate_optional_type(prop.property_type())));
                    }
                    _ => {
                        output.push_str(&format!("        {}: {},\n", prop.name(), generate_type(prop.property_type())));
                    }
                }
            }
            output.push_str("    },\n");
        } else {
            output.push_str(",\n");
        }
    }

    output.push_str("}\n");
    output.push_str("\n");

    output.push_str("fn create_component_widget(component_internal_name: &str, properties: HashMap<String, NativeUiPropertyValue>) -> anyhow::Result<ComponentWidget> {\n");
    output.push_str("   let widget = match component_internal_name {\n");

    for component in &components {
        let name = component.name();
        let internal_name = component.internal_name();
        output.push_str(&format!("        \"gauntlet:{}\" => ComponentWidget::{}", internal_name, name));

        let props = component.props();
        let has_children = !matches!(component.children(), Children::None);

        if !props.is_empty() || has_children {
            output.push_str(&" {\n");

            if has_children {
                output.push_str("            children: vec![],\n");
            }

            for prop in props {
                match prop.property_type() {
                    PropertyType::String => {
                        if prop.optional() {
                            output.push_str(&format!("            {}: parse_optional_string(&properties, \"{}\")?,\n", prop.name(), prop.name()));
                        } else {
                            output.push_str(&format!("            {}: parse_string(&properties, \"{}\")?,\n", prop.name(), prop.name()));
                        }
                    },
                    PropertyType::Number => {
                        if prop.optional() {
                            output.push_str(&format!("            {}: parse_optional_number(&properties, \"{}\")?,\n", prop.name(), prop.name()));
                        } else {
                            output.push_str(&format!("            {}: parse_number(&properties, \"{}\")?,\n", prop.name(), prop.name()));
                        }
                    },
                    PropertyType::Boolean => {
                        if prop.optional() {
                            output.push_str(&format!("            {}: parse_optional_boolean(&properties, \"{}\")?,\n", prop.name(), prop.name()));
                        } else {
                            output.push_str(&format!("            {}: parse_boolean(&properties, \"{}\")?,\n", prop.name(), prop.name()));
                        }
                    },
                    PropertyType::Array { .. } => {
                        output.push_str(&format!("            {}: vec![],\n", prop.name()));
                    },
                    PropertyType::Function => {
                        output.push_str(&format!("            {}: (),\n", prop.name()));
                    }
                };
            }
            output.push_str("        },\n");
        } else {
            output.push_str(",\n");
        }
    }

    output.push_str("        _ => panic!(\"component_internal_name {} not supported\", component_internal_name)\n");
    output.push_str("    };\n");
    output.push_str("    Ok(widget)\n");
    output.push_str("}\n");
    output.push_str("\n");


    output.push_str("fn append_component_widget_child(parent: &ComponentWidgetWrapper, child: ComponentWidgetWrapper) {\n");
    output.push_str("    let mut parent = parent.get_mut();\n");
    output.push_str("    match *parent {\n");
    output.push_str("        ComponentWidget::TextPart { .. } => panic!(\"text part cannot be a parent\"),\n");

    for component in &components {
        let name = component.name();
        let internal_name = component.internal_name();
        let has_children = !matches!(component.children(), Children::None);

        if has_children {
            output.push_str(&format!("        ComponentWidget::{} {{ ref mut children, .. }} => {{\n", name));
            output.push_str("            children.push(child)\n");
            output.push_str("        }\n");
        } else {
            output.push_str(&format!("        ComponentWidget::{} {{ .. }} => {{\n", name));
            output.push_str(&format!("            panic!(\"{} cannot be a parent\")\n", internal_name));
            output.push_str("        }\n");
        }
    }

    output.push_str("    }\n");
    output.push_str("}\n");
    output.push_str("\n");


    output.push_str("fn can_component_widget_have_children(widget: &ComponentWidgetWrapper) -> bool {\n");
    output.push_str("    let widget = widget.get();\n");
    output.push_str("    match *widget {\n");
    output.push_str("        ComponentWidget::TextPart { .. } => false,\n");

    for component in &components {
        let name = component.name();
        let has_children = !matches!(component.children(), Children::None);
        let has_children = if has_children { "true" } else { "false" };

        output.push_str(&format!("        ComponentWidget::{} {{ .. }} => {},\n", name, has_children));
    }

    output.push_str("    }\n");
    output.push_str("}\n");
    output.push_str("\n");


    output.push_str("fn get_component_widget_children(widget: &ComponentWidgetWrapper) -> Vec<ComponentWidgetWrapper> {\n");
    output.push_str("    let widget = widget.get();\n");
    output.push_str("    let children = match *widget {\n");
    output.push_str("        ComponentWidget::TextPart { .. } => panic!(\"text part cannot have children\"),\n");

    for component in &components {
        let name = component.name();
        let internal_name = component.internal_name();
        let has_children = !matches!(component.children(), Children::None);

        if has_children {
            output.push_str(&format!("        ComponentWidget::{} {{ ref children, .. }} => {{\n", name));
            output.push_str("            children\n");
            output.push_str("        }\n");
        } else {
            output.push_str(&format!("        ComponentWidget::{} {{ .. }} => {{\n", name));
            output.push_str(&format!("            panic!(\"{} cannot have children\")\n", internal_name));
            output.push_str("        }\n");
        }
    }

    output.push_str("    };\n");
    output.push_str("    children.iter().cloned().collect()\n");
    output.push_str("}\n");
    output.push_str("\n");


    output.push_str("fn set_component_widget_children(widget: &ComponentWidgetWrapper, new_children: Vec<ComponentWidgetWrapper>) {\n");
    output.push_str("    let mut widget = widget.get_mut();\n");
    output.push_str("    match *widget {\n");
    output.push_str("        ComponentWidget::TextPart { .. } => panic!(\"text part cannot have children\"),\n");

    for component in &components {
        let name = component.name();
        let internal_name = component.internal_name();
        let has_children = !matches!(component.children(), Children::None);

        if has_children {
            output.push_str(&format!("        ComponentWidget::{} {{ ref mut children, .. }} => {{\n", name));
            output.push_str("            *children = new_children\n");
            output.push_str("        }\n");
        } else {
            output.push_str(&format!("        ComponentWidget::{} {{ .. }} => {{\n", name));
            output.push_str(&format!("            panic!(\"{} cannot have children\")\n", internal_name));
            output.push_str("        }\n");
        }
    }

    output.push_str("    }\n");
    output.push_str("}\n");
    output.push_str("\n");


    output.push_str("fn get_component_widget_type(widget: &ComponentWidgetWrapper) -> String {\n");
    output.push_str("    let widget = widget.get();\n");
    output.push_str("    match *widget {\n");
    output.push_str("        ComponentWidget::TextPart { .. } => panic!(\"cannot get type of text part\"),\n");

    for component in &components {
        let name = component.name();
        let internal_name = component.internal_name();

        output.push_str(&format!("        ComponentWidget::{} {{ .. }} => \"gauntlet:{}\",\n", name, internal_name));
    }

    output.push_str("    }.to_owned()\n");
    output.push_str("}\n");
    output.push_str("\n");


    output.push_str("fn parse_optional_string(properties: &HashMap<String, NativeUiPropertyValue>, name: &str) -> anyhow::Result<Option<String>> {\n");
    output.push_str("    match properties.get(name) {\n");
    output.push_str("        None => Ok(None),\n");
    output.push_str("        Some(value) => Ok(Some(value.as_string().ok_or(anyhow::anyhow!(\"{} has to be a string\", name))?.to_owned())),\n");
    output.push_str("    }\n");
    output.push_str("}\n");
    output.push_str("\n");

    output.push_str("fn parse_string(properties: &HashMap<String, NativeUiPropertyValue>, name: &str) -> anyhow::Result<String> {\n");
    output.push_str("    Ok(properties.get(name).ok_or(anyhow::anyhow!(\"{} is required\", name))?.as_string().ok_or(anyhow::anyhow!(\"{} has to be a string\", name))?.to_owned())\n");
    output.push_str("}\n");
    output.push_str("\n");


    output.push_str("fn parse_optional_number(properties: &HashMap<String, NativeUiPropertyValue>, name: &str) -> anyhow::Result<Option<f64>> {\n");
    output.push_str("    match properties.get(name) {\n");
    output.push_str("        None => Ok(None),\n");
    output.push_str("        Some(value) => Ok(Some(value.as_number().ok_or(anyhow::anyhow!(\"{} has to be a number\", name))?.to_owned())),\n");
    output.push_str("    }\n");
    output.push_str("}\n");
    output.push_str("\n");

    output.push_str("fn parse_number(properties: &HashMap<String, NativeUiPropertyValue>, name: &str) -> anyhow::Result<f64> {\n");
    output.push_str("    Ok(properties.get(name).ok_or(anyhow::anyhow!(\"{} is required\", name))?.as_number().ok_or(anyhow::anyhow!(\"{} has to be a number\", name))?.to_owned())\n");
    output.push_str("}\n");
    output.push_str("\n");


    output.push_str("fn parse_optional_boolean(properties: &HashMap<String, NativeUiPropertyValue>, name: &str) -> anyhow::Result<Option<bool>> {\n");
    output.push_str("    match properties.get(name) {\n");
    output.push_str("        None => Ok(None),\n");
    output.push_str("        Some(value) => Ok(Some(value.as_bool().ok_or(anyhow::anyhow!(\"{} has to be a boolean\", name))?.to_owned())),\n");
    output.push_str("    }\n");
    output.push_str("}\n");
    output.push_str("\n");

    output.push_str("fn parse_boolean(properties: &HashMap<String, NativeUiPropertyValue>, name: &str) -> anyhow::Result<bool> {\n");
    output.push_str("    Ok(properties.get(name).ok_or(anyhow::anyhow!(\"{} is required\", name))?.as_bool().ok_or(anyhow::anyhow!(\"{} has to be a boolean\", name))?.to_owned())\n");
    output.push_str("}\n");
    output.push_str("\n");

    generate_file(&dest_path, &output)?;

    Ok(())
}

fn generate_file<P: AsRef<Path>>(path: P, text: &str) -> std::io::Result<()> {
    let mut f = File::create(path)?;
    f.write_all(text.as_bytes())
}

fn generate_optional_type(property_type: &PropertyType) -> String {
    format!("Option<{}>", generate_type(property_type))
}

fn generate_children_type(children: &Children) -> String {
    match children {
        Children::Members { .. } => "Vec<ComponentWidgetWrapper>".to_owned(),
        Children::String => "Vec<ComponentWidgetWrapper>".to_owned(),
        Children::None => panic!("cannot create type for Children::None")
    }
}

fn generate_type(property_type: &PropertyType) -> String {
    match property_type {
        PropertyType::String => "String".to_owned(),
        PropertyType::Number => "f64".to_owned(),
        PropertyType::Boolean => "bool".to_owned(),
        PropertyType::Array { nested } => format!("Vec<{}>", generate_type(nested)),
        PropertyType::Function => "()".to_string()
    }
}