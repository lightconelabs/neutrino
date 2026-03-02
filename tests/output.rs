use neutrino::OutputFormat;
use neutrino::client::{Column, QueryResult};
use neutrino::output::print_result;

#[test]
fn empty_result_prints_no_results() {
    print_result(
        &QueryResult::new(vec![], vec![]).unwrap(),
        OutputFormat::Table,
    );
}

#[test]
fn single_row_table() {
    print_result(
        &QueryResult::new(
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
            vec![vec![serde_json::json!(1), serde_json::json!("alice")]],
        )
        .unwrap(),
        OutputFormat::Table,
    );
}

#[test]
fn null_values_displayed() {
    print_result(
        &QueryResult::new(
            vec![Column {
                name: "value".into(),
                col_type: "varchar".into(),
            }],
            vec![
                vec![serde_json::json!("hello")],
                vec![serde_json::Value::Null],
                vec![serde_json::json!("world")],
            ],
        )
        .unwrap(),
        OutputFormat::Table,
    );
}

#[test]
fn mixed_types() {
    print_result(
        &QueryResult::new(
            vec![
                Column {
                    name: "int_col".into(),
                    col_type: "integer".into(),
                },
                Column {
                    name: "str_col".into(),
                    col_type: "varchar".into(),
                },
                Column {
                    name: "bool_col".into(),
                    col_type: "boolean".into(),
                },
                Column {
                    name: "null_col".into(),
                    col_type: "varchar".into(),
                },
            ],
            vec![vec![
                serde_json::json!(42),
                serde_json::json!("hello"),
                serde_json::json!(true),
                serde_json::Value::Null,
            ]],
        )
        .unwrap(),
        OutputFormat::Table,
    );
}

#[test]
fn many_rows() {
    print_result(
        &QueryResult::new(
            vec![
                Column {
                    name: "id".into(),
                    col_type: "integer".into(),
                },
                Column {
                    name: "label".into(),
                    col_type: "varchar".into(),
                },
            ],
            (0..100)
                .map(|i| vec![serde_json::json!(i), serde_json::json!(format!("row_{i}"))])
                .collect(),
        )
        .unwrap(),
        OutputFormat::Table,
    );
}

#[test]
fn json_output() {
    print_result(
        &QueryResult::new(
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
                vec![serde_json::json!(2), serde_json::Value::Null],
            ],
        )
        .unwrap(),
        OutputFormat::Json,
    );
}

#[test]
fn mismatched_columns_and_rows_rejected() {
    let result = QueryResult::new(
        vec![Column {
            name: "a".into(),
            col_type: "integer".into(),
        }],
        vec![vec![serde_json::json!(1), serde_json::json!(2)]],
    );
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Row 0 has 2 values but there are 1 columns")
    );
}
