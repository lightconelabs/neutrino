use neutrino::auth::{Auth, AuthFlow};
use reqwest::header::{AUTHORIZATION, HeaderMap};

#[test]
fn none_auth_sets_trino_user() {
    let auth = Auth::None {
        user: "alice".into(),
    };
    let mut headers = HeaderMap::new();
    auth.apply_headers(&mut headers).unwrap();

    assert_eq!(headers.get("X-Trino-User").unwrap(), "alice");
    assert!(headers.get(AUTHORIZATION).is_none());
}

#[test]
fn basic_auth_sets_header() {
    let auth = Auth::Basic {
        user: "alice".into(),
        password: "secret".into(),
    };
    let mut headers = HeaderMap::new();
    auth.apply_headers(&mut headers).unwrap();

    assert_eq!(headers.get("X-Trino-User").unwrap(), "alice");

    let auth_value = headers.get(AUTHORIZATION).unwrap().to_str().unwrap();
    assert!(auth_value.starts_with("Basic "));

    use base64::Engine;
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(auth_value.strip_prefix("Basic ").unwrap())
        .unwrap();
    assert_eq!(String::from_utf8(decoded).unwrap(), "alice:secret");
}

#[test]
fn jwt_auth_sets_bearer() {
    let auth = Auth::Jwt {
        token: "my.jwt.token".into(),
        user: Some("bob".into()),
    };
    let mut headers = HeaderMap::new();
    auth.apply_headers(&mut headers).unwrap();

    assert_eq!(headers.get("X-Trino-User").unwrap(), "bob");
    assert_eq!(
        headers.get(AUTHORIZATION).unwrap().to_str().unwrap(),
        "Bearer my.jwt.token"
    );
}

#[test]
fn jwt_auth_without_user() {
    let auth = Auth::Jwt {
        token: "my.jwt.token".into(),
        user: None,
    };
    let mut headers = HeaderMap::new();
    auth.apply_headers(&mut headers).unwrap();

    assert!(headers.get("X-Trino-User").is_none());
    assert_eq!(
        headers.get(AUTHORIZATION).unwrap().to_str().unwrap(),
        "Bearer my.jwt.token"
    );
}

#[test]
fn non_oauth2_resolve_returns_same_auth() {
    let flow = AuthFlow::None {
        user: "alice".into(),
    };
    let client = reqwest::blocking::Client::new();
    let result = flow.resolve(&client, "https://localhost:443").unwrap();
    assert!(matches!(result, Auth::None { ref user } if user == "alice"));
}

#[test]
fn invalid_user_header_is_rejected() {
    let auth = Auth::None {
        user: "alice\nbob".into(),
    };
    let mut headers = HeaderMap::new();
    let err = auth.apply_headers(&mut headers).unwrap_err();
    assert!(err.to_string().contains("Invalid X-Trino-User header value"));
}
