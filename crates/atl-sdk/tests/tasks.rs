//! Task-level Jira and Confluence APIs against a mock server, on Cloud and
//! Data Center.

use atl_sdk::confluence::{Confluence, MovePosition, PageRef};
use atl_sdk::jira::{IssueEdit, Jira, NewIssue};
use atl_sdk::{Auth, Client, Connection, Deployment, Error, Family, markup};
use serde_json::json;
use wiremock::matchers::{body_json, body_partial_json, method, path, path_regex, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer, family: Family, deployment: Deployment) -> Client {
    Client::with_connections([(
        family,
        Connection::new(deployment, server.uri(), Auth::Bearer { token: "t".into() }),
    )])
}

fn issue(key: &str) -> serde_json::Value {
    json!({ "key": key, "fields": {
        "summary": "Fix login", "status": { "name": "In Progress", "statusCategory": { "key": "indeterminate" } },
        "issuetype": { "name": "Bug" }, "assignee": { "displayName": "Ada" }, "updated": "2026-11-01T10:00:00.000+0000"
    }})
}

#[tokio::test]
async fn jira_search_uses_each_deployments_endpoint_and_page_token() {
    let cloud = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/rest/api/3/search/jql"))
        .and(body_partial_json(
            json!({ "jql": "project = A", "maxResults": 2, "nextPageToken": "t1" }),
        ))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({ "issues": [issue("A-1")], "nextPageToken": "t2" })),
        )
        .expect(1)
        .mount(&cloud)
        .await;
    let page = Jira::new(&client(&cloud, Family::Jira, Deployment::Cloud))
        .search("project = A", 2, Some("t1"))
        .await
        .expect("cloud search");
    assert_eq!(page.next_page_token.as_deref(), Some("t2"));
    let first = &page.issues[0];
    assert_eq!(
        (
            first.key.as_str(),
            first.status.as_str(),
            first.assignee.as_deref()
        ),
        ("A-1", "In Progress", Some("Ada"))
    );
    assert_eq!(first.url, format!("{}/browse/A-1", cloud.uri()));

    let dc = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/rest/api/2/search"))
        .and(body_partial_json(json!({ "startAt": 50 })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({ "issues": [issue("A-51")], "startAt": 50, "total": 52 })),
        )
        .mount(&dc)
        .await;
    let page = Jira::new(&client(&dc, Family::Jira, Deployment::DataCenter))
        .search("project = A", 50, Some("50"))
        .await
        .expect("dc search");
    assert_eq!(
        (page.next_page_token.as_deref(), page.total),
        (Some("51"), Some(52))
    );
}

#[tokio::test]
async fn jira_rich_text_is_markdown_out_and_native_in() {
    let cloud = MockServer::start().await;
    let mut detail = issue("A-1");
    detail["fields"]["description"] = markup::markdown_to_adf("Steps:\n\n1. open **login**");
    detail["fields"]["comment"] = json!({ "total": 2, "comments": [
        { "id": "1", "author": { "displayName": "Bo" }, "body": markup::markdown_to_adf("old") },
        { "id": "2", "author": { "displayName": "Cy" }, "body": markup::markdown_to_adf("*new*") }
    ]});
    Mock::given(method("GET"))
        .and(path("/rest/api/3/issue/A-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(detail))
        .mount(&cloud)
        .await;
    Mock::given(method("POST"))
        .and(path("/rest/api/3/issue"))
        .and(body_partial_json(json!({ "fields": { "description": { "type": "doc", "content": [{ "type": "paragraph", "content": [{ "text": "bold", "marks": [{ "type": "strong" }] }] }] } } })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({ "key": "A-2" })))
        .expect(1)
        .mount(&cloud)
        .await;
    let jira_cloud = client(&cloud, Family::Jira, Deployment::Cloud);
    let jira = Jira::new(&jira_cloud);

    let view = jira.issue("A-1", 1, &[]).await.expect("view");
    assert_eq!(view.description, "Steps:\n\n1. open **login**");
    assert_eq!(
        (
            view.comments.len(),
            view.comment_count,
            view.comments[0].body.as_str()
        ),
        (1, 2, "*new*")
    );
    let new = NewIssue {
        project: "A".into(),
        issue_type: "Task".into(),
        summary: "S".into(),
        description: Some("**bold**".into()),
        ..Default::default()
    };
    assert_eq!(jira.create(&new).await.expect("create").key, "A-2");

    let dc = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/rest/api/2/issue/A-1"))
        .and(body_partial_json(json!({ "fields": { "description": "*bold*" }, "update": { "labels": [{ "add": "x" }, { "remove": "y" }] } })))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&dc)
        .await;
    let edit = IssueEdit {
        description: Some("**bold**".into()),
        add_labels: vec!["x".into()],
        remove_labels: vec!["y".into()],
        ..Default::default()
    };
    Jira::new(&client(&dc, Family::Jira, Deployment::DataCenter))
        .edit("A-1", &edit)
        .await
        .expect("dc edit");
}

#[tokio::test]
async fn jira_moves_by_status_name_and_assigns_by_account() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/rest/api/3/issue/A-1/transitions"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({ "transitions": [
                { "id": "11", "name": "Start", "to": { "name": "In Progress" } },
                { "id": "31", "name": "Finish", "to": { "name": "Done" } }
            ]})),
        )
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/rest/api/3/issue/A-1/transitions"))
        .and(body_partial_json(json!({ "transition": { "id": "31" } })))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(path("/rest/api/3/myself"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({ "accountId": "acc-1", "displayName": "Ada" })),
        )
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/rest/api/3/issue/A-1/assignee"))
        .and(body_partial_json(json!({ "accountId": "acc-1" })))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    let client = client(&server, Family::Jira, Deployment::Cloud);
    let jira = Jira::new(&client);

    assert_eq!(
        jira.transition("A-1", "done", None, &Default::default())
            .await
            .expect("move")
            .id,
        "31"
    );
    let err = jira
        .transition("A-1", "Review", None, &Default::default())
        .await
        .expect_err("no such status");
    assert!(
        err.to_string()
            .contains("it can move to: \"In Progress\", \"Done\""),
        "{err}"
    );
    jira.assign("A-1", "me").await.expect("assign");
}

#[tokio::test]
async fn jira_input_errors_list_the_valid_values() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/rest/api/3/issue/A-1/transitions"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({ "transitions": [
                { "id": "11", "name": "Start", "to": { "name": "In Progress" } },
                { "id": "31", "name": "Finish", "to": { "name": "Done" } }
            ]})),
        )
        .mount(&server)
        .await;
    let two_adas = json!([
        { "accountId": "a1", "displayName": "Ada Lovelace", "emailAddress": "ada@acme.com" },
        { "accountId": "a2", "displayName": "Ada King" }
    ]);
    Mock::given(method("GET"))
        .and(path("/rest/api/3/user/search"))
        .and(query_param("query", "Ada"))
        .respond_with(ResponseTemplate::new(200).set_body_json(two_adas.clone()))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/rest/api/3/user/search"))
        .and(query_param("query", "ada@acme.com"))
        .respond_with(ResponseTemplate::new(200).set_body_json(two_adas))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/rest/api/3/user/assignable/search"))
        .and(query_param("issueKey", "A-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            { "accountId": "a3", "displayName": "Bo" }
        ])))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/rest/api/3/issue/A-1/assignee"))
        .and(body_partial_json(json!({ "accountId": "a1" })))
        .respond_with(ResponseTemplate::new(204))
        .expect(2)
        .mount(&server)
        .await;
    let client = client(&server, Family::Jira, Deployment::Cloud);
    let jira = Jira::new(&client);
    let need = |err: Error| match err {
        Error::NeedsInput(need) => need,
        other => panic!("expected NeedsInput, got {other}"),
    };

    let missing = need(
        jira.transition("A-1", "", None, &Default::default())
            .await
            .expect_err("no status"),
    );
    assert_eq!(missing.arg, "status");
    let values: Vec<&str> = missing.choices.iter().map(|c| c.value.as_str()).collect();
    assert_eq!(values, ["In Progress", "Done"]);
    assert_eq!(missing.choices[1].label, "Done (Finish)");
    assert!(
        missing.message.starts_with("pass a status for A-1"),
        "{}",
        missing.message
    );

    let ambiguous = need(jira.assign("A-1", "Ada").await.expect_err("two users"));
    assert_eq!(ambiguous.arg, "user");
    assert_eq!(ambiguous.choices[0].value, "id:a1");
    assert_eq!(ambiguous.choices[0].label, "Ada Lovelace <ada@acme.com>");
    // An exact email among several results needs no choice; neither does id:.
    jira.assign("A-1", "ada@acme.com")
        .await
        .expect("exact email");
    jira.assign("A-1", "id:a1").await.expect("id");

    let nobody = need(jira.assign("A-1", "").await.expect_err("no user"));
    let values: Vec<&str> = nobody.choices.iter().map(|c| c.value.as_str()).collect();
    assert_eq!(values, ["me", "x", "id:a3"]);
    assert_eq!(
        Error::NeedsInput(nobody).to_json()["input"]["arg"],
        json!("user")
    );
}

#[tokio::test]
async fn confluence_cloud_resolves_titles_reads_markdown_and_bumps_versions() {
    let server = MockServer::start().await;
    Mock::given(path("/wiki/api/v2/spaces"))
        .and(query_param("keys", "DOCS"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({ "results": [{ "id": "77", "key": "DOCS" }] })),
        )
        .mount(&server)
        .await;
    Mock::given(path("/wiki/api/v2/pages"))
        .and(query_param("space-id", "77"))
        .and(query_param("title", "Handbook"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({ "results": [{ "id": "5" }] })),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/pages/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "5", "title": "Handbook", "spaceId": "77", "version": { "number": 3 },
            "body": { "storage": { "value": "<h1>Hi</h1><p>Read <em>this</em>.</p>" } },
            "_links": { "webui": "/spaces/DOCS/pages/5/Handbook" }
        })))
        .mount(&server)
        .await;
    Mock::given(path("/wiki/api/v2/spaces/77"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "key": "DOCS" })))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/wiki/api/v2/pages/5"))
        .and(body_partial_json(json!({ "title": "Handbook", "version": { "number": 4 }, "body": { "value": "<p>New</p>" } })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "id": "5", "title": "Handbook", "_links": { "webui": "/x" } })))
        .expect(1)
        .mount(&server)
        .await;
    let client = client(&server, Family::Confluence, Deployment::Cloud);
    let confluence = Confluence::new(&client);
    let handbook = PageRef::parse("DOCS:Handbook").expect("ref");

    let page = confluence.page(&handbook).await.expect("page");
    assert_eq!(page.markdown, "# Hi\n\nRead *this*.");
    assert_eq!((page.space.as_deref(), page.version), (Some("DOCS"), 3));
    assert_eq!(
        page.url,
        format!("{}/wiki/spaces/DOCS/pages/5/Handbook", server.uri())
    );
    confluence
        .update(&handbook, None, Some(markup::markdown_to_storage("New")))
        .await
        .expect("update");
}

