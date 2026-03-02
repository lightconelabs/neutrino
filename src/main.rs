use anyhow::Result;
use clap::Parser;
use neutrino::Cli;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let sql = cli.resolve_sql_input()?.load()?;
    let output_format = cli.output;
    let limit = cli.limit;
    let neutrino::ResolvedAuth {
        auth,
        host,
        port,
        catalog,
        schema,
        insecure,
    } = cli.resolve_auth()?;

    let quiet = matches!(output_format, neutrino::OutputFormat::Json);
    let result = neutrino::client::TrinoClient::new(&host, port, auth, catalog, schema, insecure)?
        .execute(&sql, limit, quiet)?;

    neutrino::output::print_result(&result, output_format);

    Ok(())
}
