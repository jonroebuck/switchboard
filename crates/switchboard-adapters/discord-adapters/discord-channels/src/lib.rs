use async_trait::async_trait;
use switchboard_core::channels::*;
use uuid::Uuid;

pub struct DiscordChannelStore;

#[async_trait]
impl ChannelStore for DiscordChannelStore {
    async fn list_channels(&self) -> Result<Vec<Channel>, ChannelError> { todo!() }
    async fn get_channel(&self, _id: Uuid) -> Result<Channel, ChannelError> { todo!() }
    async fn create_channel(&self, _name: String, _description: String) -> Result<Channel, ChannelError> { todo!() }
    async fn list_threads(&self, _channel_id: Uuid) -> Result<Vec<Thread>, ChannelError> { todo!() }
    async fn get_thread(&self, _id: Uuid) -> Result<Thread, ChannelError> { todo!() }
    async fn create_thread(&self, _channel_id: Uuid, _title: String, _author: String) -> Result<Thread, ChannelError> { todo!() }
    async fn list_posts(&self, _thread_id: Uuid) -> Result<Vec<Post>, ChannelError> { todo!() }
    async fn create_post(&self, _thread_id: Uuid, _author: String, _content: String) -> Result<Post, ChannelError> { todo!() }
}
