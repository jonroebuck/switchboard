use std::sync::Arc;

use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use switchboard_core::channels::ChannelStore;
use uuid::Uuid;

pub fn router<S: ChannelStore + 'static>(store: Arc<S>) -> Router {
    Router::new()
        .route("/channels", get(list_channels::<S>).post(create_channel::<S>))
        .route("/channels/{id}", get(get_channel::<S>))
        .route("/channels/{channel_id}/threads", get(list_threads::<S>).post(create_thread::<S>))
        .route("/threads/{id}", get(get_thread::<S>))
        .route("/threads/{thread_id}/posts", get(list_posts::<S>).post(create_post::<S>))
        .with_state(store)
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
