use crate::client::QueryResult;
use std::io::{self, Write};

use super::format_value;

pub fn write_csv(result: &QueryResult, writer: impl Write) -> io::Result<()> {
    if result.columns().is_empty() {
        return Ok(());
    }

    let mut wtr = csv::Writer::from_writer(writer);
    wtr.write_record(result.columns().iter().map(|c| &c.name))
        .map_err(io::Error::other)?;
    for row in result.rows() {
        wtr.write_record(row.iter().map(format_value))
            .map_err(io::Error::other)?;
    }
    wtr.flush()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::Column;

    fn write_csv_to_string(result: &QueryResult) -> String {
        let mut buf = Vec::new();
        write_csv(result, &mut buf).unwrap();
        String::from_utf8(buf).unwrap()
    }

    #[test]
    fn csv_basic_output() {
        let result = QueryResult::new(
            vec![
                Column {
                    name: "id".into(),
                    col_type: "integer".into(),
                },
                Column {
                    name: "name".into(),
                    col_type: "varchar".into(),
                },
            ],
            vec![
                vec![serde_json::json!(1), serde_json::json!("alice")],
                vec![serde_json::json!(2), serde_json::json!("bob")],
            ],
        )
        .unwrap();

        let output = write_csv_to_string(&result);
        assert_eq!(output, "id,name\n1,alice\n2,bob\n");
    }

    #[test]
    fn csv_null_values() {
        let result = QueryResult::new(
            vec![Column {
                name: "val".into(),
                col_type: "varchar".into(),
            }],
            vec![vec![serde_json::Value::Null]],
        )
        .unwrap();

        let output = write_csv_to_string(&result);
        assert_eq!(output, "val\nNULL\n");
    }

    #[test]
    fn csv_quoting_for_commas() {
        let result = QueryResult::new(
            vec![Column {
                name: "val".into(),
                col_type: "varchar".into(),
            }],
            vec![vec![serde_json::json!("hello, world")]],
        )
        .unwrap();

        let output = write_csv_to_string(&result);
        assert_eq!(output, "val\n\"hello, world\"\n");
    }

    #[test]
    fn csv_empty_columns_produces_no_output() {
        let result = QueryResult::new(vec![], vec![]).unwrap();
        let output = write_csv_to_string(&result);
        assert_eq!(output, "");
    }

    #[test]
    fn csv_header_only_for_empty_rows() {
        let result = QueryResult::new(
            vec![Column {
                name: "id".into(),
                col_type: "integer".into(),
            }],
            vec![],
        )
        .unwrap();

        let output = write_csv_to_string(&result);
        assert_eq!(output, "id\n");
    }
}