#[tokio::test]
async fn confluence_data_center_creates_with_the_content_api() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/rest/api/content"))
        .and(body_partial_json(json!({ "type": "page", "title": "Notes", "space": { "key": "OPS" }, "ancestors": [{ "id": "9" }], "body": { "storage": { "representation": "storage" } } })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "id": "10", "title": "Notes", "_links": { "webui": "/display/OPS/Notes" } })))
        .expect(1)
        .mount(&server)
        .await;
    let client = client(&server, Family::Confluence, Deployment::DataCenter);
    let created = Confluence::new(&client)
        .create(
            "OPS",
            "Notes",
            Some(&PageRef::Id("9".into())),
            "<p>x</p>".into(),
        )
        .await
        .expect("create");
    assert_eq!(created.url, format!("{}/display/OPS/Notes", server.uri()));
}

#[tokio::test]
async fn read_only_connections_refuse_writes_but_allow_searches() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/rest/api/3/search/jql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "issues": [] })))
        .expect(1)
        .mount(&server)
        .await;
    let connection = Connection {
        read_only: true,
        ..Connection::new(
            Deployment::Cloud,
            server.uri(),
            Auth::Bearer { token: "t".into() },
        )
    };
    let client = Client::with_connections([(Family::Jira, connection)]);
    let jira = Jira::new(&client);

    jira.search("project = A", 10, None)
        .await
        .expect("POST search is a read");
    let err = jira
        .comment("A-1", "hi", false)
        .await
        .expect_err("comment is a write");
    assert!(matches!(err, Error::ReadOnly(_)), "{err:?}");
}

#[tokio::test]
async fn jql_is_checked_and_counted_on_both_deployments() {
    let cloud = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/rest/api/3/jql/parse"))
        .and(query_param("validation", "strict"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "queries": [{ "query": "project =", "errors": ["Expecting a value after '='."] }] })))
        .mount(&cloud)
        .await;
    Mock::given(method("POST"))
        .and(path("/rest/api/3/search/approximate-count"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "count": 42 })))
        .mount(&cloud)
        .await;
    let client_cloud = client(&cloud, Family::Jira, Deployment::Cloud);
    let jira = Jira::new(&client_cloud);
    let check = jira.check_jql("project =").await.expect("check");
    assert_eq!(
        (check.valid, check.errors),
        (false, vec!["Expecting a value after '='.".to_owned()])
    );
    assert_eq!(jira.count("project = A").await.expect("count"), 42);

    let dc = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/rest/api/2/search"))
        .and(body_partial_json(json!({ "jql": "bad ===" })))
        .respond_with(
            ResponseTemplate::new(400)
                .set_body_json(json!({ "errorMessages": ["Error in the JQL Query"] })),
        )
        .mount(&dc)
        .await;
    Mock::given(method("POST"))
        .and(path("/rest/api/2/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "total": 7, "issues": [] })))
        .mount(&dc)
        .await;
    let client_dc = client(&dc, Family::Jira, Deployment::DataCenter);
    let jira = Jira::new(&client_dc);
    let check = jira.check_jql("bad ===").await.expect("dc check");
    assert_eq!(
        (check.valid, check.errors),
        (false, vec!["Error in the JQL Query".to_owned()])
    );
    assert!(jira.check_jql("project = A").await.expect("valid").valid);
    assert_eq!(jira.count("project = A").await.expect("dc count"), 7);
}

#[tokio::test]
async fn links_read_naturally_and_filters_resolve_by_name() {
    let server = MockServer::start().await;
    Mock::given(path("/rest/api/3/issueLinkType"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "issueLinkTypes": [{ "name": "Blocks", "outward": "blocks", "inward": "is blocked by" }] })))
        .mount(&server)
        .await;
    // "A-2 is blocked by A-1" is stored as A-1 (outward) blocks A-2 (inward).
    Mock::given(method("POST"))
        .and(path("/rest/api/3/issueLink"))
        .and(body_partial_json(json!({ "type": { "name": "Blocks" }, "outwardIssue": { "key": "A-1" }, "inwardIssue": { "key": "A-2" } })))
        .respond_with(ResponseTemplate::new(201))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(path("/rest/api/3/filter/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "values": [
            { "id": "10", "name": "My bugs", "jql": "type = Bug" },
            { "id": "11", "name": "My bugs (old)", "jql": "type = Bug AND created < -1y" }
        ]})))
        .mount(&server)
        .await;
    let client = client(&server, Family::Jira, Deployment::Cloud);
    let jira = Jira::new(&client);

    jira.link("A-2", "A-1", "is blocked by")
        .await
        .expect("link");
    let err = jira
        .link("A-1", "A-2", "Relates")
        .await
        .expect_err("unknown type");
    assert!(
        err.to_string()
            .contains("choices: Blocks (blocks / is blocked by)"),
        "{err}"
    );
    assert_eq!(
        jira.filter("my BUGS").await.expect("exact name").jql,
        "type = Bug"
    );
    assert!(
        jira.filter("bugs")
            .await
            .expect_err("not exact")
            .to_string()
            .contains("choices: 10 (My bugs), 11 (My bugs (old))")
    );
}

#[tokio::test]
async fn attachments_upload_and_download_with_credentials_kept_on_the_server() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/rest/api/3/issue/A-1/attachments"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!([{ "id": "9", "filename": "log.txt", "size": 5 }])),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(path("/rest/api/3/attachment/9"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({ "id": "9", "filename": "log.txt", "size": 5 })),
        )
        .mount(&server)
        .await;
    Mock::given(path("/rest/api/3/attachment/content/9"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"hello".to_vec()))
        .mount(&server)
        .await;
    let dir = tempfile::tempdir().expect("tempdir");
    let file = dir.path().join("log.txt");
    std::fs::write(&file, "hello").expect("write");
    let client = client(&server, Family::Jira, Deployment::Cloud);
    let jira = Jira::new(&client);

    assert_eq!(
        jira.attach("A-1", &[file]).await.expect("attach")[0].filename,
        "log.txt"
    );
    let (meta, bytes) = jira.download("9").await.expect("download");
    assert_eq!((meta.size, bytes.as_slice()), (5, b"hello".as_slice()));
    let err = client
        .download(Family::Jira, "https://evil.example/steal")
        .await
        .expect_err("foreign host");
    assert!(
        err.to_string().contains("is not on the jira server"),
        "{err}"
    );
}

#[tokio::test]
async fn bitbucket_pull_requests_map_both_apis_and_data_center_merges_the_current_version() {
    let cloud = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2.0/repositories/acme/web/pullrequests"))
        .and(query_param("state", "OPEN"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "values": [{
            "id": 7, "title": "Fix header", "state": "OPEN",
            "author": { "display_name": "Ada" },
            "source": { "branch": { "name": "web-42" } }, "destination": { "branch": { "name": "main" } },
            "participants": [{ "user": { "display_name": "Bob" }, "approved": true }],
            "links": { "html": { "href": "https://bitbucket.org/acme/web/pull-requests/7" } }
        }]})))
        .expect(1)
        .mount(&cloud)
        .await;
    let client = client(&cloud, Family::Bitbucket, Deployment::Cloud);
    let prs = atl_sdk::tasks::run(&client, "bitbucket_pr_list", json!({ "repo": "acme/web" }))
        .await
        .expect("cloud list");
    assert_eq!(
        (
            &prs[0]["id"],
            &prs[0]["source"],
            &prs[0]["destination"],
            &prs[0]["approved_by"]
        ),
        (&json!(7), &json!("web-42"), &json!("main"), &json!(["Bob"]))
    );

    let dc = MockServer::start().await;
    let pr = "/rest/api/latest/projects/PROJ/repos/web/pull-requests/7";
    Mock::given(method("GET"))
        .and(path(pr))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "id": 7, "version": 3 })))
        .mount(&dc)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{pr}/merge")))
        .and(query_param("version", "3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 7, "state": "MERGED", "fromRef": { "displayId": "web-42" }, "toRef": { "displayId": "main" },
            "updatedDate": 1_700_000_000_000_i64
        })))
        .expect(1)
        .mount(&dc)
        .await;
    let client = self::client(&dc, Family::Bitbucket, Deployment::DataCenter);
    let merged = atl_sdk::tasks::run(
        &client,
        "bitbucket_pr_merge",
        json!({ "repo": "PROJ/web", "id": 7 }),
    )
    .await
    .expect("dc merge");
    assert_eq!(
        (&merged["state"], &merged["updated"]),
        (&json!("MERGED"), &json!("2023-11-14T22:13:20Z"))
    );
}

#[tokio::test]
async fn service_desk_requests_resolve_the_desk_key_and_request_type_name() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/rest/servicedeskapi/servicedesk"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            json!({ "values": [{ "id": "4", "projectKey": "HELP", "projectName": "Help desk" }] }),
        ))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/rest/servicedeskapi/servicedesk/4/requesttype"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({ "values": [{ "id": "11", "name": "Get IT help" }] })),
        )
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/rest/servicedeskapi/request"))
        .and(body_partial_json(json!({
            "serviceDeskId": "4", "requestTypeId": "11",
            "requestFieldValues": { "summary": "Laptop broken" }
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "issueKey": "HELP-9", "currentStatus": { "status": "Waiting for support" },
            "requestFieldValues": [{ "fieldId": "summary", "value": "Laptop broken" }],
            "_links": { "web": "https://x/servicedesk/customer/portal/4/HELP-9" }
        })))
        .expect(1)
        .mount(&server)
        .await;
    let client = client(&server, Family::Jira, Deployment::Cloud);
    let args = json!({ "desk": "help", "type": "get it help", "summary": "Laptop broken" });
    let created = atl_sdk::tasks::run(&client, "jsm_request_create", args)
        .await
        .expect("create");
    assert_eq!(
        (&created["key"], &created["status"], &created["summary"]),
        (
            &json!("HELP-9"),
            &json!("Waiting for support"),
            &json!("Laptop broken")
        )
    );
    let err = atl_sdk::tasks::run(
        &client,
        "jsm_request_create",
        json!({ "desk": "OPS", "type": "x", "summary": "y" }),
    )
    .await
    .expect_err("unknown desk");
    assert!(err.to_string().contains("\"HELP\" (4)"), "{err}");
}

fn field(id: &str, name: &str, schema: serde_json::Value) -> serde_json::Value {
    json!({ "id": id, "name": name, "custom": id.starts_with("customfield"), "schema": schema })
}

