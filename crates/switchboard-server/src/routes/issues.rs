use std::sync::Arc;

use axum::extract::{Path, State};
use axum::routing::{get, put};
use axum::{Json, Router};
use serde::Deserialize;
use switchboard_core::issues::IssueStore;
use switchboard_core::schema::CreateSchema;
use uuid::Uuid;

pub fn router<S: IssueStore + 'static>(store: Arc<S>) -> Router {
    Router::new()
        .route("/schema/issues", get(issue_schema::<S>))
        .route("/issues", get(list_issues::<S>).post(create_issue::<S>))
        .route("/issues/{id}", get(get_issue::<S>))
        .route("/issues/{id}/status", put(update_status::<S>))
        .route("/issues/{id}/events", get(list_events::<S>))
        .with_state(store)
}

async fn issue_schema<S: IssueStore + 'static>(
    State(store): State<Arc<S>>,
) -> Json<CreateSchema> {
    Json(store.create_issue_schema())
}

async fn list_issues<S: IssueStore + 'static>(
    State(store): State<Arc<S>>,
) -> Result<Json<Vec<switchboard_core::issues::Issue>>, String> {
    store.list_issues().await.map(Json).map_err(|e| e.to_string())
}

async fn get_issue<S: IssueStore + 'static>(
    State(store): State<Arc<S>>,
    Path(id): Path<Uuid>,
) -> Result<Json<switchboard_core::issues::Issue>, String> {
    store.get_issue(id).await.map(Json).map_err(|e| e.to_string())
}

#[derive(Deserialize)]
struct CreateIssue {
    title: String,
    description: String,
}

async fn create_issue<S: IssueStore + 'static>(
    State(store): State<Arc<S>>,
    Json(body): Json<CreateIssue>,
) -> Result<Json<switchboard_core::issues::Issue>, String> {
    store.create_issue(body.title, body.description).await.map(Json).map_err(|e| e.to_string())
}

#[derive(Deserialize)]
struct UpdateStatus {
    status: String,
    note: Option<String>,
}

async fn update_status<S: IssueStore + 'static>(
    State(store): State<Arc<S>>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateStatus>,
) -> Result<Json<switchboard_core::issues::IssueEvent>, String> {
    store.update_issue_status(id, body.status, body.note).await.map(Json).map_err(|e| e.to_string())
}

async fn list_events<S: IssueStore + 'static>(
    State(store): State<Arc<S>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<switchboard_core::issues::IssueEvent>>, String> {
    store.list_events(id).await.map(Json).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use switchboard_core::issues::{Issue, IssueError, IssueEvent};
    use tower::ServiceExt;

    struct MockIssueStore;

    #[async_trait]
    impl IssueStore for MockIssueStore {
        async fn list_issues(&self) -> Result<Vec<Issue>, IssueError> { unimplemented!() }
        async fn get_issue(&self, _id: Uuid) -> Result<Issue, IssueError> { unimplemented!() }
        async fn create_issue(&self, _title: String, _description: String) -> Result<Issue, IssueError> { unimplemented!() }
        async fn update_issue_status(&self, _id: Uuid, _status: String, _note: Option<String>) -> Result<IssueEvent, IssueError> { unimplemented!() }
        async fn list_events(&self, _issue_id: Uuid) -> Result<Vec<IssueEvent>, IssueError> { unimplemented!() }
    }

    #[tokio::test]
    async fn test_issue_schema_endpoint() {
        let app = router(Arc::new(MockIssueStore));
        let req = Request::builder()
            .uri("/schema/issues")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let schema: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(schema["resource"], "issues");
        assert_eq!(schema["required"], serde_json::json!(["title", "description"]));
        assert_eq!(schema["optional"], serde_json::json!([]));
    }
}
