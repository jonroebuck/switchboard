use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::CreateSchema;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thread {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub title: String,
    pub author: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: Uuid,
    pub thread_id: Uuid,
    pub author: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum ChannelError {
    #[error("channel not found: {0}")]
    NotFound(Uuid),
    #[error("{0}")]
    Internal(String),
}

#[async_trait]
pub trait ChannelStore: Send + Sync {
    fn create_channel_schema(&self) -> CreateSchema {
        CreateSchema {
            resource: "channels".to_string(),
            required: vec!["name".to_string(), "description".to_string()],
            optional: vec![],
        }
    }

    async fn list_channels(&self) -> Result<Vec<Channel>, ChannelError>;
    async fn get_channel(&self, id: Uuid) -> Result<Channel, ChannelError>;
    async fn create_channel(&self, name: String, description: String) -> Result<Channel, ChannelError>;

    async fn list_threads(&self, channel_id: Uuid) -> Result<Vec<Thread>, ChannelError>;
    async fn get_thread(&self, id: Uuid) -> Result<Thread, ChannelError>;
    async fn create_thread(&self, channel_id: Uuid, title: String, author: String) -> Result<Thread, ChannelError>;

    async fn list_posts(&self, thread_id: Uuid) -> Result<Vec<Post>, ChannelError>;
    async fn create_post(&self, thread_id: Uuid, author: String, content: String) -> Result<Post, ChannelError>;
}