#[tokio::test]
async fn jira_custom_fields_are_converted_by_type() {
    let cloud = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/rest/api/3/field"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            field("customfield_1", "Story Points", json!({ "type": "number" })),
            field("customfield_2", "Team", json!({ "type": "option" })),
            field("customfield_3", "Tags", json!({ "type": "array", "items": "string" })),
            field("customfield_4", "Reviewer", json!({ "type": "user" })),
            field("customfield_5", "Notes", json!({ "type": "string", "custom": "com.atlassian.jira.plugin.system.customfieldtypes:textarea" })),
            field("customfield_6", "Twin", json!({ "type": "string" })),
            field("customfield_7", "Twin", json!({ "type": "string" })),
            field("summary", "Summary", json!({ "type": "string" })),
        ])))
        .mount(&cloud)
        .await;
    Mock::given(method("GET"))
        .and(path("/rest/api/3/user/search"))
        .and(query_param("query", "ada@acme.com"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([{ "accountId": "u1" }])))
        .mount(&cloud)
        .await;
    Mock::given(method("POST"))
        .and(path("/rest/api/3/issue"))
        .and(body_partial_json(json!({ "fields": {
            "customfield_1": 3.0,
            "customfield_2": { "value": "Core" },
            "customfield_3": ["a", "b"],
            "customfield_4": { "accountId": "u1" },
            "customfield_5": { "type": "doc" },
            "customfield_6": { "raw": true },
        } })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({ "key": "A-3" })))
        .expect(1)
        .mount(&cloud)
        .await;
    let client = client(&cloud, Family::Jira, Deployment::Cloud);
    let jira = Jira::new(&client);
    let text = |s: &str| json!(s);
    let new = NewIssue {
        project: "A".into(),
        issue_type: "Task".into(),
        summary: "S".into(),
        fields: vec![
            ("story points".into(), text("3")),
            ("Team".into(), text("Core")),
            ("Tags".into(), text("a, b")),
            ("Reviewer".into(), text("ada@acme.com")),
            ("Notes".into(), text("**hi**")),
            ("customfield_6".into(), text(r#"{"raw": true}"#)),
        ],
        ..Default::default()
    };
    assert_eq!(jira.create(&new).await.expect("create").key, "A-3");

    for (name, value, expected) in [
        (
            "Twin",
            "x",
            "2 fields are named `Twin`; pass the ID instead; choices: customfield_6 (Twin), customfield_7 (Twin)",
        ),
        ("Nope", "x", "Jira has no field named `Nope`"),
        ("Story Points", "many", "field `Story Points` is a number"),
    ] {
        let edit = IssueEdit {
            fields: vec![(name.into(), text(value))],
            ..Default::default()
        };
        let err = jira.edit("A-1", &edit).await.expect_err(name);
        assert!(err.to_string().contains(expected), "{err}");
    }
}

#[tokio::test]
async fn jira_delete_and_watchers_on_cloud_and_data_center() {
    let cloud = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/rest/api/3/issue/A-1"))
        .and(query_param("deleteSubtasks", "true"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&cloud)
        .await;
    Mock::given(method("GET"))
        .and(path("/rest/api/3/myself"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({ "accountId": "u1", "displayName": "Ada" })),
        )
        .mount(&cloud)
        .await;
    Mock::given(method("POST"))
        .and(path("/rest/api/3/issue/A-1/watchers"))
        .and(body_json(json!("u1")))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&cloud)
        .await;
    Mock::given(method("GET"))
        .and(path("/rest/api/3/issue/A-1/watchers"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "isWatching": true, "watchCount": 1,
            "watchers": [{ "accountId": "u1", "displayName": "Ada" }]
        })))
        .mount(&cloud)
        .await;
    let cloud_client = client(&cloud, Family::Jira, Deployment::Cloud);
    let jira = Jira::new(&cloud_client);
    jira.delete("A-1", true).await.expect("delete");
    jira.watch("A-1", "me").await.expect("watch");
    let watchers = jira.watchers("A-1").await.expect("watchers");
    assert!(watchers.watching);
    assert_eq!(
        (watchers.count, watchers.watchers[0].id.as_str()),
        (1, "u1")
    );
    let err = jira.watch("A-1", "x").await.expect_err("nobody");
    assert!(err.to_string().contains("is not a user"), "{err}");

    let dc = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/rest/api/2/user/search"))
        .and(query_param("username", "ada"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([{ "name": "ada" }])))
        .mount(&dc)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/rest/api/2/issue/A-1/watchers"))
        .and(query_param("username", "ada"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&dc)
        .await;
    let dc_client = client(&dc, Family::Jira, Deployment::DataCenter);
    Jira::new(&dc_client)
        .unwatch("A-1", "ada")
        .await
        .expect("unwatch");
}

#[tokio::test]
async fn confluence_comments_labels_move_and_history() {
    let cloud = MockServer::start().await;
    let get = |p: &str, body: serde_json::Value| {
        Mock::given(method("GET"))
            .and(path(p.to_owned()))
            .respond_with(ResponseTemplate::new(200).set_body_json(body))
    };
    get(
        "/wiki/api/v2/pages/12/footer-comments",
        json!({ "results": [{ "id": "c1", "version": { "authorId": "a1", "createdAt": "2026-11-01" }, "body": { "storage": { "value": "<p><strong>ok</strong></p>" } } }] }),
    )
    .mount(&cloud)
    .await;
    Mock::given(method("GET"))
        .and(path("/wiki/rest/api/user/bulk"))
        .and(query_param("accountId", "a1"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({ "results": [{ "accountId": "a1", "displayName": "Ada" }] })),
        )
        .mount(&cloud)
        .await;
    Mock::given(method("POST"))
        .and(path("/wiki/api/v2/footer-comments"))
        .and(body_partial_json(
            json!({ "pageId": "12", "body": { "representation": "storage" } }),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "id": "c2" })))
        .expect(1)
        .mount(&cloud)
        .await;
    Mock::given(method("POST"))
        .and(path("/wiki/rest/api/content/12/label"))
        .and(body_json(json!([{ "prefix": "global", "name": "howto" }])))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "results": [] })))
        .expect(1)
        .mount(&cloud)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/wiki/rest/api/content/12/label/draft"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&cloud)
        .await;
    get(
        "/wiki/api/v2/pages/12/labels",
        json!({ "results": [{ "name": "howto" }] }),
    )
    .mount(&cloud)
    .await;
    Mock::given(method("PUT"))
        .and(path("/wiki/rest/api/content/12/move/after/34"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "pageId": "12" })))
        .expect(1)
        .mount(&cloud)
        .await;
    get(
        "/wiki/api/v2/pages/12/versions",
        json!({ "results": [{ "number": 4, "authorId": "a1", "createdAt": "2026-11-02", "message": "typo", "minorEdit": true }] }),
    )
    .mount(&cloud)
    .await;
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/pages/12"))
        .and(query_param("version", "3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "12", "title": "Old", "version": { "number": 3 }, "body": { "storage": { "value": "<p>v3</p>" } }
        })))
        .mount(&cloud)
        .await;
    let cloud_client = client(&cloud, Family::Confluence, Deployment::Cloud);
    let c = Confluence::new(&cloud_client);
    let page = PageRef::Id("12".into());
    let comments = c.comments(&page, 50, false).await.expect("comments");
    assert_eq!(
        (comments[0].author.as_deref(), comments[0].body.as_str()),
        (Some("Ada"), "**ok**")
    );
    assert_eq!(
        c.comment(&page, "hi", None).await.expect("comment").id,
        "c2"
    );
    let labels = c
        .change_labels(&page, &["howto".into()], &["draft".into()])
        .await
        .expect("labels");
    assert_eq!(labels, ["howto"]);
    c.move_page(&page, MovePosition::After, &PageRef::Id("34".into()))
        .await
        .expect("move");
    let versions = c.versions(&page, 25).await.expect("versions");
    assert_eq!(
        (
            versions[0].number,
            versions[0].author.as_deref(),
            versions[0].minor_edit
        ),
        (4, Some("Ada"), true)
    );
    let old = c.page_at(&page, Some(3)).await.expect("old version");
    assert_eq!((old.version, old.markdown.as_str()), (3, "v3"));

    let dc = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/rest/api/content/12"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "12", "title": "Guide", "version": { "number": 7 }, "body": { "storage": { "value": "<p>x</p>" } }
        })))
        .mount(&dc)
        .await;
    Mock::given(method("PUT"))
        .and(path("/rest/api/content/12"))
        .and(body_partial_json(json!({ "version": { "number": 8 }, "ancestors": [{ "id": "34" }], "body": { "storage": { "value": "<p>x</p>" } } })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "id": "12", "title": "Guide" })))
        .expect(1)
        .mount(&dc)
        .await;
    Mock::given(method("GET"))
        .and(path("/rest/experimental/content/12/version"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "results": [{ "number": 7, "by": { "displayName": "Bo" }, "when": "2026-10-01" }] })))
        .mount(&dc)
        .await;
    let dc_client = client(&dc, Family::Confluence, Deployment::DataCenter);
    let c = Confluence::new(&dc_client);
    let err = c
        .move_page(&page, MovePosition::Before, &PageRef::Id("34".into()))
        .await
        .expect_err("before on DC");
    assert!(err.to_string().contains("pass --parent"), "{err}");
    c.move_page(&page, MovePosition::Append, &PageRef::Id("34".into()))
        .await
        .expect("dc move");
    assert_eq!(
        c.versions(&page, 25).await.expect("dc versions")[0]
            .author
            .as_deref(),
        Some("Bo")
    );
}

