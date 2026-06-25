use std::sync::Arc;

use axum::extract::{Path, State};
use axum::routing::{get, put};
use axum::{Json, Router};
use serde::Deserialize;
use switchboard_core::issues::IssueStore;
use uuid::Uuid;

pub fn router<S: IssueStore + 'static>(store: Arc<S>) -> Router {
    Router::new()
        .route("/issues", get(list_issues::<S>).post(create_issue::<S>))
        .route("/issues/{id}", get(get_issue::<S>))
        .route("/issues/{id}/status", put(update_status::<S>))
        .route("/issues/{id}/events", get(list_events::<S>))
        .with_state(store)
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
