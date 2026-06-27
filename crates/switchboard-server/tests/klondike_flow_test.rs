use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use chrono::Utc;
use http_body_util::BodyExt;
use klondike_channels::KlondikeChannelStore;
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

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

#[tokio::test]
async fn test_channel_thread_post_flow() {
    let klondike = MockServer::start().await;
    let now = Utc::now();

    let channel_id = Uuid::new_v4();
    let thread_id = Uuid::new_v4();
    let post_id = Uuid::new_v4();

    Mock::given(method("POST"))
        .and(path("/api/v1/channels"))
        .and(body_json(json!({"name": "test", "description": ""})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": channel_id,
            "name": "test",
            "description": "",
            "created_at": now
        })))
        .expect(1)
        .mount(&klondike)
        .await;

    Mock::given(method("POST"))
        .and(path(format!("/api/v1/channels/{channel_id}/threads")))
        .and(body_json(json!({"title": "test-thread", "author": "ferb"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": thread_id,
            "channel_id": channel_id,
            "title": "test-thread",
            "author": "ferb",
            "created_at": now
        })))
        .expect(1)
        .mount(&klondike)
        .await;

    Mock::given(method("POST"))
        .and(path(format!("/api/v1/threads/{thread_id}/posts")))
        .and(body_json(json!({"author": "ferb", "content": "hello"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": post_id,
            "thread_id": thread_id,
            "author": "ferb",
            "content": "hello",
            "created_at": now
        })))
        .expect(1)
        .mount(&klondike)
        .await;

    let channel_store = Arc::new(KlondikeChannelStore::new(&klondike.uri()));
    let app = Router::new().nest(
        "/api/v1",
        switchboard_server::routes::channels::router(channel_store),
    );

    // Step 1: POST /api/v1/channels
    let (status, channel) = post(
        app.clone(),
        "/api/v1/channels",
        json!({"name": "test", "description": ""}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let returned_channel_id = channel["id"].as_str().unwrap();
    assert_eq!(returned_channel_id, channel_id.to_string());

    // Step 2: POST /api/v1/channels/{id}/threads
    let (status, thread) = post(
        app.clone(),
        &format!("/api/v1/channels/{returned_channel_id}/threads"),
        json!({"title": "test-thread", "author": "ferb"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let returned_thread_id = thread["id"].as_str().unwrap();
    assert_eq!(returned_thread_id, thread_id.to_string());

    // Step 3: POST /api/v1/threads/{id}/posts
    let (status, post_resp) = post(
        app.clone(),
        &format!("/api/v1/threads/{returned_thread_id}/posts"),
        json!({"author": "ferb", "content": "hello"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let returned_post_id = post_resp["id"].as_str().unwrap();
    assert_eq!(returned_post_id, post_id.to_string());
}