#[tokio::test]
async fn bitbucket_review_actions_and_pipeline_logs() {
    use atl_sdk::bitbucket::{Bitbucket, RepoRef};
    let ok = |body: serde_json::Value| ResponseTemplate::new(200).set_body_json(body);
    let cloud = MockServer::start().await;
    let pr = "/2.0/repositories/acme/web/pullrequests/1";
    Mock::given(method("POST"))
        .and(path(format!("{pr}/decline")))
        .respond_with(ok(json!({ "id": 1, "state": "DECLINED" })))
        .expect(1)
        .mount(&cloud)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{pr}/request-changes")))
        .respond_with(ok(json!({})))
        .expect(1)
        .mount(&cloud)
        .await;
    Mock::given(method("GET"))
        .and(path(pr))
        .respond_with(ok(json!({ "id": 1, "title": "T", "reviewers": [{ "uuid": "{b}", "display_name": "Bo" }] })))
        .mount(&cloud)
        .await;
    Mock::given(method("GET"))
        .and(path("/2.0/workspaces/acme/members"))
        .respond_with(ok(
            json!({ "values": [{ "user": { "display_name": "Ada Lovelace", "uuid": "{a}" } }] }),
        ))
        .mount(&cloud)
        .await;
    Mock::given(method("PUT"))
        .and(path(pr))
        .and(body_json(
            json!({ "title": "T", "reviewers": [{ "uuid": "{a}" }] }),
        ))
        .respond_with(ok(json!({ "id": 1, "title": "T" })))
        .expect(1)
        .mount(&cloud)
        .await;
    Mock::given(method("PUT"))
        .and(path(pr))
        .and(body_json(
            json!({ "title": "New", "destination": { "branch": { "name": "dev" } } }),
        ))
        .respond_with(ok(json!({ "id": 1, "title": "New" })))
        .expect(1)
        .mount(&cloud)
        .await;
    let pipelines = "/2.0/repositories/acme/web/pipelines";
    Mock::given(method("GET"))
        .and(path(format!("{pipelines}/345")))
        .respond_with(ok(json!({ "uuid": "{p1}", "build_number": 345, "state": { "name": "COMPLETED", "result": { "name": "FAILED" } } })))
        .mount(&cloud)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{pipelines}/%7Bp1%7D/steps")))
        .respond_with(ok(json!({ "values": [
            { "uuid": "{s1}", "name": "Build", "state": { "name": "COMPLETED", "result": { "name": "SUCCESSFUL" } } },
            { "uuid": "{s2}", "name": "Unit tests", "state": { "name": "COMPLETED", "result": { "name": "FAILED" } } }
        ] })))
        .mount(&cloud)
        .await;
    for (step, log) in [("s1", "built\n"), ("s2", "boom\n")] {
        Mock::given(method("GET"))
            .and(path(format!("{pipelines}/%7Bp1%7D/steps/%7B{step}%7D/log")))
            .respond_with(ResponseTemplate::new(200).set_body_string(log))
            .mount(&cloud)
            .await;
    }
    let cloud_client = client(&cloud, Family::Bitbucket, Deployment::Cloud);
    let b = Bitbucket::new(&cloud_client);
    let repo = RepoRef::parse("acme/web").expect("repo");
    assert_eq!(
        b.decline(&repo, 1).await.expect("decline").state,
        "DECLINED"
    );
    b.request_changes(&repo, 1).await.expect("request changes");
    b.set_reviewers(&repo, 1, &["ada lovelace".into()], &["Bo".into()])
        .await
        .expect("reviewers");
    let err = b
        .set_reviewers(&repo, 1, &[], &["Cy".into()])
        .await
        .expect_err("not a reviewer");
    assert!(
        err.to_string().contains("`Cy` is not a reviewer of #1"),
        "{err}"
    );
    b.edit_pull_request(&repo, 1, Some("New"), None, Some("dev"))
        .await
        .expect("edit");
    let detail = b.pipeline(&repo, "345").await.expect("pipeline");
    assert_eq!(
        (detail.pipeline.result.as_deref(), detail.steps.len()),
        (Some("FAILED"), 2)
    );
    assert_eq!(
        b.pipeline_logs(&repo, "345", Some("unit tests"))
            .await
            .expect("one log"),
        "boom\n"
    );
    assert_eq!(
        b.pipeline_logs(&repo, "345", None).await.expect("all logs"),
        "== Build (SUCCESSFUL) ==\nbuilt\n== Unit tests (FAILED) ==\nboom\n"
    );

    let dc = MockServer::start().await;
    let pr = "/rest/api/latest/projects/P/repos/web/pull-requests/2";
    Mock::given(method("GET"))
        .and(path(pr))
        .respond_with(ok(json!({ "id": 2, "version": 5, "title": "T", "description": "d", "reviewers": [{ "user": { "name": "bo" } }] })))
        .mount(&dc)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{pr}/decline")))
        .and(query_param("version", "5"))
        .respond_with(ok(json!({ "id": 2, "state": "DECLINED" })))
        .expect(1)
        .mount(&dc)
        .await;
    Mock::given(method("GET"))
        .and(path("/plugins/servlet/applinks/whoami"))
        .respond_with(ResponseTemplate::new(200).set_body_string("ada"))
        .mount(&dc)
        .await;
    Mock::given(method("GET"))
        .and(path("/rest/api/latest/users/ada"))
        .respond_with(ok(json!({ "name": "ada", "slug": "ada" })))
        .mount(&dc)
        .await;
    Mock::given(method("PUT"))
        .and(path(format!("{pr}/participants/ada")))
        .and(body_partial_json(json!({ "status": "NEEDS_WORK" })))
        .respond_with(ok(json!({})))
        .expect(1)
        .mount(&dc)
        .await;
    Mock::given(method("PUT"))
        .and(path(pr))
        .and(body_json(json!({ "version": 5, "title": "New", "description": "d", "reviewers": [{ "user": { "name": "bo" } }], "toRef": { "id": "refs/heads/dev" } })))
        .respond_with(ok(json!({ "id": 2, "title": "New" })))
        .expect(1)
        .mount(&dc)
        .await;
    let dc_client = client(&dc, Family::Bitbucket, Deployment::DataCenter);
    let b = Bitbucket::new(&dc_client);
    let repo = RepoRef::parse("P/web").expect("repo");
    b.decline(&repo, 2).await.expect("dc decline");
    b.request_changes(&repo, 2).await.expect("dc needs work");
    b.edit_pull_request(&repo, 2, Some("New"), None, Some("dev"))
        .await
        .expect("dc edit");
}

#[tokio::test]
async fn trello_boards_lists_and_cards_by_name() {
    use atl_sdk::trello::Trello;
    use wiremock::matchers::query_param_is_missing;
    let ok = |body: serde_json::Value| ResponseTemplate::new(200).set_body_json(body);
    let server = MockServer::start().await;
    let (board, todo, done) = (
        "b0000000000000000000000b",
        "a1111111111111111111111a",
        "a2222222222222222222222a",
    );
    Mock::given(method("GET"))
        .and(path("/1/members/me/boards"))
        .respond_with(ok(
            json!([{ "id": board, "name": "Roadmap", "shortLink": "RdMp1234" }]),
        ))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/1/boards/{board}/lists")))
        .respond_with(ok(
            json!([{ "id": todo, "name": "To do" }, { "id": done, "name": "Done" }]),
        ))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/1/lists/{todo}/cards")))
        .respond_with(ok(json!([{ "id": "c1", "name": "Write docs", "idList": todo, "labels": [{ "name": "docs" }], "shortLink": "AbCd1234" }])))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/1/cards"))
        .and(query_param("idList", todo))
        .and(query_param("name", "Ship it"))
        .and(query_param_is_missing("due"))
        .respond_with(ok(json!({ "id": "c2", "name": "Ship it", "idList": todo })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/1/cards/AbCd1234"))
        .and(query_param("fields", "idBoard"))
        .respond_with(ok(json!({ "idBoard": board })))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/1/cards/AbCd1234"))
        .and(query_param("idList", done))
        .respond_with(ok(
            json!({ "id": "c1", "name": "Write docs", "idList": done }),
        ))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/1/cards/AbCd1234"))
        .and(query_param("list", "true"))
        .respond_with(ok(json!({
            "id": "c1", "name": "Write docs", "desc": "**Steps**", "idList": todo,
            "list": { "name": "To do" }, "board": { "name": "Roadmap" },
            "actions": [{ "date": "2026-11-02", "memberCreator": { "fullName": "Ada" }, "data": { "text": "hi" } }]
        })))
        .mount(&server)
        .await;
    let client = client(&server, Family::Trello, Deployment::Cloud);
    let trello = Trello::new(&client);

    let cards = trello
        .cards(Some("roadmap"), Some("to do"), false)
        .await
        .expect("cards");
    assert_eq!(
        (cards[0].list.as_deref(), cards[0].labels.as_slice()),
        (Some("To do"), ["docs".to_owned()].as_slice())
    );
    trello
        .create_card(Some("RdMp1234"), "To do", "Ship it", None, None)
        .await
        .expect("create");
    let moved = trello
        .move_card("https://trello.com/c/AbCd1234/1-write-docs", "Done", None)
        .await
        .expect("move");
    assert_eq!(moved.list.as_deref(), Some("Done"));
    let card = trello.card("AbCd1234").await.expect("view");
    assert_eq!(
        (
            card.board.as_deref(),
            card.card.list.as_deref(),
            card.comments[0].author.as_deref()
        ),
        (Some("Roadmap"), Some("To do"), Some("Ada"))
    );
    let err = trello.lists("Nope").await.expect_err("unknown board");
    assert!(err.to_string().contains("boards: \"Roadmap\""), "{err}");
    let err = trello
        .cards(Some("Roadmap"), Some("Later"), false)
        .await
        .expect_err("unknown list");
    assert!(
        err.to_string().contains("lists: \"To do\", \"Done\""),
        "{err}"
    );
}

#[tokio::test]
async fn admin_users_are_found_by_email_across_directories() {
    use atl_sdk::admin::{Admin, EventQuery};
    use wiremock::matchers::query_param_is_missing;
    let ok = |body: serde_json::Value| ResponseTemplate::new(200).set_body_json(body);
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/admin/v1/orgs"))
        .respond_with(ok(
            json!({ "data": [{ "id": "o1", "attributes": { "name": "Acme" } }], "links": {} }),
        ))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/admin/v2/orgs/o1/directories"))
        .respond_with(ok(json!({ "data": [{ "directoryId": "d1", "name": "A" }, { "directoryId": "d2", "name": "B" }] })))
        .mount(&server)
        .await;
    let users = |dir: &str| format!("/admin/v2/orgs/o1/directories/{dir}/users");
    Mock::given(method("GET"))
        .and(path(users("d1")))
        .and(query_param("searchTerm", "ada@acme.com"))
        .respond_with(ok(json!({ "data": [] })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(users("d2")))
        .and(query_param("searchTerm", "ada@acme.com"))
        .respond_with(ok(json!({ "data": [{ "accountId": "u1", "email": "Ada@acme.com", "accountStatus": "active" }] })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/admin/v2/orgs/o1/directories/d2/users/u1/suspend"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(users("d1")))
        .and(query_param_is_missing("searchTerm"))
        .and(query_param_is_missing("cursor"))
        .respond_with(ok(json!({ "data": [{ "accountId": "u2" }], "links": { "next": format!("{}{}?cursor=2", server.uri(), users("d1")) } })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(users("d1")))
        .and(query_param("cursor", "2"))
        .respond_with(ok(json!({ "data": [{ "accountId": "u3" }] })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(users("d2")))
        .and(query_param_is_missing("searchTerm"))
        .respond_with(ok(json!({ "data": [{ "accountId": "u4" }] })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/admin/v1/orgs/o1/events"))
        .and(query_param("q", "added"))
        .respond_with(ok(json!({ "data": [{ "id": "e1", "attributes": { "time": "t", "action": "user_added_to_group", "actor": { "name": "Ada" } }, "message": { "content": "Ada added Bo" } }] })))
        .mount(&server)
        .await;
    let client = client(&server, Family::Admin, Deployment::Cloud);
    let admin = Admin::new(&client);

    let user = admin
        .suspend(Some("acme"), "ada@acme.com", None)
        .await
        .expect("suspend");
    assert_eq!(
        (
            user.account_id.as_str(),
            user.directory.as_str(),
            user.status.as_deref()
        ),
        ("u1", "d2", Some("suspended"))
    );
    let all: Vec<String> = admin
        .users(None, None, 50)
        .await
        .expect("users")
        .into_iter()
        .map(|u| u.account_id)
        .collect();
    assert_eq!(all, ["u2", "u3", "u4"]);
    assert_eq!(admin.users(None, None, 2).await.expect("limited").len(), 2);
    let events = admin
        .events(
            None,
            &EventQuery {
                search: Some("added".into()),
                limit: 50,
                ..Default::default()
            },
        )
        .await
        .expect("events");
    assert_eq!(
        (events[0].actor.as_deref(), events[0].message.as_deref()),
        (Some("Ada"), Some("Ada added Bo"))
    );
    let err = admin
        .directories(Some("Globex"))
        .await
        .expect_err("unknown org");
    assert!(
        err.to_string().contains("organizations: Acme (o1)"),
        "{err}"
    );
}

#[tokio::test]
async fn jira_create_edit_move_and_clone_send_the_new_fields() {
    use atl_sdk::jira::{CloneOptions, TransitionFields};
    let server = MockServer::start().await;
    Mock::given(path("/rest/api/3/myself"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({ "accountId": "me-1", "displayName": "Me" })),
        )
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/rest/api/3/issue"))
        .and(body_partial_json(json!({ "fields": {
            "components": [{ "name": "Backend" }],
            "fixVersions": [{ "name": "1.2" }],
            "versions": [{ "name": "1.1" }],
            "timetracking": { "originalEstimate": "2h" },
            "reporter": { "accountId": "me-1" }
        }})))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({ "key": "A-2" })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/rest/api/3/issue/A-1"))
        .and(query_param("notifyUsers", "false"))
        .and(body_partial_json(json!({
            "fields": { "assignee": { "accountId": "me-1" }, "parent": { "key": "A-9" } },
            "update": {
                "labels": [{ "add": "new" }, { "remove": "old" }],
                "components": [{ "remove": { "name": "Legacy" } }]
            }
        })))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/rest/api/3/issue/A-1/transitions"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({ "transitions": [
                { "id": "31", "name": "Finish", "to": { "name": "Done" } }
            ]})),
        )
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/rest/api/3/issue/A-1/transitions"))
        .and(body_partial_json(json!({ "fields": { "resolution": { "name": "Fixed" }, "assignee": { "accountId": "me-1" } } })))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    let mut source = issue("A-1");
    source["fields"]["project"] = json!({ "key": "A" });
    source["fields"]["labels"] = json!(["x"]);
    source["fields"]["description"] = markup::markdown_to_adf("Fix **v1** now");
    Mock::given(method("GET"))
        .and(path("/rest/api/3/issue/A-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(source))
        .mount(&server)
        .await;
    let mut cloned = markup::markdown_to_adf("Fix **v2** now");
    cloned["version"] = json!(1);
    Mock::given(method("POST"))
        .and(path("/rest/api/3/issue"))
        .and(body_partial_json(json!({ "fields": {
            "summary": "Fix login v2",
            "labels": ["copy"],
            "priority": { "name": "High" },
            "description": cloned
        }})))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({ "key": "A-3" })))
        .expect(1)
        .mount(&server)
        .await;
    let client = client(&server, Family::Jira, Deployment::Cloud);
    let jira = Jira::new(&client);

    let new = NewIssue {
        project: "A".into(),
        issue_type: "Bug".into(),
        summary: "S".into(),
        components: vec!["Backend".into()],
        fix_versions: vec!["1.2".into()],
        affects_versions: vec!["1.1".into()],
        estimate: Some("2h".into()),
        reporter: Some("me".into()),
        ..Default::default()
    };
    assert_eq!(jira.create(&new).await.expect("create").key, "A-2");
    let edit = IssueEdit {
        assignee: Some("me".into()),
        parent: Some("A-9".into()),
        add_labels: vec!["new".into()],
        remove_labels: vec!["old".into()],
        remove_components: vec!["Legacy".into()],
        skip_notify: true,
        ..Default::default()
    };
    jira.edit("A-1", &edit).await.expect("edit");
    let fields = TransitionFields {
        resolution: Some("Fixed".into()),
        assignee: Some("me".into()),
    };
    jira.transition("A-1", "Done", None, &fields)
        .await
        .expect("move");
    let options = CloneOptions {
        priority: Some("High".into()),
        labels: vec!["copy".into()],
        replace: vec![
            ("v1".into(), "v2".into()),
            ("Fix login".into(), "Fix login v2".into()),
        ],
        ..Default::default()
    };
    assert_eq!(
        jira.clone_issue("A-1", &options).await.expect("clone").key,
        "A-3"
    );
}

#[tokio::test]
async fn jira_unlink_remote_links_and_releases() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/rest/api/3/issue/A-1"))
        .and(query_param("fields", "issuelinks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "fields": { "issuelinks": [
            { "id": "100", "type": { "name": "Blocks", "inward": "is blocked by", "outward": "blocks" }, "outwardIssue": { "key": "A-2" } },
            { "id": "101", "type": { "name": "Relates", "inward": "relates to", "outward": "relates to" }, "inwardIssue": { "key": "A-3" } }
        ]}})))
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/rest/api/3/issueLink/100"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/rest/api/3/issue/A-1/remotelink"))
        .and(body_json(
            json!({ "object": { "url": "https://ci/1", "title": "CI" } }),
        ))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({ "id": 7 })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/rest/api/3/project/A/version"))
        .and(query_param("startAt", "0"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "isLast": false, "values": [
            { "id": "1", "name": "1.0", "released": true, "archived": false, "releaseDate": "2026-01-01" }
        ]})))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/rest/api/3/project/A/version"))
        .and(query_param("startAt", "1"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({ "isLast": true, "values": [
                { "id": "2", "name": "1.1", "released": false, "archived": false }
            ]})),
        )
        .mount(&server)
        .await;
    let cloud = client(&server, Family::Jira, Deployment::Cloud);
    let jira = Jira::new(&cloud);

    assert_eq!(
        jira.unlink("A-1", "a-2", None).await.expect("unlink"),
        ["Blocks"]
    );
    let err = jira
        .unlink("A-1", "A-3", Some("blocks"))
        .await
        .expect_err("wrong type");
    assert!(
        err.to_string()
            .contains("A-1 and A-3 are not linked by `blocks`"),
        "{err}"
    );
    let link = jira
        .remote_link("A-1", "https://ci/1", Some("CI"))
        .await
        .expect("remote link");
    assert_eq!(link.id, Some(7));
    let releases = jira.releases("A").await.expect("releases");
    let names: Vec<&str> = releases.iter().map(|r| r.name.as_str()).collect();
    assert_eq!(names, ["1.0", "1.1"]);

    let dc = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/rest/api/2/project/A/versions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            { "id": "9", "name": "2.0", "released": false, "archived": true }
        ])))
        .mount(&dc)
        .await;
    let dc_client = client(&dc, Family::Jira, Deployment::DataCenter);
    let archived = atl_sdk::tasks::run(
        &dc_client,
        "jira_release_list",
        json!({ "project": "A", "state": "archived" }),
    )
    .await
    .expect("dc releases");
    assert_eq!(archived[0]["name"], "2.0");
}

