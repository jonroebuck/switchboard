use std::sync::Arc;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use chrono::Utc;
use http_body_util::BodyExt;
use serde_json::{json, Value};
use switchboard_core::artifacts::*;
use switchboard_core::channels::*;
use switchboard_core::issues::*;
use tokio::sync::Mutex;
use tower::ServiceExt;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Mock stores
// ---------------------------------------------------------------------------

struct MockChannelStore {
    channels: Mutex<Vec<Channel>>,
    threads: Mutex<Vec<Thread>>,
    posts: Mutex<Vec<Post>>,
}

impl MockChannelStore {
    fn new() -> Self {
        Self {
            channels: Mutex::new(Vec::new()),
            threads: Mutex::new(Vec::new()),
            posts: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl ChannelStore for MockChannelStore {
    async fn list_channels(&self) -> Result<Vec<Channel>, ChannelError> {
        Ok(self.channels.lock().await.clone())
    }

    async fn get_channel(&self, id: Uuid) -> Result<Channel, ChannelError> {
        self.channels
            .lock()
            .await
            .iter()
            .find(|c| c.id == id)
            .cloned()
            .ok_or(ChannelError::NotFound(id))
    }

    async fn create_channel(&self, name: String, description: String) -> Result<Channel, ChannelError> {
        let channel = Channel {
            id: Uuid::new_v4(),
            name,
            description,
            created_at: Utc::now(),
        };
        self.channels.lock().await.push(channel.clone());
        Ok(channel)
    }

    async fn list_threads(&self, channel_id: Uuid) -> Result<Vec<Thread>, ChannelError> {
        Ok(self.threads.lock().await.iter().filter(|t| t.channel_id == channel_id).cloned().collect())
    }

    async fn get_thread(&self, id: Uuid) -> Result<Thread, ChannelError> {
        self.threads.lock().await.iter().find(|t| t.id == id).cloned().ok_or(ChannelError::NotFound(id))
    }

    async fn create_thread(&self, channel_id: Uuid, title: String, author: String) -> Result<Thread, ChannelError> {
        let thread = Thread {
            id: Uuid::new_v4(),
            channel_id,
            title,
            author,
            created_at: Utc::now(),
        };
        self.threads.lock().await.push(thread.clone());
        Ok(thread)
    }

    async fn list_posts(&self, thread_id: Uuid) -> Result<Vec<Post>, ChannelError> {
        Ok(self.posts.lock().await.iter().filter(|p| p.thread_id == thread_id).cloned().collect())
    }

    async fn create_post(&self, thread_id: Uuid, author: String, content: String) -> Result<Post, ChannelError> {
        let post = Post {
            id: Uuid::new_v4(),
            thread_id,
            author,
            content,
            created_at: Utc::now(),
        };
        self.posts.lock().await.push(post.clone());
        Ok(post)
    }
}

struct MockIssueStore {
    issues: Mutex<Vec<Issue>>,
    events: Mutex<Vec<IssueEvent>>,
}

impl MockIssueStore {
    fn new() -> Self {
        Self {
            issues: Mutex::new(Vec::new()),
            events: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl IssueStore for MockIssueStore {
    async fn list_issues(&self) -> Result<Vec<Issue>, IssueError> {
        Ok(self.issues.lock().await.clone())
    }

    async fn get_issue(&self, id: Uuid) -> Result<Issue, IssueError> {
        self.issues.lock().await.iter().find(|i| i.id == id).cloned().ok_or(IssueError::NotFound(id))
    }

    async fn create_issue(&self, title: String, description: String) -> Result<Issue, IssueError> {
        let now = Utc::now();
        let issue = Issue {
            id: Uuid::new_v4(),
            title,
            description,
            status: "backlog".into(),
            assignee: None,
            created_at: now,
            updated_at: now,
        };
        self.issues.lock().await.push(issue.clone());
        Ok(issue)
    }

    async fn update_issue_status(&self, id: Uuid, status: String, note: Option<String>) -> Result<IssueEvent, IssueError> {
        let mut issues = self.issues.lock().await;
        let issue = issues.iter_mut().find(|i| i.id == id).ok_or(IssueError::NotFound(id))?;
        let from_status = issue.status.clone();
        issue.status = status.clone();
        issue.updated_at = Utc::now();
        let event = IssueEvent {
            id: Uuid::new_v4(),
            issue_id: id,
            from_status,
            to_status: status,
            note,
            timestamp: Utc::now(),
        };
        drop(issues);
        self.events.lock().await.push(event.clone());
        Ok(event)
    }

    async fn list_events(&self, issue_id: Uuid) -> Result<Vec<IssueEvent>, IssueError> {
        Ok(self.events.lock().await.iter().filter(|e| e.issue_id == issue_id).cloned().collect())
    }
}

struct MockArtifactStore {
    artifacts: Mutex<Vec<Artifact>>,
}

impl MockArtifactStore {
    fn new() -> Self {
        Self {
            artifacts: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl ArtifactStore for MockArtifactStore {
    async fn list_artifacts(&self) -> Result<Vec<Artifact>, ArtifactError> {
        Ok(self.artifacts.lock().await.clone())
    }

    async fn get_artifact(&self, id: Uuid) -> Result<Artifact, ArtifactError> {
        self.artifacts.lock().await.iter().find(|a| a.id == id).cloned().ok_or(ArtifactError::NotFound(id))
    }

    async fn create_artifact(
        &self,
        name: String,
        version: String,
        source_type: String,
        source_location: String,
        content_type: String,
    ) -> Result<Artifact, ArtifactError> {
        let artifact = Artifact {
            id: Uuid::new_v4(),
            name,
            version,
            source_type,
            source_location,
            content_type,
            created_at: Utc::now(),
        };
        self.artifacts.lock().await.push(artifact.clone());
        Ok(artifact)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_app() -> Router {
    let channels = Arc::new(MockChannelStore::new());
    let issues = Arc::new(MockIssueStore::new());
    let artifacts = Arc::new(MockArtifactStore::new());

    Router::new().nest(
        "/api/v1",
        Router::new()
            .merge(switchboard_server::routes::channels::router(channels))
            .merge(switchboard_server::routes::issues::router(issues))
            .merge(switchboard_server::routes::artifacts::router(artifacts)),
    )
}

async fn get(app: Router, uri: &str) -> (StatusCode, Value) {
    let resp = app
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = resp.status();
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&body).unwrap())
}

async fn post(app: Router, uri: &str, body: Value) -> (StatusCode, Value) {
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(uri)
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = resp.status();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&bytes).unwrap())
}

async fn put(app: Router, uri: &str, body: Value) -> (StatusCode, Value) {
    let resp = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(uri)
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = resp.status();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&bytes).unwrap())
}

// ===========================================================================
// Channel route tests
// ===========================================================================

#[tokio::test]
async fn channels_list_empty() {
    let app = build_app();
    let (status, body) = get(app, "/api/v1/channels").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn channels_create_and_get() {
    let app = build_app();

    let (status, created) = post(
        app.clone(),
        "/api/v1/channels",
        json!({"name": "general", "description": "General chat"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(created["name"], "general");
    assert_eq!(created["description"], "General chat");
    assert!(created["id"].is_string());
    assert!(created["created_at"].is_string());

    let id = created["id"].as_str().unwrap();
    let (status, fetched) = get(app, &format!("/api/v1/channels/{id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(fetched["id"], id);
    assert_eq!(fetched["name"], "general");
}

#[tokio::test]
async fn channels_list_after_create() {
    let app = build_app();
    post(app.clone(), "/api/v1/channels", json!({"name": "a", "description": "A"})).await;
    post(app.clone(), "/api/v1/channels", json!({"name": "b", "description": "B"})).await;

    let (status, body) = get(app, "/api/v1/channels").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn threads_create_and_list() {
    let app = build_app();
    let (_, channel) = post(app.clone(), "/api/v1/channels", json!({"name": "ch", "description": "d"})).await;
    let ch_id = channel["id"].as_str().unwrap();

    let (status, thread) = post(
        app.clone(),
        &format!("/api/v1/channels/{ch_id}/threads"),
        json!({"title": "Topic", "author": "alice"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(thread["title"], "Topic");
    assert_eq!(thread["author"], "alice");
    assert_eq!(thread["channel_id"], ch_id);

    let (status, threads) = get(app, &format!("/api/v1/channels/{ch_id}/threads")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(threads.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn thread_get() {
    let app = build_app();
    let (_, channel) = post(app.clone(), "/api/v1/channels", json!({"name": "ch", "description": "d"})).await;
    let ch_id = channel["id"].as_str().unwrap();
    let (_, thread) = post(
        app.clone(),
        &format!("/api/v1/channels/{ch_id}/threads"),
        json!({"title": "T", "author": "bob"}),
    )
    .await;
    let t_id = thread["id"].as_str().unwrap();

    let (status, fetched) = get(app, &format!("/api/v1/threads/{t_id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(fetched["id"], t_id);
    assert_eq!(fetched["title"], "T");
}

#[tokio::test]
async fn posts_create_and_list() {
    let app = build_app();
    let (_, channel) = post(app.clone(), "/api/v1/channels", json!({"name": "ch", "description": "d"})).await;
    let ch_id = channel["id"].as_str().unwrap();
    let (_, thread) = post(
        app.clone(),
        &format!("/api/v1/channels/{ch_id}/threads"),
        json!({"title": "T", "author": "alice"}),
    )
    .await;
    let t_id = thread["id"].as_str().unwrap();

    let (status, p) = post(
        app.clone(),
        &format!("/api/v1/threads/{t_id}/posts"),
        json!({"author": "bob", "content": "Hello!"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(p["author"], "bob");
    assert_eq!(p["content"], "Hello!");
    assert_eq!(p["thread_id"], t_id);

    let (status, posts) = get(app, &format!("/api/v1/threads/{t_id}/posts")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(posts.as_array().unwrap().len(), 1);
}

// ===========================================================================
// Issue route tests
// ===========================================================================

#[tokio::test]
async fn issues_list_empty() {
    let app = build_app();
    let (status, body) = get(app, "/api/v1/issues").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn issues_create_and_get() {
    let app = build_app();

    let (status, issue) = post(
        app.clone(),
        "/api/v1/issues",
        json!({"title": "Bug", "description": "Something broke"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(issue["title"], "Bug");
    assert_eq!(issue["status"], "backlog");
    assert!(issue["assignee"].is_null());
    assert!(issue["id"].is_string());
    assert!(issue["created_at"].is_string());
    assert!(issue["updated_at"].is_string());

    let id = issue["id"].as_str().unwrap();
    let (status, fetched) = get(app, &format!("/api/v1/issues/{id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(fetched["id"], id);
    assert_eq!(fetched["title"], "Bug");
}

#[tokio::test]
async fn issues_list_after_create() {
    let app = build_app();
    post(app.clone(), "/api/v1/issues", json!({"title": "A", "description": "a"})).await;
    post(app.clone(), "/api/v1/issues", json!({"title": "B", "description": "b"})).await;

    let (status, body) = get(app, "/api/v1/issues").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn issues_status_update() {
    let app = build_app();
    let (_, issue) = post(app.clone(), "/api/v1/issues", json!({"title": "T", "description": "d"})).await;
    let id = issue["id"].as_str().unwrap();

    let (status, event) = put(
        app.clone(),
        &format!("/api/v1/issues/{id}/status"),
        json!({"status": "in_progress", "note": "Starting work"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(event["from_status"], "backlog");
    assert_eq!(event["to_status"], "in_progress");
    assert_eq!(event["note"], "Starting work");
    assert_eq!(event["issue_id"], id);
}

#[tokio::test]
async fn issues_list_events() {
    let app = build_app();
    let (_, issue) = post(app.clone(), "/api/v1/issues", json!({"title": "T", "description": "d"})).await;
    let id = issue["id"].as_str().unwrap();
    put(app.clone(), &format!("/api/v1/issues/{id}/status"), json!({"status": "in_progress"})).await;
    put(app.clone(), &format!("/api/v1/issues/{id}/status"), json!({"status": "done"})).await;

    let (status, events) = get(app, &format!("/api/v1/issues/{id}/events")).await;
    assert_eq!(status, StatusCode::OK);
    let events = events.as_array().unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0]["to_status"], "in_progress");
    assert_eq!(events[1]["to_status"], "done");
}

// ===========================================================================
// Artifact route tests
// ===========================================================================

#[tokio::test]
async fn artifacts_list_empty() {
    let app = build_app();
    let (status, body) = get(app, "/api/v1/artifacts").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn artifacts_create_and_get() {
    let app = build_app();

    let (status, artifact) = post(
        app.clone(),
        "/api/v1/artifacts",
        json!({
            "name": "doc.pdf",
            "version": "1",
            "source_type": "upload",
            "source_location": "/files/doc.pdf",
            "content_type": "application/pdf"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(artifact["name"], "doc.pdf");
    assert_eq!(artifact["version"], "1");
    assert_eq!(artifact["source_type"], "upload");
    assert_eq!(artifact["content_type"], "application/pdf");
    assert!(artifact["id"].is_string());
    assert!(artifact["created_at"].is_string());

    let id = artifact["id"].as_str().unwrap();
    let (status, fetched) = get(app, &format!("/api/v1/artifacts/{id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(fetched["id"], id);
    assert_eq!(fetched["name"], "doc.pdf");
}

#[tokio::test]
async fn artifacts_list_after_create() {
    let app = build_app();
    post(
        app.clone(),
        "/api/v1/artifacts",
        json!({"name": "a", "version": "1", "source_type": "s", "source_location": "l", "content_type": "c"}),
    )
    .await;
    post(
        app.clone(),
        "/api/v1/artifacts",
        json!({"name": "b", "version": "1", "source_type": "s", "source_location": "l", "content_type": "c"}),
    )
    .await;

    let (status, body) = get(app, "/api/v1/artifacts").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 2);
}
