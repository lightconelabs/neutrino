use anyhow::Result;
use clap::Parser;
use neutrino::Cli;
use std::fs::File;
use std::io::{self, BufWriter};

fn main() -> Result<()> {
    // Reset SIGPIPE to default behavior so the process terminates silently when
    // output is piped into a tool that closes early (e.g. `neutrino ... | head`).
    // Rust ignores SIGPIPE by default, which causes write calls to return errors
    // instead of terminating the process — the standard behavior for Unix CLI tools.
    #[cfg(unix)]
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }

    let cli = Cli::parse();
    let sql = cli.resolve_sql_input()?.load()?;
    let output_format = cli.format;
    let output_file = cli.output.clone();
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

    match output_file {
        Some(path) => {
            let file = File::create(&path)?;
            let writer = BufWriter::new(file);
            neutrino::output::write_result(&result, output_format, writer)?;
        }
        None => {
            let stdout = io::stdout().lock();
            let writer = BufWriter::new(stdout);
            neutrino::output::write_result(&result, output_format, writer)?;
        }
    }

    Ok(())
}