#[tokio::test]
async fn confluence_tree_follows_folders_pages_and_pagination() {
    use wiremock::matchers::query_param_is_missing;
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/pages/10/direct-children"))
        .and(query_param_is_missing("cursor"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": [{ "id": "F", "title": "Folder", "type": "folder" }],
            "_links": { "next": "/wiki/api/v2/pages/10/direct-children?cursor=abc" }
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/pages/10/direct-children"))
        .and(query_param("cursor", "abc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": [{ "id": "P", "title": "Page", "type": "page" }]
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/folders/F/direct-children"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": [{ "id": "Q", "title": "In folder", "type": "page" }]
        })))
        .mount(&server)
        .await;
    for id in ["P", "Q"] {
        Mock::given(method("GET"))
            .and(path(format!("/wiki/api/v2/pages/{id}/direct-children")))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "results": [] })))
            .mount(&server)
            .await;
    }
    let cloud = client(&server, Family::Confluence, Deployment::Cloud);
    let tree = Confluence::new(&cloud)
        .tree(&PageRef::Id("10".into()), None)
        .await
        .expect("tree");
    let rows: Vec<(&str, &str, usize, &str)> = tree
        .iter()
        .map(|n| (n.id.as_str(), n.kind.as_str(), n.depth, n.parent.as_str()))
        .collect();
    assert_eq!(
        rows,
        [
            ("F", "folder", 1, "10"),
            ("Q", "page", 2, "F"),
            ("P", "page", 1, "10")
        ]
    );
    let direct = Confluence::new(&cloud)
        .tree(&PageRef::Id("10".into()), Some(1))
        .await
        .expect("children");
    assert_eq!(direct.len(), 2);

    let dc = MockServer::start().await;
    let page = |i: usize| json!({ "id": i.to_string(), "title": format!("P{i}"), "_links": { "webui": format!("/x/{i}") } });
    Mock::given(method("GET"))
        .and(path("/rest/api/content/1/child/page"))
        .and(query_param("start", "0"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({ "results": (0..200).map(page).collect::<Vec<_>>() })),
        )
        .mount(&dc)
        .await;
    Mock::given(method("GET"))
        .and(path("/rest/api/content/1/child/page"))
        .and(query_param("start", "200"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "results": [page(200)] })))
        .mount(&dc)
        .await;
    let dc_client = client(&dc, Family::Confluence, Deployment::DataCenter);
    let children = Confluence::new(&dc_client)
        .tree(&PageRef::Id("1".into()), Some(1))
        .await
        .expect("dc children");
    assert_eq!(children.len(), 201);
}

#[tokio::test]
async fn confluence_export_and_push_round_trip() {
    use atl_sdk::docs::{ExportOptions, PushOptions, export, push, split_front_matter};
    let server = MockServer::start().await;
    let page = |id: &str, title: &str, parent: &str, version: u64, storage: &str| {
        json!({ "id": id, "title": title, "spaceId": "S", "parentId": parent,
            "version": { "number": version }, "body": { "storage": { "value": storage } },
            "_links": { "webui": format!("/spaces/DOCS/pages/{id}") } })
    };
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/pages/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(page(
            "1",
            "Handbook",
            "100",
            7,
            "<p>Welcome</p><ac:image><ri:attachment ri:filename=\"diagram.png\" /></ac:image>",
        )))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/pages/2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(page(
            "2",
            "Child",
            "1",
            3,
            "<p>Old</p>",
        )))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/spaces/S"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "id": "S", "key": "DOCS" })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/spaces"))
        .and(query_param("keys", "DOCS"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({ "results": [{ "id": "S" }] })),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/pages/1/attachments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "results": [
            { "id": "a1", "title": "diagram.png", "downloadLink": "/download/attachments/1/diagram.png" }
        ]})))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/pages/2/attachments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "results": [] })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/wiki/download/attachments/1/diagram.png"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"PNG".to_vec()))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/pages/1/direct-children"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(
                json!({ "results": [{ "id": "2", "title": "Child", "type": "page" }] }),
            ),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/pages/2/direct-children"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "results": [] })))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/wiki/api/v2/pages/2"))
        .and(wiremock::matchers::body_string_contains("New text"))
        .respond_with(ResponseTemplate::new(200).set_body_json(page(
            "2",
            "Child",
            "1",
            4,
            "<p>New text</p>",
        )))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/wiki/api/v2/pages"))
        .and(body_partial_json(json!({ "title": "New page", "parentId": "1", "spaceId": "S" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "id": "3", "title": "New page", "_links": { "webui": "/spaces/DOCS/pages/3" } })))
        .expect(1)
        .mount(&server)
        .await;
    let cloud = client(&server, Family::Confluence, Deployment::Cloud);
    let confluence = Confluence::new(&cloud);
    let dir = tempfile::tempdir().expect("tempdir");
    let options = ExportOptions {
        recursive: true,
        max_depth: None,
        exclude: Vec::new(),
        attachments: true,
        dry_run: false,
    };
    let rows = export(&confluence, &PageRef::Id("1".into()), dir.path(), &options)
        .await
        .expect("export");
    assert_eq!(rows.len(), 2);
    let root_file = dir.path().join("Handbook/index.md");
    let root_text = std::fs::read_to_string(&root_file).expect("root file");
    let (front, body) = split_front_matter(&root_text, "index.md").expect("front matter");
    assert_eq!(
        (
            front.id.as_deref(),
            front.space.as_deref(),
            front.parent.as_deref(),
            front.version
        ),
        (Some("1"), Some("DOCS"), Some("100"), Some(7))
    );
    assert!(
        body.contains("![diagram.png](<_attachments/diagram.png>)"),
        "{body}"
    );
    assert_eq!(
        std::fs::read(dir.path().join("Handbook/_attachments/diagram.png")).expect("png"),
        b"PNG"
    );
    let child_file = dir.path().join("Handbook/Child/index.md");
    assert!(child_file.is_file());

    let push_options = PushOptions {
        space: None,
        parent: None,
        force: false,
        dry_run: false,
    };
    let pushed = push(&confluence, &[dir.path().to_path_buf()], &push_options)
        .await
        .expect("push unchanged");
    let actions: Vec<&str> = pushed.iter().map(|p| p.action.as_str()).collect();
    assert_eq!(actions, ["unchanged", "unchanged"]);

    let child = std::fs::read_to_string(&child_file).expect("child");
    std::fs::write(&child_file, child.replace("Old", "New text")).expect("edit");
    std::fs::write(dir.path().join("Handbook/New.md"), "# New page\n\nHello\n").expect("new");
    let pushed = push(&confluence, &[dir.path().to_path_buf()], &push_options)
        .await
        .expect("push changes");
    let actions: Vec<(&str, &str)> = pushed
        .iter()
        .map(|p| (p.title.as_str(), p.action.as_str()))
        .collect();
    assert_eq!(
        actions,
        [
            ("Handbook", "unchanged"),
            ("Child", "updated"),
            ("New page", "created")
        ]
    );
    let child_text = std::fs::read_to_string(&child_file).expect("child");
    let (front, _) = split_front_matter(&child_text, "child.md").expect("front matter");
    assert_eq!(front.version, Some(4));
    let new_text = std::fs::read_to_string(dir.path().join("Handbook/New.md")).expect("new");
    assert_eq!(
        split_front_matter(&new_text, "new.md")
            .expect("front matter")
            .0
            .id
            .as_deref(),
        Some("3")
    );

    let stale = dir.path().join("stale.md");
    std::fs::write(&stale, "---\nid: \"2\"\nversion: 1\n---\nx\n").expect("stale");
    let err = push(&confluence, &[stale], &push_options)
        .await
        .expect_err("conflict");
    assert!(
        err.to_string()
            .contains("changed on the server (version 3, the file has 1)"),
        "{err}"
    );
}

