use std::sync::Arc;

use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use switchboard_core::artifacts::ArtifactStore;
use uuid::Uuid;

pub fn router<S: ArtifactStore + 'static>(store: Arc<S>) -> Router {
    Router::new()
        .route("/artifacts", get(list_artifacts::<S>).post(create_artifact::<S>))
        .route("/artifacts/{id}", get(get_artifact::<S>))
        .with_state(store)
}

async fn list_artifacts<S: ArtifactStore + 'static>(
    State(store): State<Arc<S>>,
) -> Result<Json<Vec<switchboard_core::artifacts::Artifact>>, String> {
    store.list_artifacts().await.map(Json).map_err(|e| e.to_string())
}

async fn get_artifact<S: ArtifactStore + 'static>(
    State(store): State<Arc<S>>,
    Path(id): Path<Uuid>,
) -> Result<Json<switchboard_core::artifacts::Artifact>, String> {
    store.get_artifact(id).await.map(Json).map_err(|e| e.to_string())
}

#[derive(Deserialize)]
struct CreateArtifact {
    name: String,
    version: String,
    source_type: String,
    source_location: String,
    content_type: String,
}

async fn create_artifact<S: ArtifactStore + 'static>(
    State(store): State<Arc<S>>,
    Json(body): Json<CreateArtifact>,
) -> Result<Json<switchboard_core::artifacts::Artifact>, String> {
    store
        .create_artifact(body.name, body.version, body.source_type, body.source_location, body.content_type)
        .await
        .map(Json)
        .map_err(|e| e.to_string())
}
