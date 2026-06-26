use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::CreateSchema;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub source_type: String,
    pub source_location: String,
    pub content_type: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum ArtifactError {
    #[error("artifact not found: {0}")]
    NotFound(Uuid),
    #[error("{0}")]
    Internal(String),
}

#[async_trait]
pub trait ArtifactStore: Send + Sync {
    fn create_artifact_schema(&self) -> CreateSchema {
        CreateSchema {
            resource: "artifacts".to_string(),
            required: vec![
                "name".to_string(),
                "version".to_string(),
                "source_type".to_string(),
                "source_location".to_string(),
                "content_type".to_string(),
            ],
            optional: vec![],
        }
    }

    async fn list_artifacts(&self) -> Result<Vec<Artifact>, ArtifactError>;
    async fn get_artifact(&self, id: Uuid) -> Result<Artifact, ArtifactError>;
    async fn create_artifact(
        &self,
        name: String,
        version: String,
        source_type: String,
        source_location: String,
        content_type: String,
    ) -> Result<Artifact, ArtifactError>;
}
