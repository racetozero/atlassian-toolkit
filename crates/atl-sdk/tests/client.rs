//! HTTP-boundary tests for the SDK against a local mock server.

use atl_sdk::{
    Auth, CallArgs, Client, Config, Connection, Deployment, Error, Family, HttpMethod, Pager,
    verify,
};
use serde_json::{Map, Value, json};
use std::time::Duration;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

fn connection(server: &MockServer, deployment: Deployment, auth: Auth) -> Connection {
    Connection::new(deployment, server.uri(), auth)
}

fn basic() -> Auth {
    Auth::Basic {
        username: "ada@example.com".into(),
        secret: "token".into(),
    }
}

#[tokio::test]
async fn call_sends_credentials_encoded_path_and_repeated_query_values() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/rest/api/3/issue/A%2FB%20C"))
        // base64("ada@example.com:token")
        .and(header(
            "authorization",
            "Basic YWRhQGV4YW1wbGUuY29tOnRva2Vu",
        ))
        .respond_with(|request: &Request| {
            let fields: Vec<String> = request
                .url
                .query_pairs()
                .filter(|(k, _)| k == "fields")
                .map(|(_, v)| v.into_owned())
                .collect();
            ResponseTemplate::new(200).set_body_json(json!({ "fields": fields }))
        })
        .expect(1)
        .mount(&server)
        .await;
    let client = Client::with_connections([(
        Family::Jira,
        connection(&server, Deployment::Cloud, basic()),
    )]);

    let args = CallArgs::default()
        .param("issue-id-or-key", "A/B C")
        .param("fields", json!(["summary", "status"]));
    let response = client
        .call("jira", "getIssue", &args)
        .await
        .expect("call succeeds");

    assert_eq!(
        response.json(),
        Some(json!({ "fields": ["summary", "status"] }))
    );
}

#[tokio::test]
async fn data_center_uses_its_own_catalog_and_bearer_tokens() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/rest/api/2/myself"))
        .and(header("authorization", "Bearer pat"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "displayName": "Ada" })))
        .expect(2)
        .mount(&server)
        .await;
    let conn = connection(
        &server,
        Deployment::DataCenter,
        Auth::Bearer {
            token: "pat".into(),
        },
    );
    let client = Client::with_connections([(Family::Jira, conn.clone())]);

    let (service, catalog, op) = client
        .resolve("jira", "myself.get-user")
        .expect("DC myself exists");
    assert_eq!(op.path, "/rest/api/2/myself");
    let request = client
        .prepare(service, catalog, op, &CallArgs::default())
        .expect("prepare");
    client.execute(&request).await.expect("execute");
    assert_eq!(verify(Family::Jira, &conn).await.expect("verify"), "Ada");
}

