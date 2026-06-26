use async_trait::async_trait;
use reqwest::Client;
use switchboard_core::artifacts::*;
use uuid::Uuid;

pub struct KlondikeArtifactStore {
    client: Client,
    base_url: String,
}

impl KlondikeArtifactStore {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }
}

#[async_trait]
impl ArtifactStore for KlondikeArtifactStore {
    async fn list_artifacts(&self) -> Result<Vec<Artifact>, ArtifactError> {
        self.client
            .get(format!("{}/api/v1/artifacts", self.base_url))
            .send()
            .await
            .map_err(|e| ArtifactError::Internal(e.to_string()))?
            .json()
            .await
            .map_err(|e| ArtifactError::Internal(e.to_string()))
    }

    async fn get_artifact(&self, id: Uuid) -> Result<Artifact, ArtifactError> {
        self.client
            .get(format!("{}/api/v1/artifacts/{id}", self.base_url))
            .send()
            .await
            .map_err(|e| ArtifactError::Internal(e.to_string()))?
            .json()
            .await
            .map_err(|e| ArtifactError::Internal(e.to_string()))
    }

    async fn create_artifact(
        &self,
        name: String,
        version: String,
        source_type: String,
        source_location: String,
        content_type: String,
    ) -> Result<Artifact, ArtifactError> {
        self.client
            .post(format!("{}/api/v1/artifacts", self.base_url))
            .json(&serde_json::json!({
                "name": name,
                "version": version,
                "source_type": source_type,
                "source_location": source_location,
                "content_type": content_type,
            }))
            .send()
            .await
            .map_err(|e| ArtifactError::Internal(e.to_string()))?
            .json()
            .await
            .map_err(|e| ArtifactError::Internal(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use switchboard_core::artifacts::ArtifactStore;

    #[test]
    fn test_create_artifact_schema() {
        let store = KlondikeArtifactStore::new("http://localhost:3000");
        let schema = store.create_artifact_schema();
        assert_eq!(schema.resource, "artifacts");
        assert_eq!(
            schema.required,
            vec!["name", "version", "source_type", "source_location", "content_type"]
        );
        assert!(schema.optional.is_empty());
    }
}
