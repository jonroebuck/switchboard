use async_trait::async_trait;
use switchboard_core::issues::*;
use uuid::Uuid;

pub struct GithubIssueStore;

#[async_trait]
impl IssueStore for GithubIssueStore {
    async fn list_issues(&self) -> Result<Vec<Issue>, IssueError> { todo!() }
    async fn get_issue(&self, _id: Uuid) -> Result<Issue, IssueError> { todo!() }
    async fn create_issue(&self, _title: String, _description: String) -> Result<Issue, IssueError> { todo!() }
    async fn update_issue_status(&self, _id: Uuid, _status: String, _note: Option<String>) -> Result<IssueEvent, IssueError> { todo!() }
    async fn list_events(&self, _issue_id: Uuid) -> Result<Vec<IssueEvent>, IssueError> { todo!() }
}
