use async_trait::async_trait;
use switchboard_core::artifacts::*;
use uuid::Uuid;

pub struct SharepointArtifactStore;

#[async_trait]
impl ArtifactStore for SharepointArtifactStore {
    async fn list_artifacts(&self) -> Result<Vec<Artifact>, ArtifactError> { todo!() }
    async fn get_artifact(&self, _id: Uuid) -> Result<Artifact, ArtifactError> { todo!() }
    async fn create_artifact(&self, _name: String, _version: String, _source_type: String, _source_location: String, _content_type: String) -> Result<Artifact, ArtifactError> { todo!() }
}
