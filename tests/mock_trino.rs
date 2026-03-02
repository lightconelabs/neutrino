use neutrino::auth::Auth;
use std::thread;
use tiny_http::{Header, Response, Server};

fn json_header() -> Header {
    Header::from_bytes("Content-Type", "application/json").unwrap()
}

fn start_mock(responses: Vec<(u16, String)>) -> (u16, thread::JoinHandle<()>) {
    let server = Server::http("127.0.0.1:0").unwrap();
    let port = server.server_addr().to_ip().unwrap().port();

    let handle = thread::spawn(move || {
        for (status_code, body) in responses {
            server
                .recv()
                .unwrap()
                .respond(
                    Response::from_string(&body)
                        .with_status_code(tiny_http::StatusCode(status_code))
                        .with_header(json_header()),
                )
                .unwrap();
        }
    });

    (port, handle)
}

fn find_header<'a>(headers: &'a [tiny_http::Header], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|h| h.field.as_str().as_str().eq_ignore_ascii_case(name))
        .map(|h| h.value.as_str())
}

#[test]
fn submit_and_poll_single_page() {
    let (port, handle) = start_mock(vec![(
        200,
        serde_json::json!({
            "id": "query_1",
            "columns": [
                {"name": "value", "type": "integer"},
                {"name": "name", "type": "varchar"}
            ],
            "data": [[1, "hello"], [2, "world"]],
            "stats": {"state": "FINISHED"}
        })
        .to_string(),
    )]);

    let body: neutrino::client::QueryResponse = reqwest::blocking::Client::new()
        .post(format!("http://127.0.0.1:{port}/v1/statement"))
        .header("X-Trino-User", "test")
        .body("SELECT 1")
        .send()
        .unwrap()
        .json()
        .unwrap();

    assert_eq!(body.id.unwrap(), "query_1");
    assert_eq!(body.columns.as_ref().unwrap().len(), 2);
    assert_eq!(body.columns.as_ref().unwrap()[0].name, "value");
    assert_eq!(body.columns.as_ref().unwrap()[1].name, "name");
    assert_eq!(body.data.as_ref().unwrap().len(), 2);
    assert_eq!(body.data.as_ref().unwrap()[0][0], serde_json::json!(1));
    assert_eq!(body.data.as_ref().unwrap()[0][1], serde_json::json!("hello"));
    assert!(body.next_uri.is_none());
    assert_eq!(body.stats.unwrap().state, neutrino::client::QueryState::Finished);

    handle.join().unwrap();
}

#[test]
fn submit_and_poll_multiple_pages() {
    let server = Server::http("127.0.0.1:0").unwrap();
    let port = server.server_addr().to_ip().unwrap().port();

    let page1 = serde_json::json!({
        "id": "query_2",
        "columns": [{"name": "n", "type": "integer"}],
        "data": [[1], [2]],
        "stats": {"state": "RUNNING"},
        "nextUri": format!("http://127.0.0.1:{port}/v1/next")
    });
    let page2 = serde_json::json!({
        "id": "query_2",
        "data": [[3], [4]],
        "stats": {"state": "FINISHED"}
    });

    let handle = thread::spawn(move || {
        let req1 = server.recv().unwrap();
        assert!(req1.url().starts_with("/v1/statement"));
        req1.respond(Response::from_string(page1.to_string()).with_header(json_header()))
            .unwrap();

        let req2 = server.recv().unwrap();
        assert!(req2.url().starts_with("/v1/next"));
        req2.respond(Response::from_string(page2.to_string()).with_header(json_header()))
            .unwrap();
    });

    let client = reqwest::blocking::Client::new();

    let body1: neutrino::client::QueryResponse = client
        .post(format!("http://127.0.0.1:{port}/v1/statement"))
        .body("SELECT n FROM t")
        .send()
        .unwrap()
        .json()
        .unwrap();
    let data1 = body1.data.unwrap();
    assert_eq!(data1.len(), 2);

    let body2: neutrino::client::QueryResponse = client
        .get(body1.next_uri.unwrap())
        .send()
        .unwrap()
        .json()
        .unwrap();
    let data2 = body2.data.unwrap();
    assert_eq!(data2.len(), 2);
    assert!(body2.next_uri.is_none());

    let all_rows: Vec<_> = [data1, data2].concat();
    assert_eq!(all_rows.len(), 4);
    assert_eq!(all_rows[2][0], serde_json::json!(3));

    handle.join().unwrap();
}

#[test]
fn error_response_parsed() {
    let (port, handle) = start_mock(vec![(
        200,
        serde_json::json!({
            "id": "query_err",
            "stats": {"state": "FAILED"},
            "error": {
                "message": "Table does not exist",
                "errorName": "TABLE_NOT_FOUND",
                "errorType": "USER_ERROR"
            }
        })
        .to_string(),
    )]);

    let body: neutrino::client::QueryResponse = reqwest::blocking::Client::new()
        .post(format!("http://127.0.0.1:{port}/v1/statement"))
        .body("SELECT * FROM nonexistent")
        .send()
        .unwrap()
        .json()
        .unwrap();

    let error = body.error.unwrap();
    assert_eq!(error.message, "Table does not exist");
    assert_eq!(error.error_name.unwrap(), "TABLE_NOT_FOUND");
    assert_eq!(error.error_type.unwrap(), "USER_ERROR");

    handle.join().unwrap();
}

#[test]
fn basic_auth_header_sent() {
    let server = Server::http("127.0.0.1:0").unwrap();
    let port = server.server_addr().to_ip().unwrap().port();

    let handle = thread::spawn(move || {
        let req = server.recv().unwrap();

        assert!(
            find_header(req.headers(), "Authorization")
                .expect("Missing Authorization header")
                .starts_with("Basic "),
        );
        assert_eq!(
            find_header(req.headers(), "X-Trino-User").expect("Missing X-Trino-User"),
            "alice",
        );

        req.respond(
            Response::from_string(r#"{"id":"q1","stats":{"state":"FINISHED"}}"#)
                .with_header(json_header()),
        )
        .unwrap();
    });

    let mut headers = reqwest::header::HeaderMap::new();
    Auth::Basic {
        user: "alice".into(),
        password: "secret".into(),
    }
    .apply_headers(&mut headers)
    .unwrap();

    reqwest::blocking::Client::new()
        .post(format!("http://127.0.0.1:{port}/v1/statement"))
        .headers(headers)
        .body("SELECT 1")
        .send()
        .unwrap();

    handle.join().unwrap();
}

#[test]
fn catalog_and_schema_headers() {
    let server = Server::http("127.0.0.1:0").unwrap();
    let port = server.server_addr().to_ip().unwrap().port();

    let handle = thread::spawn(move || {
        let req = server.recv().unwrap();

        assert_eq!(
            find_header(req.headers(), "X-Trino-Catalog").expect("Missing X-Trino-Catalog"),
            "hive",
        );
        assert_eq!(
            find_header(req.headers(), "X-Trino-Schema").expect("Missing X-Trino-Schema"),
            "default",
        );

        req.respond(
            Response::from_string(r#"{"id":"q1","stats":{"state":"FINISHED"}}"#)
                .with_header(json_header()),
        )
        .unwrap();
    });

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("X-Trino-Catalog", "hive".parse().unwrap());
    headers.insert("X-Trino-Schema", "default".parse().unwrap());
    headers.insert("X-Trino-User", "test".parse().unwrap());

    reqwest::blocking::Client::new()
        .post(format!("http://127.0.0.1:{port}/v1/statement"))
        .headers(headers)
        .body("SELECT 1")
        .send()
        .unwrap();

    handle.join().unwrap();
}
