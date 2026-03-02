use crate::client::QueryResult;
use crate::OutputFormat;
use tabled::{Table, builder::Builder, settings::Style};

pub fn print_result(result: &QueryResult, format: OutputFormat) {
    match format {
        OutputFormat::Table => print_table(result),
        OutputFormat::Json => print_json(result),
    }
}

fn print_table(result: &QueryResult) {
    if result.columns().is_empty() {
        println!("(no results)");
        return;
    }

    let mut builder = Builder::default();
    builder.push_record(result.columns().iter().map(|c| format!("{} ({})", c.name, c.col_type)));
    for row in result.rows() {
        builder.push_record(row.iter().map(format_value));
    }

    let mut table = Table::from(builder);
    table.with(Style::rounded());
    println!("{table}");
    println!("({} rows)", result.rows().len());
}

fn print_json(result: &QueryResult) {
    let rows: Vec<serde_json::Value> = result
        .rows()
        .iter()
        .map(|row| {
            serde_json::Value::Object(
                result
                    .columns()
                    .iter()
                    .zip(row.iter())
                    .map(|(col, val)| (col.name.clone(), val.clone()))
                    .collect(),
            )
        })
        .collect();

    println!("{}", serde_json::to_string_pretty(&rows).unwrap());
}

fn format_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "NULL".to_string(),
        serde_json::Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}
