use crate::client::QueryResult;
use std::io::{self, Write};
use tabled::{Table, builder::Builder, settings::Style};

use super::format_value;

pub fn write_table(result: &QueryResult, mut writer: impl Write) -> io::Result<()> {
    if result.columns().is_empty() {
        writeln!(writer, "(no results)")?;
        return Ok(());
    }

    let mut builder = Builder::default();
    builder.push_record(
        result
            .columns()
            .iter()
            .map(|c| format!("{} ({})", c.name, c.col_type)),
    );
    for row in result.rows() {
        builder.push_record(row.iter().map(format_value));
    }

    let mut table = Table::from(builder);
    table.with(Style::rounded());
    writeln!(writer, "{table}")?;
    writeln!(writer, "({} rows)", result.rows().len())
}