#[tokio::test]
async fn http_errors_carry_the_product_error_message() {
    let server = MockServer::start().await;
    Mock::given(path("/rest/api/3/issue/NOPE-1"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "errorMessages": ["Issue does not exist or you do not have permission to see it."],
            "errors": {}
        })))
        .mount(&server)
        .await;
    Mock::given(path("/rest/api/3/myself"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
        .mount(&server)
        .await;
    let client = Client::with_connections([(
        Family::Jira,
        connection(&server, Deployment::Cloud, basic()),
    )]);

    let err = client
        .call(
            "jira",
            "getIssue",
            &CallArgs::default().param("issueIdOrKey", "NOPE-1"),
        )
        .await
        .expect_err("404");
    match &err {
        Error::Http(http) if http.status == 404 => assert!(
            http.message.starts_with("Issue does not exist"),
            "{}",
            http.message
        ),
        other => panic!("unexpected {other:?}"),
    }
    assert!(!err.is_auth());

    let err = client
        .call("jira", "getCurrentUser", &CallArgs::default())
        .await
        .expect_err("401");
    assert!(err.is_auth(), "{err}");
}

#[tokio::test]
async fn missing_credentials_fail_before_any_request_but_still_prepare() {
    let client = Client::with_connections([]);
    let (service, catalog, op) = client.resolve("jira", "getIssue").expect("resolve");
    let request = client
        .prepare(
            service,
            catalog,
            op,
            &CallArgs::default().param("issueIdOrKey", "A-1"),
        )
        .expect("dry runs need no credentials");
    assert_eq!(
        request.url.as_str(),
        "https://your-domain.atlassian.net/rest/api/3/issue/A-1"
    );

    let err = client.execute(&request).await.expect_err("no credentials");
    assert!(err.is_auth());
    assert!(err.to_string().contains("ATL_JIRA_TOKEN"), "{err}");
}

#[test]
fn invalid_inputs_name_the_problem_and_the_fix() {
    let client = Client::with_connections([]);
    let (service, catalog, op) = client.resolve("jira", "getIssue").expect("resolve");
    let cases = [
        (
            CallArgs::default(),
            "needs parameter `issueIdOrKey` (CLI flag --issue-id-or-key)",
        ),
        (
            CallArgs::default()
                .param("issueIdOrKey", "A-1")
                .param("bogus", 1),
            "has no parameter `bogus`; valid parameters: issueIdOrKey",
        ),
        (
            CallArgs::default()
                .param("issueIdOrKey", "A-1")
                .body(json!({})),
            "takes no request body",
        ),
    ];
    for (args, expected) in cases {
        let err = client
            .prepare(service, catalog, op, &args)
            .expect_err("invalid");
        assert!(matches!(err, Error::Invalid(_)), "{err:?}");
        assert!(
            err.to_string().contains(expected),
            "{err} does not contain {expected}"
        );
    }
    let (service, catalog, op) = client.resolve("jira", "createIssue").expect("resolve");
    let err = client
        .prepare(service, catalog, op, &CallArgs::default())
        .expect_err("body required");
    assert!(
        err.to_string()
            .contains("needs a request body; pass --json"),
        "{err}"
    );
}

#[tokio::test]
async fn multipart_uploads_send_the_file_and_the_xsrf_header() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/rest/api/3/issue/A-1/attachments"))
        .and(header("x-atlassian-token", "no-check"))
        .respond_with(|request: &Request| {
            let content_type = request
                .headers
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            let body = String::from_utf8_lossy(&request.body);
            let ok = content_type.starts_with("multipart/form-data")
                && body.contains("name=\"file\"")
                && body.contains("hello attachment");
            ResponseTemplate::new(if ok { 200 } else { 400 }).set_body_json(json!([]))
        })
        .expect(1)
        .mount(&server)
        .await;
    let dir = tempfile::tempdir().expect("tempdir");
    let file = dir.path().join("note.txt");
    std::fs::write(&file, "hello attachment").expect("write");
    let client = Client::with_connections([(
        Family::Jira,
        connection(&server, Deployment::Cloud, basic()),
    )]);

    let mut args = CallArgs::default().param("issueIdOrKey", "A-1");
    args.files.push(file);
    client
        .call("jira", "addAttachment", &args)
        .await
        .expect("upload accepted");
}

