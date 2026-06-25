use chrono::Utc;
use klondike_issues::KlondikeIssueStore;
use serde_json::json;
use switchboard_core::issues::IssueStore;
use uuid::Uuid;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn create_issue() {
    let server = MockServer::start().await;
    let id = Uuid::new_v4();
    let now = Utc::now();

    Mock::given(method("POST"))
        .and(path("/api/v1/issues"))
        .and(body_json(json!({"title": "Bug", "description": "Something broke"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": id,
            "title": "Bug",
            "description": "Something broke",
            "status": "backlog",
            "assignee": null,
            "created_at": now,
            "updated_at": now
        })))
        .expect(1)
        .mount(&server)
        .await;

    let store = KlondikeIssueStore::new(&server.uri());
    let issue = store
        .create_issue("Bug".into(), "Something broke".into())
        .await
        .unwrap();
    assert_eq!(issue.id, id);
    assert_eq!(issue.title, "Bug");
    assert_eq!(issue.status, "backlog");
    assert!(issue.assignee.is_none());
}

#[tokio::test]
async fn get_issue() {
    let server = MockServer::start().await;
    let id = Uuid::new_v4();
    let now = Utc::now();

    Mock::given(method("GET"))
        .and(path(format!("/api/v1/issues/{id}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": id,
            "title": "Bug",
            "description": "desc",
            "status": "in_progress",
            "assignee": "alice",
            "created_at": now,
            "updated_at": now
        })))
        .expect(1)
        .mount(&server)
        .await;

    let store = KlondikeIssueStore::new(&server.uri());
    let issue = store.get_issue(id).await.unwrap();
    assert_eq!(issue.id, id);
    assert_eq!(issue.title, "Bug");
    assert_eq!(issue.status, "in_progress");
    assert_eq!(issue.assignee.as_deref(), Some("alice"));
}

#[tokio::test]
async fn list_issues() {
    let server = MockServer::start().await;
    let now = Utc::now();

    Mock::given(method("GET"))
        .and(path("/api/v1/issues"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "id": Uuid::new_v4(),
                "title": "A",
                "description": "a",
                "status": "backlog",
                "assignee": null,
                "created_at": now,
                "updated_at": now
            },
            {
                "id": Uuid::new_v4(),
                "title": "B",
                "description": "b",
                "status": "done",
                "assignee": "bob",
                "created_at": now,
                "updated_at": now
            }
        ])))
        .expect(1)
        .mount(&server)
        .await;

    let store = KlondikeIssueStore::new(&server.uri());
    let issues = store.list_issues().await.unwrap();
    assert_eq!(issues.len(), 2);
    assert_eq!(issues[0].title, "A");
    assert_eq!(issues[1].title, "B");
}

#[tokio::test]
async fn status_transitions() {
    let server = MockServer::start().await;
    let issue_id = Uuid::new_v4();
    let now = Utc::now();

    Mock::given(method("PUT"))
        .and(path(format!("/api/v1/issues/{issue_id}/status")))
        .and(body_json(json!({"status": "in_progress", "note": null})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": Uuid::new_v4(),
            "issue_id": issue_id,
            "from_status": "backlog",
            "to_status": "in_progress",
            "note": null,
            "timestamp": now
        })))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("PUT"))
        .and(path(format!("/api/v1/issues/{issue_id}/status")))
        .and(body_json(json!({"status": "done", "note": null})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": Uuid::new_v4(),
            "issue_id": issue_id,
            "from_status": "in_progress",
            "to_status": "done",
            "note": null,
            "timestamp": now
        })))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("PUT"))
        .and(path(format!("/api/v1/issues/{issue_id}/status")))
        .and(body_json(json!({"status": "blocked", "note": "Dependency"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": Uuid::new_v4(),
            "issue_id": issue_id,
            "from_status": "done",
            "to_status": "blocked",
            "note": "Dependency",
            "timestamp": now
        })))
        .expect(1)
        .mount(&server)
        .await;

    let store = KlondikeIssueStore::new(&server.uri());

    let ev1 = store
        .update_issue_status(issue_id, "in_progress".into(), None)
        .await
        .unwrap();
    assert_eq!(ev1.from_status, "backlog");
    assert_eq!(ev1.to_status, "in_progress");
    assert!(ev1.note.is_none());

    let ev2 = store
        .update_issue_status(issue_id, "done".into(), None)
        .await
        .unwrap();
    assert_eq!(ev2.from_status, "in_progress");
    assert_eq!(ev2.to_status, "done");

    let ev3 = store
        .update_issue_status(issue_id, "blocked".into(), Some("Dependency".into()))
        .await
        .unwrap();
    assert_eq!(ev3.from_status, "done");
    assert_eq!(ev3.to_status, "blocked");
    assert_eq!(ev3.note.as_deref(), Some("Dependency"));
}

#[tokio::test]
async fn list_events() {
    let server = MockServer::start().await;
    let issue_id = Uuid::new_v4();
    let now = Utc::now();

    Mock::given(method("GET"))
        .and(path(format!("/api/v1/issues/{issue_id}/events")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "id": Uuid::new_v4(),
                "issue_id": issue_id,
                "from_status": "backlog",
                "to_status": "in_progress",
                "note": null,
                "timestamp": now
            },
            {
                "id": Uuid::new_v4(),
                "issue_id": issue_id,
                "from_status": "in_progress",
                "to_status": "done",
                "note": "Complete",
                "timestamp": now
            }
        ])))
        .expect(1)
        .mount(&server)
        .await;

    let store = KlondikeIssueStore::new(&server.uri());
    let events = store.list_events(issue_id).await.unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].to_status, "in_progress");
    assert_eq!(events[1].to_status, "done");
    assert_eq!(events[1].note.as_deref(), Some("Complete"));
}
