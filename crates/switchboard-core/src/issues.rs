use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub status: String,
    pub assignee: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueEvent {
    pub id: Uuid,
    pub issue_id: Uuid,
    pub from_status: String,
    pub to_status: String,
    pub note: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum IssueError {
    #[error("issue not found: {0}")]
    NotFound(Uuid),
    #[error("{0}")]
    Internal(String),
}

#[async_trait]
pub trait IssueStore: Send + Sync {
    async fn list_issues(&self) -> Result<Vec<Issue>, IssueError>;
    async fn get_issue(&self, id: Uuid) -> Result<Issue, IssueError>;
    async fn create_issue(&self, title: String, description: String) -> Result<Issue, IssueError>;
    async fn update_issue_status(&self, id: Uuid, status: String, note: Option<String>) -> Result<IssueEvent, IssueError>;
    async fn list_events(&self, issue_id: Uuid) -> Result<Vec<IssueEvent>, IssueError>;
}