/// One case per pagination style: the first page, the query parameter the
/// second request must carry, and the final page.
#[tokio::test]
async fn pager_follows_every_atlassian_pagination_style() {
    struct Case {
        name: &'static str,
        first: fn(&str) -> (Value, Option<String>),
        second_param: (&'static str, &'static str),
    }
    let cases = [
        Case {
            name: "jira startAt/total",
            first: |_| {
                (
                    json!({ "startAt": 0, "maxResults": 2, "total": 3, "values": [1, 2] }),
                    None,
                )
            },
            second_param: ("startAt", "2"),
        },
        Case {
            name: "jira nextPageToken",
            first: |_| (json!({ "issues": [1], "nextPageToken": "tok" }), None),
            second_param: ("nextPageToken", "tok"),
        },
        Case {
            name: "confluence _links.next",
            first: |base| {
                (
                    json!({ "results": [1], "_links": { "next": "/list?cursor=c2", "base": base } }),
                    None,
                )
            },
            second_param: ("cursor", "c2"),
        },
        Case {
            name: "bitbucket dc nextPageStart",
            first: |_| {
                (
                    json!({ "values": [1], "isLastPage": false, "nextPageStart": 25 }),
                    None,
                )
            },
            second_param: ("start", "25"),
        },
        Case {
            name: "bitbucket cloud next url",
            first: |base| {
                (
                    json!({ "values": [1], "next": format!("{base}/list?page=2") }),
                    None,
                )
            },
            second_param: ("page", "2"),
        },
        Case {
            name: "admin links.next cursor",
            first: |_| (json!({ "data": [1], "links": { "next": "abc" } }), None),
            second_param: ("cursor", "abc"),
        },
        Case {
            name: "link header",
            first: |base| {
                (
                    json!({ "results": [1] }),
                    Some(format!("<{base}/list?cursor=h2>; rel=\"next\"")),
                )
            },
            second_param: ("cursor", "h2"),
        },
    ];

    for case in cases {
        let server = MockServer::start().await;
        let (key, value) = case.second_param;
        Mock::given(path("/list"))
            .and(query_param(key, value))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!({ "values": [3], "isLast": true })),
            )
            .expect(1)
            .mount(&server)
            .await;
        let (first, link) = (case.first)(&server.uri());
        let mut response = ResponseTemplate::new(200).set_body_json(first);
        if let Some(link) = link {
            response = response.insert_header("link", link.as_str());
        }
        Mock::given(path("/list"))
            .respond_with(response)
            .expect(1)
            .mount(&server)
            .await;

        let client = Client::with_connections([(
            Family::Jira,
            connection(&server, Deployment::Cloud, basic()),
        )]);
        let request = client
            .prepare_raw(Family::Jira, HttpMethod::Get, "/list", &Map::new(), None)
            .expect("prepare");
        let mut pager = Pager::new(&client, request, 10, Duration::ZERO);
        let mut pages = 0;
        while pager
            .next_page()
            .await
            .unwrap_or_else(|err| panic!("{}: {err}", case.name))
            .is_some()
        {
            pages += 1;
        }
        assert_eq!(pages, 2, "{}", case.name);
        server.verify().await;
    }
}

#[tokio::test]
async fn pager_stops_at_the_page_limit() {
    let server = MockServer::start().await;
    Mock::given(path("/list"))
        .respond_with(|request: &Request| {
            // Every page points at a new page, so only the limit stops the pager.
            let token = request
                .url
                .query_pairs()
                .find(|(k, _)| k == "nextPageToken")
                .map(|(_, v)| v.into_owned())
                .unwrap_or_default();
            ResponseTemplate::new(200)
                .set_body_json(json!({ "values": [1], "nextPageToken": format!("{token}x") }))
        })
        .mount(&server)
        .await;
    let client = Client::with_connections([(
        Family::Jira,
        connection(&server, Deployment::Cloud, basic()),
    )]);
    let request = client
        .prepare_raw(Family::Jira, HttpMethod::Get, "/list", &Map::new(), None)
        .expect("prepare");
    let mut pager = Pager::new(&client, request, 3, Duration::ZERO);
    let mut pages = 0;
    while pager.next_page().await.expect("page").is_some() {
        pages += 1;
    }
    assert_eq!(pages, 3);
}

#[tokio::test]
async fn verify_rejects_confluence_anonymous_answers() {
    let server = MockServer::start().await;
    Mock::given(path("/wiki/rest/api/user/current"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({ "type": "anonymous", "displayName": "Anonymous" })),
        )
        .mount(&server)
        .await;
    let err = verify(
        Family::Confluence,
        &connection(&server, Deployment::Cloud, basic()),
    )
    .await
    .expect_err("anonymous is not a login");
    assert!(err.is_auth(), "{err}");
}

