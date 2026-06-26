use std::sync::Arc;

use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use switchboard_core::artifacts::ArtifactStore;
use switchboard_core::schema::CreateSchema;
use uuid::Uuid;

pub fn router<S: ArtifactStore + 'static>(store: Arc<S>) -> Router {
    Router::new()
        .route("/schema/artifacts", get(artifact_schema::<S>))
        .route("/artifacts", get(list_artifacts::<S>).post(create_artifact::<S>))
        .route("/artifacts/{id}", get(get_artifact::<S>))
        .with_state(store)
}

async fn artifact_schema<S: ArtifactStore + 'static>(
    State(store): State<Arc<S>>,
) -> Json<CreateSchema> {
    Json(store.create_artifact_schema())
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

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use switchboard_core::artifacts::{Artifact, ArtifactError};
    use tower::ServiceExt;

    struct MockArtifactStore;

    #[async_trait]
    impl ArtifactStore for MockArtifactStore {
        async fn list_artifacts(&self) -> Result<Vec<Artifact>, ArtifactError> { unimplemented!() }
        async fn get_artifact(&self, _id: Uuid) -> Result<Artifact, ArtifactError> { unimplemented!() }
        async fn create_artifact(&self, _name: String, _version: String, _source_type: String, _source_location: String, _content_type: String) -> Result<Artifact, ArtifactError> { unimplemented!() }
    }

    #[tokio::test]
    async fn test_artifact_schema_endpoint() {
        let app = router(Arc::new(MockArtifactStore));
        let req = Request::builder()
            .uri("/schema/artifacts")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let schema: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(schema["resource"], "artifacts");
        assert_eq!(
            schema["required"],
            serde_json::json!(["name", "version", "source_type", "source_location", "content_type"])
        );
        assert_eq!(schema["optional"], serde_json::json!([]));
    }
}
