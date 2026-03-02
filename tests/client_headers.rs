use neutrino::auth::AuthFlow;
use neutrino::client::TrinoClient;

#[test]
fn invalid_catalog_header_is_rejected() {
    let client = TrinoClient::new(
        "127.0.0.1",
        1,
        AuthFlow::None {
            user: "alice".into(),
        },
        Some("bad\nvalue".into()),
        None,
        true,
    )
    .unwrap();

    match client.execute("SELECT 1", None, false) {
        Ok(_) => panic!("expected invalid header error"),
        Err(err) => {
            assert!(
                format!("{err:#}").contains("Invalid X-Trino-Catalog header value"),
                "unexpected error: {err:#}"
            );
        }
    }
}