#[tokio::test]
async fn rate_limits_and_outages_are_retried_only_when_safe() {
    let server = MockServer::start().await;
    // POST search: 429 is retried (the server did not process it).
    Mock::given(method("POST"))
        .and(path("/rest/api/3/search/jql"))
        .respond_with(ResponseTemplate::new(429).insert_header("retry-after", "0"))
        .up_to_n_times(1)
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/rest/api/3/search/jql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "issues": [] })))
        .expect(1)
        .mount(&server)
        .await;
    // POST create: 503 is not retried (it might have created the issue).
    Mock::given(method("POST"))
        .and(path("/rest/api/3/issue"))
        .respond_with(ResponseTemplate::new(503).insert_header("retry-after", "0"))
        .expect(1)
        .mount(&server)
        .await;
    // GET: 503 is retried until the retry budget is spent.
    Mock::given(method("GET"))
        .and(path("/rest/api/3/myself"))
        .respond_with(ResponseTemplate::new(503).insert_header("retry-after", "0"))
        .expect(3)
        .mount(&server)
        .await;
    let mut client = Client::with_connections([(
        Family::Jira,
        connection(&server, Deployment::Cloud, basic()),
    )]);
    client.set_max_retries(2);

    let search = CallArgs::default().body(json!({ "jql": "project = A" }));
    client
        .call("jira", "searchAndReconsileIssuesUsingJqlPost", &search)
        .await
        .expect("retried after 429");
    let create = CallArgs::default().body(json!({ "fields": {} }));
    let err = client
        .call("jira", "createIssue", &create)
        .await
        .expect_err("503 on POST");
    assert!(
        matches!(&err, Error::Http(http) if http.status == 503),
        "{err}"
    );
    let err = client
        .call("jira", "getCurrentUser", &CallArgs::default())
        .await
        .expect_err("still 503");
    assert!(
        matches!(&err, Error::Http(http) if http.status == 503 && http.retry_after == Some(0)),
        "{err}"
    );
}

