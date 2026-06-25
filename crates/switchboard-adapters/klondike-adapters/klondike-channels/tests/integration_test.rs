use chrono::Utc;
use klondike_channels::KlondikeChannelStore;
use serde_json::json;
use switchboard_core::channels::ChannelStore;
use uuid::Uuid;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn create_channel() {
    let server = MockServer::start().await;
    let id = Uuid::new_v4();
    let now = Utc::now();

    Mock::given(method("POST"))
        .and(path("/api/v1/channels"))
        .and(body_json(json!({"name": "general", "description": "General chat"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": id,
            "name": "general",
            "description": "General chat",
            "created_at": now
        })))
        .expect(1)
        .mount(&server)
        .await;

    let store = KlondikeChannelStore::new(&server.uri());
    let channel = store
        .create_channel("general".into(), "General chat".into())
        .await
        .unwrap();
    assert_eq!(channel.id, id);
    assert_eq!(channel.name, "general");
    assert_eq!(channel.description, "General chat");
}

#[tokio::test]
async fn get_channel() {
    let server = MockServer::start().await;
    let id = Uuid::new_v4();
    let now = Utc::now();

    Mock::given(method("GET"))
        .and(path(format!("/api/v1/channels/{id}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": id,
            "name": "general",
            "description": "desc",
            "created_at": now
        })))
        .expect(1)
        .mount(&server)
        .await;

    let store = KlondikeChannelStore::new(&server.uri());
    let channel = store.get_channel(id).await.unwrap();
    assert_eq!(channel.id, id);
    assert_eq!(channel.name, "general");
}

#[tokio::test]
async fn list_channels() {
    let server = MockServer::start().await;
    let now = Utc::now();

    Mock::given(method("GET"))
        .and(path("/api/v1/channels"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {"id": Uuid::new_v4(), "name": "a", "description": "A", "created_at": now},
            {"id": Uuid::new_v4(), "name": "b", "description": "B", "created_at": now}
        ])))
        .expect(1)
        .mount(&server)
        .await;

    let store = KlondikeChannelStore::new(&server.uri());
    let channels = store.list_channels().await.unwrap();
    assert_eq!(channels.len(), 2);
    assert_eq!(channels[0].name, "a");
    assert_eq!(channels[1].name, "b");
}

#[tokio::test]
async fn create_thread() {
    let server = MockServer::start().await;
    let channel_id = Uuid::new_v4();
    let thread_id = Uuid::new_v4();
    let now = Utc::now();

    Mock::given(method("POST"))
        .and(path(format!("/api/v1/channels/{channel_id}/threads")))
        .and(body_json(json!({"title": "Topic", "author": "alice"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": thread_id,
            "channel_id": channel_id,
            "title": "Topic",
            "author": "alice",
            "created_at": now
        })))
        .expect(1)
        .mount(&server)
        .await;

    let store = KlondikeChannelStore::new(&server.uri());
    let thread = store
        .create_thread(channel_id, "Topic".into(), "alice".into())
        .await
        .unwrap();
    assert_eq!(thread.id, thread_id);
    assert_eq!(thread.channel_id, channel_id);
    assert_eq!(thread.title, "Topic");
    assert_eq!(thread.author, "alice");
}

#[tokio::test]
async fn list_threads() {
    let server = MockServer::start().await;
    let channel_id = Uuid::new_v4();
    let now = Utc::now();

    Mock::given(method("GET"))
        .and(path(format!("/api/v1/channels/{channel_id}/threads")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([{
            "id": Uuid::new_v4(),
            "channel_id": channel_id,
            "title": "Thread 1",
            "author": "alice",
            "created_at": now
        }])))
        .expect(1)
        .mount(&server)
        .await;

    let store = KlondikeChannelStore::new(&server.uri());
    let threads = store.list_threads(channel_id).await.unwrap();
    assert_eq!(threads.len(), 1);
    assert_eq!(threads[0].title, "Thread 1");
}

#[tokio::test]
async fn create_post() {
    let server = MockServer::start().await;
    let thread_id = Uuid::new_v4();
    let post_id = Uuid::new_v4();
    let now = Utc::now();

    Mock::given(method("POST"))
        .and(path(format!("/api/v1/threads/{thread_id}/posts")))
        .and(body_json(json!({"author": "bob", "content": "Hello!"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": post_id,
            "thread_id": thread_id,
            "author": "bob",
            "content": "Hello!",
            "created_at": now
        })))
        .expect(1)
        .mount(&server)
        .await;

    let store = KlondikeChannelStore::new(&server.uri());
    let post = store
        .create_post(thread_id, "bob".into(), "Hello!".into())
        .await
        .unwrap();
    assert_eq!(post.id, post_id);
    assert_eq!(post.thread_id, thread_id);
    assert_eq!(post.author, "bob");
    assert_eq!(post.content, "Hello!");
}

#[tokio::test]
async fn list_posts() {
    let server = MockServer::start().await;
    let thread_id = Uuid::new_v4();
    let now = Utc::now();

    Mock::given(method("GET"))
        .and(path(format!("/api/v1/threads/{thread_id}/posts")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "id": Uuid::new_v4(),
                "thread_id": thread_id,
                "author": "bob",
                "content": "first",
                "created_at": now
            },
            {
                "id": Uuid::new_v4(),
                "thread_id": thread_id,
                "author": "alice",
                "content": "second",
                "created_at": now
            }
        ])))
        .expect(1)
        .mount(&server)
        .await;

    let store = KlondikeChannelStore::new(&server.uri());
    let posts = store.list_posts(thread_id).await.unwrap();
    assert_eq!(posts.len(), 2);
    assert_eq!(posts[0].author, "bob");
    assert_eq!(posts[1].author, "alice");
}
