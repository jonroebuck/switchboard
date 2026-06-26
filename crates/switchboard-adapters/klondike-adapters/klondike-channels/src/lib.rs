use async_trait::async_trait;
use reqwest::Client;
use switchboard_core::channels::*;
use uuid::Uuid;

pub struct KlondikeChannelStore {
    client: Client,
    base_url: String,
}

impl KlondikeChannelStore {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }
}

#[async_trait]
impl ChannelStore for KlondikeChannelStore {
    async fn list_channels(&self) -> Result<Vec<Channel>, ChannelError> {
        self.client
            .get(format!("{}/api/v1/channels", self.base_url))
            .send()
            .await
            .map_err(|e| ChannelError::Internal(e.to_string()))?
            .json()
            .await
            .map_err(|e| ChannelError::Internal(e.to_string()))
    }

    async fn get_channel(&self, id: Uuid) -> Result<Channel, ChannelError> {
        self.client
            .get(format!("{}/api/v1/channels/{id}", self.base_url))
            .send()
            .await
            .map_err(|e| ChannelError::Internal(e.to_string()))?
            .json()
            .await
            .map_err(|e| ChannelError::Internal(e.to_string()))
    }

    async fn create_channel(&self, name: String, description: String) -> Result<Channel, ChannelError> {
        self.client
            .post(format!("{}/api/v1/channels", self.base_url))
            .json(&serde_json::json!({ "name": name, "description": description }))
            .send()
            .await
            .map_err(|e| ChannelError::Internal(e.to_string()))?
            .json()
            .await
            .map_err(|e| ChannelError::Internal(e.to_string()))
    }

    async fn list_threads(&self, channel_id: Uuid) -> Result<Vec<Thread>, ChannelError> {
        self.client
            .get(format!("{}/api/v1/channels/{channel_id}/threads", self.base_url))
            .send()
            .await
            .map_err(|e| ChannelError::Internal(e.to_string()))?
            .json()
            .await
            .map_err(|e| ChannelError::Internal(e.to_string()))
    }

    async fn get_thread(&self, id: Uuid) -> Result<Thread, ChannelError> {
        self.client
            .get(format!("{}/api/v1/threads/{id}", self.base_url))
            .send()
            .await
            .map_err(|e| ChannelError::Internal(e.to_string()))?
            .json()
            .await
            .map_err(|e| ChannelError::Internal(e.to_string()))
    }

    async fn create_thread(&self, channel_id: Uuid, title: String, author: String) -> Result<Thread, ChannelError> {
        self.client
            .post(format!("{}/api/v1/channels/{channel_id}/threads", self.base_url))
            .json(&serde_json::json!({ "title": title, "author": author }))
            .send()
            .await
            .map_err(|e| ChannelError::Internal(e.to_string()))?
            .json()
            .await
            .map_err(|e| ChannelError::Internal(e.to_string()))
    }

    async fn list_posts(&self, thread_id: Uuid) -> Result<Vec<Post>, ChannelError> {
        self.client
            .get(format!("{}/api/v1/threads/{thread_id}/posts", self.base_url))
            .send()
            .await
            .map_err(|e| ChannelError::Internal(e.to_string()))?
            .json()
            .await
            .map_err(|e| ChannelError::Internal(e.to_string()))
    }

    async fn create_post(&self, thread_id: Uuid, author: String, content: String) -> Result<Post, ChannelError> {
        self.client
            .post(format!("{}/api/v1/threads/{thread_id}/posts", self.base_url))
            .json(&serde_json::json!({ "author": author, "content": content }))
            .send()
            .await
            .map_err(|e| ChannelError::Internal(e.to_string()))?
            .json()
            .await
            .map_err(|e| ChannelError::Internal(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use switchboard_core::channels::ChannelStore;

    #[test]
    fn test_create_channel_schema() {
        let store = KlondikeChannelStore::new("http://localhost:3000");
        let schema = store.create_channel_schema();
        assert_eq!(schema.resource, "channels");
        assert_eq!(schema.required, vec!["name", "description"]);
        assert!(schema.optional.is_empty());
    }
}
