use async_trait::async_trait;
use reqwest::Client;
use switchboard_core::issues::*;
use uuid::Uuid;

pub struct KlondikeIssueStore {
    client: Client,
    base_url: String,
}

impl KlondikeIssueStore {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }
}

#[async_trait]
impl IssueStore for KlondikeIssueStore {
    async fn list_issues(&self) -> Result<Vec<Issue>, IssueError> {
        self.client
            .get(format!("{}/api/v1/issues", self.base_url))
            .send()
            .await
            .map_err(|e| IssueError::Internal(e.to_string()))?
            .json()
            .await
            .map_err(|e| IssueError::Internal(e.to_string()))
    }

    async fn get_issue(&self, id: Uuid) -> Result<Issue, IssueError> {
        self.client
            .get(format!("{}/api/v1/issues/{id}", self.base_url))
            .send()
            .await
            .map_err(|e| IssueError::Internal(e.to_string()))?
            .json()
            .await
            .map_err(|e| IssueError::Internal(e.to_string()))
    }

    async fn create_issue(&self, title: String, description: String) -> Result<Issue, IssueError> {
        self.client
            .post(format!("{}/api/v1/issues", self.base_url))
            .json(&serde_json::json!({ "title": title, "description": description }))
            .send()
            .await
            .map_err(|e| IssueError::Internal(e.to_string()))?
            .json()
            .await
            .map_err(|e| IssueError::Internal(e.to_string()))
    }

    async fn update_issue_status(&self, id: Uuid, status: String, note: Option<String>) -> Result<IssueEvent, IssueError> {
        self.client
            .put(format!("{}/api/v1/issues/{id}/status", self.base_url))
            .json(&serde_json::json!({ "status": status, "note": note }))
            .send()
            .await
            .map_err(|e| IssueError::Internal(e.to_string()))?
            .json()
            .await
            .map_err(|e| IssueError::Internal(e.to_string()))
    }

    async fn list_events(&self, issue_id: Uuid) -> Result<Vec<IssueEvent>, IssueError> {
        self.client
            .get(format!("{}/api/v1/issues/{issue_id}/events", self.base_url))
            .send()
            .await
            .map_err(|e| IssueError::Internal(e.to_string()))?
            .json()
            .await
            .map_err(|e| IssueError::Internal(e.to_string()))
    }
}