#[tokio::test]
async fn oauth_code_exchange_and_refresh_per_provider() {
    use atl_sdk::oauth;
    use wiremock::matchers::{body_json, body_string_contains};
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/oauth/token"))
        .and(body_json(json!({
            "client_id": "cid", "client_secret": "sec", "grant_type": "authorization_code",
            "code": "c1", "redirect_uri": "http://localhost:8976/callback"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            json!({ "access_token": "a1", "refresh_token": "r1", "expires_in": 3600 }),
        ))
        .expect(1)
        .mount(&server)
        .await;
    let token_url = format!("{}/oauth/token", server.uri());
    let tokens = oauth::exchange_code(
        &token_url,
        "cid",
        "sec",
        "c1",
        "http://localhost:8976/callback",
    )
    .await
    .expect("exchange");
    assert_eq!(
        (
            tokens.access_token.as_str(),
            tokens.refresh_token.as_deref(),
            tokens.expires_in
        ),
        ("a1", Some("r1"), Some(3600))
    );

    // Bitbucket: form body and HTTP Basic client authentication.
    Mock::given(method("POST"))
        .and(path("/site/oauth2/access_token"))
        .and(header("authorization", "Basic a2V5OnNlYw=="))
        .and(body_string_contains("grant_type=refresh_token"))
        .and(body_string_contains("refresh_token=r0"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({ "access_token": "b2", "expires_in": 7200 })),
        )
        .expect(1)
        .mount(&server)
        .await;
    let old = Auth::OAuth {
        access_token: "b1".into(),
        refresh_token: "r0".into(),
        expires_at: 1,
        client_id: "key".into(),
        client_secret: "sec".into(),
        token_url: format!("{}/site/oauth2/access_token", server.uri()),
    };
    let Auth::OAuth {
        access_token,
        refresh_token,
        expires_at,
        ..
    } = oauth::refresh(&old).await.expect("refresh")
    else {
        panic!("not oauth");
    };
    assert_eq!(
        (access_token.as_str(), refresh_token.as_str()),
        ("b2", "r0")
    );
    assert!(expires_at > 1_700_000_000, "{expires_at}");
}

#[tokio::test]
async fn expired_and_rejected_oauth_tokens_are_refreshed_and_saved() {
    use wiremock::matchers::body_partial_json;
    let server = MockServer::start().await;
    for (old, new) in [("r1", "new1"), ("r2", "new2")] {
        Mock::given(method("POST"))
            .and(path("/oauth/token"))
            .and(body_partial_json(json!({ "grant_type": "refresh_token", "refresh_token": old })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "access_token": new, "refresh_token": if old == "r1" { "r2" } else { "r3" }, "expires_in": 3600
            })))
            .expect(1)
            .mount(&server)
            .await;
    }
    Mock::given(method("GET"))
        .and(path("/rest/api/3/myself"))
        .and(header("authorization", "Bearer new1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "accountId": "u1" })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/rest/api/3/issue/A-1"))
        .and(header("authorization", "Bearer new1"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({ "message": "expired" })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/rest/api/3/issue/A-1"))
        .and(header("authorization", "Bearer new2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "key": "A-1" })))
        .expect(1)
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().expect("tempdir");
    let config_path = dir.path().join("config.toml");
    let mut config = Config::default();
    config.profiles.entry("default".into()).or_default().insert(
        Family::Jira,
        Connection::new(
            Deployment::Cloud,
            server.uri(),
            Auth::OAuth {
                access_token: "old".into(),
                refresh_token: "r1".into(),
                expires_at: 1,
                client_id: "cid".into(),
                client_secret: "sec".into(),
                token_url: format!("{}/oauth/token", server.uri()),
            },
        ),
    );
    config.save_to(&config_path).expect("save");
    let client = Client::new(&Config::load_from(&config_path).expect("load"), None);

    // Expired: refreshed before the request.
    let me = client
        .send_json(
            Family::Jira,
            HttpMethod::Get,
            "/rest/api/3/myself",
            &[],
            None,
        )
        .await
        .expect("myself");
    assert_eq!(me["accountId"], "u1");
    let saved = std::fs::read_to_string(&config_path).expect("read");
    assert!(
        saved.contains("new1") && saved.contains("\"r2\""),
        "{saved}"
    );

    // Rejected: refreshed once and retried.
    let issue = client
        .send_json(
            Family::Jira,
            HttpMethod::Get,
            "/rest/api/3/issue/A-1",
            &[],
            None,
        )
        .await
        .expect("issue");
    assert_eq!(issue["key"], "A-1");
    let saved = std::fs::read_to_string(&config_path).expect("read");
    assert!(
        saved.contains("new2") && saved.contains("\"r3\""),
        "{saved}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn parallel_requests_refresh_an_expired_token_once() {
    use wiremock::matchers::body_partial_json;
    let server = MockServer::start().await;
    // Refresh tokens rotate: a second refresh with r1 would be rejected.
    Mock::given(method("POST"))
        .and(path("/oauth/token"))
        .and(body_partial_json(
            json!({ "grant_type": "refresh_token", "refresh_token": "r1" }),
        ))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(
                    json!({ "access_token": "new1", "refresh_token": "r2", "expires_in": 3600 }),
                )
                .set_delay(std::time::Duration::from_millis(200)),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/rest/api/3/myself"))
        .and(header("authorization", "Bearer new1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "accountId": "u1" })))
        .expect(8)
        .mount(&server)
        .await;
    let mut config = Config::default();
    config.profiles.entry("default".into()).or_default().insert(
        Family::Jira,
        Connection::new(
            Deployment::Cloud,
            server.uri(),
            Auth::OAuth {
                access_token: "old".into(),
                refresh_token: "r1".into(),
                expires_at: 1,
                client_id: "cid".into(),
                client_secret: "sec".into(),
                token_url: format!("{}/oauth/token", server.uri()),
            },
        ),
    );
    let client = Client::new(&config, None);
    let calls = (0..8).map(|_| {
        client.send_json(
            Family::Jira,
            HttpMethod::Get,
            "/rest/api/3/myself",
            &[],
            None,
        )
    });
    for result in futures::future::join_all(calls).await {
        assert_eq!(result.expect("myself")["accountId"], "u1");
    }
}

/// A self-signed test certificate and its key, made with openssl for these
/// tests only (not a secret).
const TEST_CERT: &str = "-----BEGIN CERTIFICATE-----\nMIIBXTCCAQKgAwIBAgIUdWqe6Pp4px8VHqQQgGcxgguYhNowCgYIKoZIzj0EAwIw\nEzERMA8GA1UEAwwIYXRsLXRlc3QwIBcNMjYxMTEzMDIyODAzWhgPMjEyNjEwMjAw\nMjI4MDNaMBMxETAPBgNVBAMMCGF0bC10ZXN0MFkwEwYHKoZIzj0CAQYIKoZIzj0D\nAQcDQgAEJCb2F3x4AtaoAF8bUruMA23YeoM4jmdxzhRP5OaM01FXUO5cv+x8MIMW\nrEfbFyAxhOzZ9mjZaFX2nz6tZk/NFaMyMDAwHQYDVR0OBBYEFL83XrEv9Lm0Chdq\nW/f6toK8V+gIMA8GA1UdEwEB/wQFMAMBAf8wCgYIKoZIzj0EAwIDSQAwRgIhAKCs\nIno+YgQ3c0E00+sRNz3kpO4YXlASaQmN/daC303qAiEAkKf1nfcUB7164UxKOPP3\n/S2GH6gcJSJelzyAbYigL4I=\n-----END CERTIFICATE-----\n";
const TEST_KEY: &str = "-----BEGIN PRIVATE KEY-----\nMIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgFARvmiB8gSmp09EQ\nelf3z/Ms30/V6ZzfY/bt9dAW9j+hRANCAAQkJvYXfHgC1qgAXxtSu4wDbdh6gziO\nZ3HOFE/k5ozTUVdQ7ly/7HwwgxasR9sXIDGE7Nn2aNloVfafPq1mT80V\n-----END PRIVATE KEY-----\n";

#[tokio::test]
async fn tls_files_are_checked_with_clear_errors() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/rest/api/2/myself"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "name": "ada" })))
        .expect(1)
        .mount(&server)
        .await;
    let dir = tempfile::tempdir().expect("tempdir");
    let (cert, key) = (dir.path().join("me.pem"), dir.path().join("me.key"));
    std::fs::write(&cert, TEST_CERT).expect("cert");
    std::fs::write(&key, TEST_KEY).expect("key");

    let missing = Client::with_connections([(
        Family::Jira,
        Connection {
            client_cert: Some(dir.path().join("nope.pem")),
            ..connection(&server, Deployment::DataCenter, basic())
        },
    )]);
    let err = missing
        .request(
            "jira",
            HttpMethod::Get,
            "/rest/api/2/myself",
            &Map::new(),
            None,
        )
        .await
        .expect_err("missing certificate");
    let text = err.to_string();
    assert!(
        text.contains("nope.pem")
            && text.contains("--client-cert")
            && text.contains("ATL_JIRA_CLIENT_CERT"),
        "{text}"
    );

    // A valid identity and CA bundle build a client; plain HTTP ignores them.
    let valid = Client::with_connections([(
        Family::Jira,
        Connection {
            client_cert: Some(cert.clone()),
            client_key: Some(key),
            ca_cert: Some(cert),
            ..connection(&server, Deployment::DataCenter, basic())
        },
    )]);
    valid
        .request(
            "jira",
            HttpMethod::Get,
            "/rest/api/2/myself",
            &Map::new(),
            None,
        )
        .await
        .expect("request with TLS settings");
}

#[tokio::test]
async fn cookie_auth_sends_one_cookie_header() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/rest/api/2/myself"))
        .and(header("cookie", "JSESSIONID=1; gw=2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "name": "ada" })))
        .expect(1)
        .mount(&server)
        .await;
    let client = Client::with_connections([(
        Family::Jira,
        Connection {
            extra_cookie: Some("gw=2".into()),
            ..connection(
                &server,
                Deployment::DataCenter,
                Auth::Cookie {
                    cookie: "JSESSIONID=1".into(),
                },
            )
        },
    )]);
    client
        .request(
            "jira",
            HttpMethod::Get,
            "/rest/api/2/myself",
            &Map::new(),
            None,
        )
        .await
        .expect("cookie request");
    let request = client
        .prepare_raw(Family::Jira, HttpMethod::Get, "/x", &Map::new(), None)
        .expect("prepare");
    assert_eq!(request.describe()["headers"]["cookie"], "***");
}

#[tokio::test]
async fn paging_never_follows_a_next_link_to_another_server() {
    let server = MockServer::start().await;
    let other = MockServer::start().await;
    Mock::given(path("/list"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "values": [1], "next": format!("{}/steal", other.uri())
        })))
        .expect(1)
        .mount(&server)
        .await;
    let client = Client::with_connections([(
        Family::Bitbucket,
        connection(&server, Deployment::Cloud, basic()),
    )]);
    let request = client
        .prepare_raw(
            Family::Bitbucket,
            HttpMethod::Get,
            "/list",
            &Map::new(),
            None,
        )
        .expect("prepare");
    let mut pager = Pager::new(&client, request, 10, Duration::ZERO);
    assert!(pager.next_page().await.expect("first page").is_some());
    assert!(pager.next_page().await.expect("stops").is_none());
    assert!(
        other
            .received_requests()
            .await
            .unwrap_or_default()
            .is_empty(),
        "no request to the other server"
    );
}