#[tokio::test]
async fn bitbucket_comments_checks_branches_commits_drafts_and_stops() {
    use atl_sdk::tasks::run;
    let ok = |body: serde_json::Value| ResponseTemplate::new(200).set_body_json(body);
    let cloud = MockServer::start().await;
    let repo = "/2.0/repositories/acme/web";
    let pr = format!("{repo}/pullrequests/7");
    Mock::given(method("GET"))
        .and(path(format!("{pr}/comments")))
        .respond_with(ok(json!({ "values": [
            { "id": 1, "user": { "display_name": "Ada" }, "created_on": "2026-11-01T10:00:00Z", "content": { "raw": "Looks **good**" } },
            { "id": 2, "user": { "display_name": "Bo" }, "created_on": "2026-11-01T11:00:00Z", "content": { "raw": "Rename this" },
              "inline": { "path": "src/main.rs", "to": 12 }, "parent": { "id": 1 }, "resolution": { "type": "comment_resolution" } },
            { "id": 3, "deleted": true, "content": { "raw": "" } }
        ] })))
        .mount(&cloud)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{pr}/statuses")))
        .respond_with(ok(json!({ "values": [
            { "key": "build", "name": "Build", "state": "SUCCESSFUL", "url": "https://ci/1" },
            { "key": "lint", "name": "Lint", "state": "FAILED", "url": "https://ci/2" }
        ] })))
        .mount(&cloud)
        .await;
    Mock::given(method("DELETE"))
        .and(path(format!("{pr}/approve")))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&cloud)
        .await;
    Mock::given(method("GET"))
        .and(path(pr.clone()))
        .respond_with(ok(json!({ "id": 7, "title": "T", "draft": true })))
        .mount(&cloud)
        .await;
    Mock::given(method("PUT"))
        .and(path(pr.clone()))
        .and(body_json(json!({ "title": "T", "draft": false })))
        .respond_with(ok(json!({ "id": 7, "title": "T", "draft": false })))
        .expect(1)
        .mount(&cloud)
        .await;
    Mock::given(method("GET"))
        .and(path(repo))
        .respond_with(ok(
            json!({ "full_name": "acme/web", "mainbranch": { "name": "main" } }),
        ))
        .mount(&cloud)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{repo}/refs/branches")))
        .and(body_json(
            json!({ "name": "feature/x", "target": { "hash": "main" } }),
        ))
        .respond_with(ok(
            json!({ "name": "feature/x", "target": { "hash": "abc123" } }),
        ))
        .expect(1)
        .mount(&cloud)
        .await;
    Mock::given(method("DELETE"))
        .and(path(format!("{repo}/refs/branches/feature%2Fx")))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&cloud)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{repo}/commits/dev")))
        .respond_with(ok(json!({ "values": [
            { "hash": "abc123", "author": { "raw": "Ada <ada@x>" }, "date": "2026-11-02T00:00:00Z", "message": "Fix login\n\nLonger text" }
        ] })))
        .mount(&cloud)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{repo}/pipelines/9")))
        .respond_with(ok(
            json!({ "uuid": "{p9}", "build_number": 9, "state": { "name": "IN_PROGRESS" } }),
        ))
        .mount(&cloud)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{repo}/pipelines/%7Bp9%7D")))
        .respond_with(ok(json!({ "uuid": "{p9}", "build_number": 9, "state": { "name": "COMPLETED", "result": { "name": "STOPPED" } } })))
        .mount(&cloud)
        .await;
    Mock::given(method("GET"))
        .and(path_regex(r"/pipelines/.*/steps$"))
        .respond_with(ok(json!({ "values": [] })))
        .mount(&cloud)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{repo}/pipelines/%7Bp9%7D/stopPipeline")))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&cloud)
        .await;
    let client = client(&cloud, Family::Bitbucket, Deployment::Cloud);
    let args = json!({ "repo": "acme/web", "id": 7 });

    let comments = run(&client, "bitbucket_pr_comments", args.clone())
        .await
        .expect("comments");
    assert_eq!(comments.as_array().map(Vec::len), Some(2), "{comments}");
    assert_eq!(
        (
            comments[1]["path"].clone(),
            comments[1]["line"].clone(),
            comments[1]["parent"].clone(),
            comments[1]["resolved"].clone()
        ),
        (json!("src/main.rs"), json!(12), json!("1"), json!(true))
    );
    let checks = run(&client, "bitbucket_pr_checks", args.clone())
        .await
        .expect("checks");
    assert_eq!(
        (checks["passed"].clone(), checks["pending"].clone()),
        (json!(false), json!(false))
    );
    assert_eq!(checks["checks"][1]["name"], "Lint");
    run(&client, "bitbucket_pr_unapprove", args.clone())
        .await
        .expect("unapprove");
    let ready = run(&client, "bitbucket_pr_ready", args.clone())
        .await
        .expect("ready");
    assert_eq!(ready["draft"], false);
    let branch = run(
        &client,
        "bitbucket_branch_create",
        json!({ "repo": "acme/web", "name": "feature/x" }),
    )
    .await
    .expect("branch create");
    assert_eq!(branch["commit"], "abc123");
    run(
        &client,
        "bitbucket_branch_delete",
        json!({ "repo": "acme/web", "name": "feature/x" }),
    )
    .await
    .expect("branch delete");
    let commits = run(
        &client,
        "bitbucket_commit_list",
        json!({ "repo": "acme/web", "branch": "dev" }),
    )
    .await
    .expect("commits");
    assert_eq!(
        (commits[0]["message"].clone(), commits[0]["author"].clone()),
        (json!("Fix login"), json!("Ada <ada@x>"))
    );
    let stopped = run(
        &client,
        "bitbucket_pipeline_stop",
        json!({ "repo": "acme/web", "id": "9" }),
    )
    .await
    .expect("stop");
    assert_eq!(stopped["result"], "STOPPED");

    let dc = MockServer::start().await;
    let repo = "/rest/api/latest/projects/PROJ/repos/web";
    let pr = format!("{repo}/pull-requests/7");
    Mock::given(method("GET"))
        .and(path(format!("{pr}/activities")))
        .respond_with(ok(json!({ "isLastPage": true, "values": [
            { "action": "COMMENTED", "commentAnchor": { "path": "a.rs", "line": 3 },
              "comment": { "id": 20, "text": "Inline", "createdDate": 1_700_000_100_000u64, "author": { "displayName": "Bo" },
                "comments": [{ "id": 21, "text": "Reply", "createdDate": 1_700_000_200_000u64, "author": { "displayName": "Ada" },
                  "comments": [{ "id": 22, "text": "Deep", "createdDate": 1_700_000_300_000u64, "state": "RESOLVED" }] }] } },
            { "action": "COMMENTED", "commentAction": "REPLIED", "comment": { "id": 21, "text": "Reply", "createdDate": 1_700_000_200_000u64 } },
            { "action": "APPROVED" },
            { "action": "COMMENTED", "comment": { "id": 10, "text": "General", "createdDate": 1_700_000_000_000u64 } }
        ] })))
        .mount(&dc)
        .await;
    Mock::given(method("GET"))
        .and(path(pr.clone()))
        .respond_with(ok(
            json!({ "id": 7, "fromRef": { "latestCommit": "c0ffee" } }),
        ))
        .mount(&dc)
        .await;
    Mock::given(method("GET"))
        .and(path("/rest/build-status/latest/commits/c0ffee"))
        .respond_with(ok(
            json!({ "values": [{ "key": "ci", "name": "Bamboo", "state": "INPROGRESS" }] }),
        ))
        .mount(&dc)
        .await;
    Mock::given(method("DELETE"))
        .and(path(
            "/rest/branch-utils/latest/projects/PROJ/repos/web/branches",
        ))
        .and(body_json(json!({ "name": "refs/heads/old" })))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&dc)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{repo}/commits")))
        .and(query_param("until", "refs/heads/dev"))
        .respond_with(ok(json!({ "values": [{ "id": "c0ffee", "message": "Ship it", "authorTimestamp": 1_700_000_000_000u64, "author": { "name": "ada" } }] })))
        .mount(&dc)
        .await;
    let client = self::client(&dc, Family::Bitbucket, Deployment::DataCenter);
    let args = json!({ "repo": "PROJ/web", "id": 7 });
    let comments = run(&client, "bitbucket_pr_comments", args.clone())
        .await
        .expect("dc comments");
    let ids: Vec<&str> = comments
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|c| c["id"].as_str())
        .collect();
    assert_eq!(ids, ["10", "20", "21", "22"]);
    assert_eq!(
        (
            comments[3]["parent"].clone(),
            comments[3]["path"].clone(),
            comments[3]["resolved"].clone()
        ),
        (json!("21"), json!("a.rs"), json!(true))
    );
    let checks = run(&client, "bitbucket_pr_checks", args)
        .await
        .expect("dc checks");
    assert_eq!(
        (checks["passed"].clone(), checks["pending"].clone()),
        (json!(true), json!(true))
    );
    run(
        &client,
        "bitbucket_branch_delete",
        json!({ "repo": "PROJ/web", "name": "old" }),
    )
    .await
    .expect("dc branch delete");
    let commits = run(
        &client,
        "bitbucket_commit_list",
        json!({ "repo": "PROJ/web", "branch": "dev" }),
    )
    .await
    .expect("dc commits");
    assert_eq!(
        (commits[0]["hash"].clone(), commits[0]["date"].clone()),
        (json!("c0ffee"), json!("2023-11-14T22:13:20Z"))
    );
}

