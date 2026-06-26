use std::sync::Arc;

use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use switchboard_core::channels::ChannelStore;
use switchboard_core::schema::CreateSchema;
use uuid::Uuid;

pub fn router<S: ChannelStore + 'static>(store: Arc<S>) -> Router {
    Router::new()
        .route("/schema/channels", get(channel_schema::<S>))
        .route("/channels", get(list_channels::<S>).post(create_channel::<S>))
        .route("/channels/{id}", get(get_channel::<S>))
        .route("/channels/{channel_id}/threads", get(list_threads::<S>).post(create_thread::<S>))
        .route("/threads/{id}", get(get_thread::<S>))
        .route("/threads/{thread_id}/posts", get(list_posts::<S>).post(create_post::<S>))
        .with_state(store)
}

async fn channel_schema<S: ChannelStore + 'static>(
    State(store): State<Arc<S>>,
) -> Json<CreateSchema> {
    Json(store.create_channel_schema())
}

async fn list_channels<S: ChannelStore + 'static>(
    State(store): State<Arc<S>>,
) -> Result<Json<Vec<switchboard_core::channels::Channel>>, String> {
    store.list_channels().await.map(Json).map_err(|e| e.to_string())
}

async fn get_channel<S: ChannelStore + 'static>(
    State(store): State<Arc<S>>,
    Path(id): Path<Uuid>,
) -> Result<Json<switchboard_core::channels::Channel>, String> {
    store.get_channel(id).await.map(Json).map_err(|e| e.to_string())
}

#[derive(Deserialize)]
struct CreateChannel {
    name: String,
    description: String,
}

async fn create_channel<S: ChannelStore + 'static>(
    State(store): State<Arc<S>>,
    Json(body): Json<CreateChannel>,
) -> Result<Json<switchboard_core::channels::Channel>, String> {
    store.create_channel(body.name, body.description).await.map(Json).map_err(|e| e.to_string())
}

async fn list_threads<S: ChannelStore + 'static>(
    State(store): State<Arc<S>>,
    Path(channel_id): Path<Uuid>,
) -> Result<Json<Vec<switchboard_core::channels::Thread>>, String> {
    store.list_threads(channel_id).await.map(Json).map_err(|e| e.to_string())
}

async fn get_thread<S: ChannelStore + 'static>(
    State(store): State<Arc<S>>,
    Path(id): Path<Uuid>,
) -> Result<Json<switchboard_core::channels::Thread>, String> {
    store.get_thread(id).await.map(Json).map_err(|e| e.to_string())
}

#[derive(Deserialize)]
struct CreateThread {
    title: String,
    author: String,
}

async fn create_thread<S: ChannelStore + 'static>(
    State(store): State<Arc<S>>,
    Path(channel_id): Path<Uuid>,
    Json(body): Json<CreateThread>,
) -> Result<Json<switchboard_core::channels::Thread>, String> {
    store.create_thread(channel_id, body.title, body.author).await.map(Json).map_err(|e| e.to_string())
}

async fn list_posts<S: ChannelStore + 'static>(
    State(store): State<Arc<S>>,
    Path(thread_id): Path<Uuid>,
) -> Result<Json<Vec<switchboard_core::channels::Post>>, String> {
    store.list_posts(thread_id).await.map(Json).map_err(|e| e.to_string())
}

#[derive(Deserialize)]
struct CreatePost {
    author: String,
    content: String,
}

async fn create_post<S: ChannelStore + 'static>(
    State(store): State<Arc<S>>,
    Path(thread_id): Path<Uuid>,
    Json(body): Json<CreatePost>,
) -> Result<Json<switchboard_core::channels::Post>, String> {
    store.create_post(thread_id, body.author, body.content).await.map(Json).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use switchboard_core::channels::{Channel, ChannelError, Post, Thread};
    use tower::ServiceExt;

    struct MockChannelStore;

    #[async_trait]
    impl ChannelStore for MockChannelStore {
        async fn list_channels(&self) -> Result<Vec<Channel>, ChannelError> { unimplemented!() }
        async fn get_channel(&self, _id: Uuid) -> Result<Channel, ChannelError> { unimplemented!() }
        async fn create_channel(&self, _name: String, _description: String) -> Result<Channel, ChannelError> { unimplemented!() }
        async fn list_threads(&self, _channel_id: Uuid) -> Result<Vec<Thread>, ChannelError> { unimplemented!() }
        async fn get_thread(&self, _id: Uuid) -> Result<Thread, ChannelError> { unimplemented!() }
        async fn create_thread(&self, _channel_id: Uuid, _title: String, _author: String) -> Result<Thread, ChannelError> { unimplemented!() }
        async fn list_posts(&self, _thread_id: Uuid) -> Result<Vec<Post>, ChannelError> { unimplemented!() }
        async fn create_post(&self, _thread_id: Uuid, _author: String, _content: String) -> Result<Post, ChannelError> { unimplemented!() }
    }

    #[tokio::test]
    async fn test_channel_schema_endpoint() {
        let app = router(Arc::new(MockChannelStore));
        let req = Request::builder()
            .uri("/schema/channels")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let schema: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(schema["resource"], "channels");
        assert_eq!(schema["required"], serde_json::json!(["name", "description"]));
        assert_eq!(schema["optional"], serde_json::json!([]));
    }
}
