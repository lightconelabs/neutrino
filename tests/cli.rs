use clap::Parser;
use neutrino::{Cli, SqlInput};

fn parse(args: &[&str]) -> Result<Cli, clap::Error> {
    Cli::try_parse_from(std::iter::once("neutrino").chain(args.iter().copied()))
}

#[test]
fn no_args_fails_on_missing_host() {
    assert!(parse(&[]).is_err());
}

#[test]
fn host_is_required() {
    let err = parse(&["SELECT 1"]).unwrap_err().to_string();
    assert!(err.contains("--host"), "expected --host error, got: {err}");
}

#[test]
fn minimal_valid_args() {
    let cli = parse(&["--host", "trino.example.com", "--user", "alice", "SELECT 1"]).unwrap();
    assert_eq!(cli.host, "trino.example.com");
    assert_eq!(cli.user.as_deref(), Some("alice"));
    assert_eq!(cli.query.as_deref(), Some("SELECT 1"));
}

#[test]
fn default_port_is_443() {
    let cli = parse(&["--host", "localhost", "SELECT 1"]).unwrap();
    assert_eq!(cli.port, 443);
}

#[test]
fn custom_port() {
    let cli = parse(&["--host", "localhost", "--port", "8080", "SELECT 1"]).unwrap();
    assert_eq!(cli.port, 8080);
}

#[test]
fn file_flag_parsed() {
    let cli = parse(&["--host", "localhost", "-f", "query.sql"]).unwrap();
    assert_eq!(cli.file.as_deref(), Some("query.sql"));
}

#[test]
fn all_flags_parsed() {
    let cli = parse(&[
        "--host", "trino.example.com",
        "--port", "8443",
        "--user", "alice",
        "--catalog", "hive",
        "--schema", "default",
        "--password", "secret",
        "--insecure",
        "SHOW TABLES",
    ])
    .unwrap();

    assert_eq!(cli.host, "trino.example.com");
    assert_eq!(cli.port, 8443);
    assert_eq!(cli.user.as_deref(), Some("alice"));
    assert_eq!(cli.catalog.as_deref(), Some("hive"));
    assert_eq!(cli.schema.as_deref(), Some("default"));
    assert_eq!(cli.password.as_deref(), Some("secret"));
    assert!(cli.insecure);
    assert_eq!(cli.query.as_deref(), Some("SHOW TABLES"));
}

#[test]
fn resolve_sql_from_query() {
    let cli = parse(&["--host", "localhost", "SELECT 1"]).unwrap();
    assert_eq!(cli.resolve_sql().unwrap(), "SELECT 1");
}

#[test]
fn resolve_sql_input_from_query() {
    let cli = parse(&["--host", "localhost", "SELECT 1"]).unwrap();
    let input = cli.resolve_sql_input().unwrap();
    assert!(matches!(input, SqlInput::Inline(ref query) if query == "SELECT 1"));
}

#[test]
fn resolve_sql_from_file() {
    let dir = std::env::temp_dir().join("neutrino_test_resolve_sql");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("test.sql");
    std::fs::write(&path, "SELECT 42").unwrap();

    let cli = parse(&["--host", "localhost", "-f", path.to_str().unwrap()]).unwrap();
    assert_eq!(cli.resolve_sql().unwrap(), "SELECT 42");

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn resolve_sql_input_from_file() {
    let dir = std::env::temp_dir().join("neutrino_test_resolve_sql_input");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("test.sql");
    std::fs::write(&path, "SELECT 42").unwrap();

    let cli = parse(&["--host", "localhost", "-f", path.to_str().unwrap()]).unwrap();
    let input = cli.resolve_sql_input().unwrap();
    assert!(matches!(input, SqlInput::File(ref p) if p == &path));

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn resolve_sql_missing_file_errors() {
    let cli = parse(&["--host", "localhost", "-f", "/nonexistent.sql"]).unwrap();
    let err = cli.resolve_sql().unwrap_err().to_string();
    assert!(err.contains("Failed to read SQL file"), "got: {err}");
}

#[test]
fn resolve_sql_no_query_errors() {
    let cli = parse(&["--host", "localhost"]).unwrap();
    let err = cli.resolve_sql().unwrap_err().to_string();
    assert!(err.contains("No query provided"), "got: {err}");
}

#[test]
fn resolve_auth_basic() {
    let cli = parse(&["--host", "localhost", "--user", "alice", "--password", "secret", "SELECT 1"]).unwrap();
    let (auth, ..) = cli.resolve_auth().unwrap();
    assert!(matches!(auth, neutrino::auth::AuthFlow::Basic { ref user, ref password }
        if user == "alice" && password == "secret"));
}

#[test]
fn resolve_auth_password_without_user_errors() {
    let cli = parse(&["--host", "localhost", "--password", "secret", "SELECT 1"]).unwrap();
    let err = cli.resolve_auth().unwrap_err().to_string();
    assert!(err.contains("--user is required"), "got: {err}");
}

#[test]
fn resolve_auth_jwt() {
    let cli = parse(&["--host", "localhost", "--jwt-token", "abc.def.ghi", "SELECT 1"]).unwrap();
    let (auth, ..) = cli.resolve_auth().unwrap();
    assert!(matches!(auth, neutrino::auth::AuthFlow::Jwt { ref token, .. } if token == "abc.def.ghi"));
}

#[test]
fn resolve_auth_user_only() {
    let cli = parse(&["--host", "localhost", "--user", "alice", "SELECT 1"]).unwrap();
    let (auth, ..) = cli.resolve_auth().unwrap();
    assert!(matches!(auth, neutrino::auth::AuthFlow::None { ref user } if user == "alice"));
}

#[test]
fn resolve_auth_defaults_to_oauth2() {
    let cli = parse(&["--host", "localhost", "SELECT 1"]).unwrap();
    let (auth, ..) = cli.resolve_auth().unwrap();
    assert!(matches!(auth, neutrino::auth::AuthFlow::OAuth2));
}