#[tokio::test]
async fn jira_comments_worklogs_history_links_components_and_sprints() {
    use atl_sdk::tasks::run;
    let ok = |body: serde_json::Value| ResponseTemplate::new(200).set_body_json(body);
    let cloud = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/rest/api/3/issue/AB-1/comment"))
        .respond_with(ok(json!({ "total": 1, "comments": [
            { "id": "10", "author": { "displayName": "Ada" }, "created": "2026-11-01T10:00:00.000+0000",
              "body": { "type": "doc", "version": 1, "content": [{ "type": "paragraph", "content": [{ "type": "text", "text": "Hello" }] }] } }
        ] })))
        .mount(&cloud)
        .await;
    Mock::given(method("PUT"))
        .and(path("/rest/api/3/issue/AB-1/comment/10"))
        .and(body_partial_json(json!({ "body": { "type": "doc" } })))
        .respond_with(ok(json!({ "id": "10" })))
        .expect(1)
        .mount(&cloud)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/rest/api/3/issue/AB-1/comment/10"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&cloud)
        .await;
    Mock::given(method("GET"))
        .and(path("/rest/api/3/issue/AB-1/worklog"))
        .respond_with(ok(json!({ "worklogs": [
            { "id": "5", "author": { "displayName": "Ada" }, "timeSpent": "2h", "timeSpentSeconds": 7200, "started": "2026-11-02T09:00:00.000+0000" }
        ] })))
        .mount(&cloud)
        .await;
    Mock::given(method("GET"))
        .and(path("/rest/api/3/issue/AB-1/changelog"))
        .respond_with(ok(json!({ "isLast": true, "values": [
            { "created": "2026-11-01T10:00:00.000+0000", "author": { "displayName": "Ada" },
              "items": [{ "field": "status", "fromString": "To Do", "toString": "In Progress" }] },
            { "created": "2026-11-03T10:00:00.000+0000", "author": { "displayName": "Bo" },
              "items": [{ "field": "assignee", "fromString": null, "toString": "Bo" }, { "field": "labels", "from": null, "to": "api" }] }
        ] })))
        .mount(&cloud)
        .await;
    Mock::given(method("GET"))
        .and(path("/rest/api/3/issue/AB-1"))
        .respond_with(ok(json!({ "key": "AB-1", "fields": {
            "summary": "S", "status": { "name": "To Do" }, "issuetype": { "name": "Bug" },
            "issuelinks": [
                { "type": { "name": "Blocks", "inward": "is blocked by", "outward": "blocks" }, "outwardIssue": { "key": "AB-2", "fields": { "summary": "Two", "status": { "name": "Done" } } } },
                { "type": { "name": "Blocks", "inward": "is blocked by", "outward": "blocks" }, "inwardIssue": { "key": "AB-3", "fields": { "summary": "Three" } } }
            ] } })))
        .mount(&cloud)
        .await;
    Mock::given(method("GET"))
        .and(path("/rest/api/3/project/AB/components"))
        .respond_with(ok(
            json!([{ "id": "1", "name": "API", "lead": { "displayName": "Ada" } }]),
        ))
        .mount(&cloud)
        .await;
    Mock::given(method("POST"))
        .and(path("/rest/agile/1.0/sprint"))
        .and(body_json(json!({ "name": "Sprint 12", "originBoardId": 3, "endDate": "2026-11-27T00:00:00.000Z" })))
        .respond_with(ok(json!({ "id": 44, "name": "Sprint 12", "state": "future", "endDate": "2026-11-27T00:00:00.000Z" })))
        .expect(1)
        .mount(&cloud)
        .await;
    Mock::given(method("GET"))
        .and(path("/rest/agile/1.0/sprint/44"))
        .respond_with(ok(json!({ "id": 44, "name": "Sprint 12", "state": "future", "startDate": "2026-11-13T00:00:00.000Z", "endDate": "2026-11-27T00:00:00.000Z" })))
        .mount(&cloud)
        .await;
    Mock::given(method("GET"))
        .and(path("/rest/agile/1.0/sprint/45"))
        .respond_with(ok(
            json!({ "id": 45, "name": "No dates", "state": "future" }),
        ))
        .mount(&cloud)
        .await;
    Mock::given(method("POST"))
        .and(path("/rest/agile/1.0/sprint/44"))
        .and(body_json(json!({ "state": "active", "startDate": "2026-11-13T00:00:00.000Z", "endDate": "2026-11-27T00:00:00.000Z" })))
        .respond_with(ok(json!({ "id": 44, "name": "Sprint 12", "state": "active" })))
        .expect(1)
        .mount(&cloud)
        .await;
    Mock::given(method("POST"))
        .and(path("/rest/agile/1.0/sprint/44"))
        .and(body_json(json!({ "state": "closed" })))
        .respond_with(ok(
            json!({ "id": 44, "name": "Sprint 12", "state": "closed" }),
        ))
        .expect(1)
        .mount(&cloud)
        .await;
    let client = client(&cloud, Family::Jira, Deployment::Cloud);

    let comments = run(&client, "jira_comment_list", json!({ "key": "AB-1" }))
        .await
        .expect("comments");
    assert_eq!(
        (comments[0]["id"].clone(), comments[0]["body"].clone()),
        (json!("10"), json!("Hello"))
    );
    run(
        &client,
        "jira_comment_edit",
        json!({ "key": "AB-1", "id": "10", "body": "Fixed **now**" }),
    )
    .await
    .expect("edit");
    run(
        &client,
        "jira_comment_delete",
        json!({ "key": "AB-1", "id": "10" }),
    )
    .await
    .expect("delete");
    let worklogs = run(&client, "jira_worklog_list", json!({ "key": "AB-1" }))
        .await
        .expect("worklogs");
    assert_eq!(
        (
            worklogs[0]["time_spent"].clone(),
            worklogs[0]["seconds"].clone()
        ),
        (json!("2h"), json!(7200))
    );
    let history = run(&client, "jira_issue_history", json!({ "key": "AB-1" }))
        .await
        .expect("history");
    let fields: Vec<&str> = history
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|c| c["field"].as_str())
        .collect();
    assert_eq!(
        fields,
        ["assignee", "labels", "status"],
        "newest first: {history}"
    );
    assert_eq!(
        (history[1]["to"].clone(), history[2]["from"].clone()),
        (json!("api"), json!("To Do"))
    );
    let view = run(
        &client,
        "jira_issue_view",
        json!({ "key": "AB-1", "comments": 0 }),
    )
    .await
    .expect("view");
    assert_eq!(
        view["links"],
        json!([
            { "relation": "blocks", "key": "AB-2", "summary": "Two", "status": "Done" },
            { "relation": "is blocked by", "key": "AB-3", "summary": "Three", "status": null }
        ])
    );
    let components = run(&client, "jira_component_list", json!({ "project": "AB" }))
        .await
        .expect("components");
    assert_eq!(components[0]["lead"], "Ada");
    let sprint = run(
        &client,
        "jira_sprint_create",
        json!({ "name": "Sprint 12", "board": 3, "end": "2026-11-27" }),
    )
    .await
    .expect("sprint create");
    assert_eq!(sprint["id"], 44);
    run(&client, "jira_sprint_start", json!({ "sprint": 44 }))
        .await
        .expect("start");
    let err = run(&client, "jira_sprint_start", json!({ "sprint": 45 }))
        .await
        .expect_err("no end date");
    assert!(err.to_string().contains("pass --end"), "{err}");
    let closed = run(&client, "jira_sprint_close", json!({ "sprint": 44 }))
        .await
        .expect("close");
    assert_eq!(closed["state"], "closed");

    let dc = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/rest/api/2/issue/AB-1"))
        .and(query_param("expand", "changelog"))
        .respond_with(ok(json!({ "key": "AB-1", "changelog": { "histories": [
            { "created": "2026-11-01T10:00:00.000+0000", "author": { "displayName": "Ada" }, "items": [{ "field": "priority", "fromString": "Low", "toString": "High" }] }
        ] } })))
        .mount(&dc)
        .await;
    let history = run(
        &self::client(&dc, Family::Jira, Deployment::DataCenter),
        "jira_issue_history",
        json!({ "key": "AB-1" }),
    )
    .await
    .expect("dc history");
    assert_eq!(
        (history[0]["field"].clone(), history[0]["to"].clone()),
        (json!("priority"), json!("High"))
    );
}

