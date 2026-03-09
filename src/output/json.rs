use crate::client::QueryResult;
use std::io::{self, Write};

pub fn write_json(result: &QueryResult, mut writer: impl Write) -> io::Result<()> {
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

    writeln!(writer, "{}", serde_json::to_string_pretty(&rows).unwrap())
}
