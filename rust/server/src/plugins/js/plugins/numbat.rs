use numbat::InterpreterResult;
use numbat::markup::Formatter;
use numbat::module_importer::BuiltinModuleImporter;
use numbat::pretty_print::PrettyPrint;
use numbat::value::Value;

use anyhow::anyhow;
use deno_core::op;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct NumbatResult {
    left: String,
    right: String,
}

#[op]
fn run_numbat(input: String) -> anyhow::Result<NumbatResult> {
    // TODO add check for plugin id

    let mut context = numbat::Context::new(BuiltinModuleImporter::default());
    let _ = context.interpret("use prelude", numbat::resolver::CodeSource::Internal);

    let (statements, result) = context.interpret(&input, numbat::resolver::CodeSource::Text)?;

    let formatter = numbat::markup::PlainTextFormatter;

    let expression = statements
        .iter()
        .map(|s| formatter.format(&s.pretty_print(), false))
        .collect::<Vec<_>>()
        .join(" ")
        .replace('➞', "to");

    let value = match result {
        InterpreterResult::Value(value) => value,
        InterpreterResult::Continue => Err(anyhow!("numbat returned Continue"))?
    };

    let value = match value {
        Value::Quantity(value) => value.to_string(),
        Value::Boolean(value) => value.to_string(),
        Value::String(value) => value,
    };

    Ok(NumbatResult {
        left: expression,
        right: value
    })
}