#[tokio::test]
async fn jsm_approvals_participants_slas_and_attachments() {
    use atl_sdk::tasks::run;
    let ok = |body: serde_json::Value| ResponseTemplate::new(200).set_body_json(body);
    let server = MockServer::start().await;
    let base = "/rest/servicedeskapi/request/HELP-1";
    Mock::given(method("GET"))
        .and(path(format!("{base}/approval")))
        .respond_with(ok(json!({ "values": [
            { "id": 7, "name": "Manager", "finalDecision": "pending", "canAnswerApproval": true, "approvers": [{ "approver": { "displayName": "Ada" } }] },
            { "id": 6, "name": "Old", "finalDecision": "approved", "canAnswerApproval": false }
        ] })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{base}/approval/7")))
        .and(body_json(json!({ "decision": "decline" })))
        .respond_with(ok(
            json!({ "id": 7, "name": "Manager", "finalDecision": "declined" }),
        ))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/rest/api/3/myself"))
        .respond_with(ok(json!({ "accountId": "me-1", "displayName": "Me" })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{base}/participant")))
        .and(body_json(json!({ "accountIds": ["me-1"] })))
        .respond_with(ok(json!({})))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{base}/participant")))
        .respond_with(ok(json!({ "values": [{ "displayName": "Me" }] })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{base}/sla")))
        .respond_with(ok(json!({ "values": [
            { "name": "Time to first response", "completedCycles": [{ "breached": false, "elapsedTime": { "friendly": "1h" } }] },
            { "name": "Time to resolution", "ongoingCycle": { "breached": false, "paused": false, "remainingTime": { "friendly": "3h 20m" }, "goalDuration": { "friendly": "8h" } } }
        ] })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(base))
        .respond_with(ok(json!({ "issueKey": "HELP-1", "serviceDeskId": "4" })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/rest/servicedeskapi/servicedesk/4/attachTemporaryFile"))
        .and(wiremock::matchers::header("X-ExperimentalApi", "opt-in"))
        .respond_with(ok(json!({ "temporaryAttachments": [{ "temporaryAttachmentId": "tmp-1", "fileName": "shot.png" }] })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{base}/attachment")))
        .and(body_json(
            json!({ "temporaryAttachmentIds": ["tmp-1"], "public": false }),
        ))
        .respond_with(ok(
            json!({ "attachments": { "values": [{ "filename": "shot.png", "size": 9 }] } }),
        ))
        .expect(1)
        .mount(&server)
        .await;
    let client = client(&server, Family::Jira, Deployment::Cloud);

    let approvals = run(&client, "jsm_request_approvals", json!({ "key": "HELP-1" }))
        .await
        .expect("approvals");
    assert_eq!(
        (
            approvals[0]["id"].clone(),
            approvals[0]["approvers"].clone()
        ),
        (json!("7"), json!(["Ada"]))
    );
    let declined = run(
        &client,
        "jsm_request_approve",
        json!({ "key": "HELP-1", "decision": "decline" }),
    )
    .await
    .expect("decline");
    assert_eq!(declined["decision"], "declined");
    let people = run(
        &client,
        "jsm_request_participants",
        json!({ "key": "HELP-1", "add": ["me"] }),
    )
    .await
    .expect("participants");
    assert_eq!(people["participants"], json!(["Me"]));
    let slas = run(&client, "jsm_request_sla", json!({ "key": "HELP-1" }))
        .await
        .expect("sla");
    assert_eq!(
        (
            slas[0]["state"].clone(),
            slas[1]["state"].clone(),
            slas[1]["remaining"].clone()
        ),
        (json!("met"), json!("ongoing"), json!("3h 20m"))
    );
    let dir = tempfile::tempdir().expect("tempdir");
    let upload = dir.path().join("shot.png");
    std::fs::write(&upload, "png-bytes").expect("write the upload");
    let attached = run(
        &client,
        "jsm_request_attach",
        json!({ "key": "HELP-1", "files": [upload], "internal": true }),
    )
    .await
    .expect("attach");
    assert_eq!(attached[0]["filename"], "shot.png");
}

#[tokio::test]
async fn trello_card_edit_and_checklists() {
    use atl_sdk::tasks::run;
    let ok = |body: serde_json::Value| ResponseTemplate::new(200).set_body_json(body);
    let server = MockServer::start().await;
    let card = "/1/cards/abcdefgh";
    Mock::given(method("PUT"))
        .and(path(card))
        .and(query_param("name", "Ship it"))
        .and(query_param("dueComplete", "true"))
        .respond_with(ok(json!({ "id": "c1" })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(card))
        .and(query_param("fields", "idBoard"))
        .respond_with(ok(json!({ "idBoard": "b1" })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/1/boards/b1/labels"))
        .respond_with(ok(json!([{ "id": "l1", "name": "Bug", "color": "red" }, { "id": "l2", "name": "", "color": "green" }])))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/1/boards/b1/members"))
        .respond_with(ok(
            json!([{ "id": "m1", "username": "ada", "fullName": "Ada Lovelace" }]),
        ))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{card}/idLabels")))
        .and(query_param("value", "l2"))
        .respond_with(ok(json!([])))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path(format!("{card}/idLabels/l1")))
        .respond_with(ok(json!([])))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{card}/idMembers")))
        .and(query_param("value", "m1"))
        .respond_with(ok(json!([])))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(card))
        .and(query_param("list", "true"))
        .respond_with(ok(
            json!({ "id": "c1", "name": "Ship it", "list": { "name": "Doing" } }),
        ))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{card}/checklists")))
        .respond_with(ok(json!([{ "id": "k1", "name": "Release", "checkItems": [
            { "id": "i1", "name": "Tag", "state": "complete" }, { "id": "i2", "name": "Notes", "state": "incomplete" }
        ] }])))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path(format!("{card}/checkItem/i2")))
        .and(query_param("state", "complete"))
        .respond_with(ok(json!({})))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{card}/checklists")))
        .and(query_param("name", "QA"))
        .respond_with(ok(json!({ "id": "k2" })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/1/checklists/k2/checkItems"))
        .respond_with(ok(json!({ "id": "i9" })))
        .expect(2)
        .mount(&server)
        .await;
    let client = client(&server, Family::Trello, Deployment::Cloud);

    let edited = run(
        &client,
        "trello_card_edit",
        json!({
            "card": "abcdefgh", "name": "Ship it", "done": true,
            "add_labels": ["green"], "remove_labels": ["bug"], "add_members": ["Ada Lovelace"]
        }),
    )
    .await
    .expect("edit");
    assert_eq!(edited["list"], "Doing");
    let err = run(
        &client,
        "trello_card_edit",
        json!({ "card": "abcdefgh", "add_labels": ["nope"] }),
    )
    .await
    .expect_err("unknown label");
    assert!(err.to_string().contains("Bug, green"), "{err}");
    let lists = run(
        &client,
        "trello_checklist_list",
        json!({ "card": "abcdefgh" }),
    )
    .await
    .expect("checklists");
    assert_eq!(lists[0]["items"][0]["done"], true);
    let item = run(
        &client,
        "trello_checklist_check",
        json!({ "card": "abcdefgh", "item": "notes" }),
    )
    .await
    .expect("check");
    assert_eq!(
        (item["id"].clone(), item["done"].clone()),
        (json!("i2"), json!(true))
    );
    let created = run(
        &client,
        "trello_checklist_create",
        json!({ "card": "abcdefgh", "name": "QA", "items": ["One", "Two"] }),
    )
    .await
    .expect("create checklist");
    assert_eq!(created["items"].as_array().map(Vec::len), Some(2));
}

#[tokio::test]
async fn admin_group_members_and_confluence_blogs_and_spaces() {
    use atl_sdk::tasks::run;
    let ok = |body: serde_json::Value| ResponseTemplate::new(200).set_body_json(body);
    let admin = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/admin/v1/orgs"))
        .respond_with(ok(
            json!({ "data": [{ "id": "o1", "attributes": { "name": "Acme" } }] }),
        ))
        .mount(&admin)
        .await;
    Mock::given(method("GET"))
        .and(path("/admin/v2/orgs/o1/directories"))
        .respond_with(ok(
            json!({ "data": [{ "directoryId": "d1", "name": "A" }] }),
        ))
        .mount(&admin)
        .await;
    Mock::given(method("GET"))
        .and(path("/admin/v2/orgs/o1/directories/d1/groups"))
        .respond_with(ok(json!({ "data": [{ "id": "g1", "name": "Developers" }, { "id": "g2", "name": "Developers-old" }] })))
        .mount(&admin)
        .await;
    Mock::given(method("GET"))
        .and(path("/admin/v2/orgs/o1/directories/d1/users"))
        .respond_with(ok(
            json!({ "data": [{ "accountId": "u1", "email": "ada@acme.com" }] }),
        ))
        .mount(&admin)
        .await;
    Mock::given(method("POST"))
        .and(path(
            "/admin/v2/orgs/o1/directories/d1/groups/g1/memberships",
        ))
        .and(body_json(json!({ "accountId": "u1" })))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&admin)
        .await;
    Mock::given(method("DELETE"))
        .and(path(
            "/admin/v2/orgs/o1/directories/d1/groups/g1/memberships/u1",
        ))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&admin)
        .await;
    let client = client(&admin, Family::Admin, Deployment::Cloud);
    let added = run(
        &client,
        "admin_group_add_member",
        json!({ "group": "developers", "user": "ada@acme.com" }),
    )
    .await
    .expect("add member");
    assert_eq!(
        (added["group_id"].clone(), added["account_id"].clone()),
        (json!("g1"), json!("u1"))
    );
    run(
        &client,
        "admin_group_remove_member",
        json!({ "group": "g1", "user": "u1" }),
    )
    .await
    .expect("remove member");

    let cloud = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/spaces"))
        .and(query_param("keys", "ENG"))
        .respond_with(ok(json!({ "results": [{ "id": "42", "key": "ENG" }] })))
        .mount(&cloud)
        .await;
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/spaces/42/blogposts"))
        .respond_with(ok(json!({ "results": [{ "id": "9", "title": "Launch", "_links": { "webui": "/spaces/ENG/blog/9" } }] })))
        .mount(&cloud)
        .await;
    Mock::given(method("POST"))
        .and(path("/wiki/api/v2/blogposts"))
        .and(body_partial_json(json!({ "spaceId": "42", "title": "News", "status": "current", "body": { "representation": "storage" } })))
        .respond_with(ok(json!({ "id": "10", "title": "News" })))
        .expect(1)
        .mount(&cloud)
        .await;
    Mock::given(method("POST"))
        .and(path("/wiki/api/v2/spaces"))
        .and(body_json(json!({ "key": "OPS", "name": "Operations", "description": { "value": "Runbooks", "representation": "plain" } })))
        .respond_with(ok(json!({ "id": "77", "key": "OPS", "name": "Operations" })))
        .expect(1)
        .mount(&cloud)
        .await;
    let client = self::client(&cloud, Family::Confluence, Deployment::Cloud);
    let posts = run(&client, "confluence_blog_list", json!({ "space": "ENG" }))
        .await
        .expect("blogs");
    assert_eq!(posts[0]["title"], "Launch");
    run(
        &client,
        "confluence_blog_create",
        json!({ "space": "ENG", "title": "News", "markdown": "# Hi" }),
    )
    .await
    .expect("blog create");
    let space = run(
        &client,
        "confluence_space_create",
        json!({ "key": "OPS", "name": "Operations", "description": "Runbooks" }),
    )
    .await
    .expect("space create");
    assert_eq!(space["id"], "77");

    let dc = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/rest/api/content"))
        .and(body_partial_json(
            json!({ "type": "blogpost", "space": { "key": "ENG" }, "title": "News" }),
        ))
        .respond_with(ok(json!({ "id": "11", "title": "News" })))
        .expect(1)
        .mount(&dc)
        .await;
    Mock::given(method("GET"))
        .and(path("/rest/api/content"))
        .and(query_param("type", "blogpost"))
        .and(query_param("spaceKey", "ENG"))
        .respond_with(ok(json!({ "results": [{ "id": "11", "title": "News" }] })))
        .mount(&dc)
        .await;
    let client = self::client(&dc, Family::Confluence, Deployment::DataCenter);
    run(
        &client,
        "confluence_blog_create",
        json!({ "space": "ENG", "title": "News", "markdown": "Hi" }),
    )
    .await
    .expect("dc blog create");
    let posts = run(&client, "confluence_blog_list", json!({ "space": "ENG" }))
        .await
        .expect("dc blogs");
    assert_eq!(posts[0]["id"], "11");
}

#[tokio::test]
async fn an_ambiguous_reporter_asks_for_the_reporter() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/rest/api/3/user/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            { "accountId": "a1", "displayName": "Ann Lee" },
            { "accountId": "a2", "displayName": "Ann Kim" }
        ])))
        .mount(&server)
        .await;
    let client = client(&server, Family::Jira, Deployment::Cloud);
    let err = atl_sdk::tasks::run(
        &client,
        "jira_issue_create",
        json!({ "project": "AB", "summary": "S", "reporter": "ann" }),
    )
    .await
    .expect_err("two users match");
    let json = err.to_json();
    assert_eq!(json["code"], "input_required", "{json}");
    assert_eq!(json["input"]["arg"], "reporter", "{json}");
    assert_eq!(json["input"]["choices"].as_array().map(Vec::len), Some(2));
}

#[tokio::test]
async fn page_and_repository_references_are_checked_before_the_task_runs() {
    let server = MockServer::start().await;
    let confluence = client(&server, Family::Confluence, Deployment::Cloud);
    let err = atl_sdk::tasks::run(
        &confluence,
        "confluence_page_get",
        json!({ "page": "not a page" }),
    )
    .await
    .expect_err("bad page");
    assert_eq!(err.to_json()["code"], "invalid");
    assert!(err.to_string().contains("is not a page reference"), "{err}");
    let bitbucket = client(&server, Family::Bitbucket, Deployment::Cloud);
    let err = atl_sdk::tasks::run(
        &bitbucket,
        "bitbucket_pr_list",
        json!({ "repo": "no-slash" }),
    )
    .await
    .expect_err("bad repo");
    assert!(err.to_string().contains("pass WORKSPACE/REPO"), "{err}");
    assert!(
        server
            .received_requests()
            .await
            .unwrap_or_default()
            .is_empty(),
        "nothing was sent"
    );
}
