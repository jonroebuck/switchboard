use async_trait::async_trait;
use switchboard_core::channels::*;
use uuid::Uuid;

pub struct SlackChannelStore;

#[async_trait]
impl ChannelStore for SlackChannelStore {
    async fn list_channels(&self) -> Result<Vec<Channel>, ChannelError> { todo!() }
    async fn get_channel(&self, _id: Uuid) -> Result<Channel, ChannelError> { todo!() }
    async fn create_channel(&self, _name: String, _description: String) -> Result<Channel, ChannelError> { todo!() }
    async fn list_threads(&self, _channel_id: Uuid) -> Result<Vec<Thread>, ChannelError> { todo!() }
    async fn get_thread(&self, _id: Uuid) -> Result<Thread, ChannelError> { todo!() }
    async fn create_thread(&self, _channel_id: Uuid, _title: String, _author: String) -> Result<Thread, ChannelError> { todo!() }
    async fn list_posts(&self, _thread_id: Uuid) -> Result<Vec<Post>, ChannelError> { todo!() }
    async fn create_post(&self, _thread_id: Uuid, _author: String, _content: String) -> Result<Post, ChannelError> { todo!() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use switchboard_core::channels::ChannelStore;

    #[test]
    fn test_create_channel_schema() {
        let schema = SlackChannelStore.create_channel_schema();
        assert_eq!(schema.resource, "channels");
        assert_eq!(schema.required, vec!["name", "description"]);
        assert!(schema.optional.is_empty());
    }
}
