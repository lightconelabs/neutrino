mod csv;
mod json;
mod table;

use crate::OutputFormat;
use crate::client::QueryResult;
use std::io::{self, Write};

pub fn print_result(result: &QueryResult, format: OutputFormat) {
    let stdout = io::stdout().lock();
    let writer = io::BufWriter::new(stdout);
    write_result(result, format, writer).unwrap();
}

pub fn write_result(
    result: &QueryResult,
    format: OutputFormat,
    writer: impl Write,
) -> io::Result<()> {
    match format {
        OutputFormat::Table => table::write_table(result, writer),
        OutputFormat::Json => json::write_json(result, writer),
        OutputFormat::Csv => csv::write_csv(result, writer),
    }
}

fn format_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "NULL".to_string(),
        serde_json::Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}
