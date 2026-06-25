use async_trait::async_trait;
use chrono::Utc;
use switchboard_core::artifacts::*;
use switchboard_core::channels::*;
use switchboard_core::issues::*;
use tokio::sync::Mutex;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// In-memory ChannelStore
// ---------------------------------------------------------------------------

struct InMemoryChannelStore {
    channels: Mutex<Vec<Channel>>,
    threads: Mutex<Vec<Thread>>,
    posts: Mutex<Vec<Post>>,
}

impl InMemoryChannelStore {
    fn new() -> Self {
        Self {
            channels: Mutex::new(Vec::new()),
            threads: Mutex::new(Vec::new()),
            posts: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl ChannelStore for InMemoryChannelStore {
    async fn list_channels(&self) -> Result<Vec<Channel>, ChannelError> {
        Ok(self.channels.lock().await.clone())
    }

    async fn get_channel(&self, id: Uuid) -> Result<Channel, ChannelError> {
        self.channels
            .lock()
            .await
            .iter()
            .find(|c| c.id == id)
            .cloned()
            .ok_or(ChannelError::NotFound(id))
    }

    async fn create_channel(
        &self,
        name: String,
        description: String,
    ) -> Result<Channel, ChannelError> {
        let channel = Channel {
            id: Uuid::new_v4(),
            name,
            description,
            created_at: Utc::now(),
        };
        self.channels.lock().await.push(channel.clone());
        Ok(channel)
    }

    async fn list_threads(&self, channel_id: Uuid) -> Result<Vec<Thread>, ChannelError> {
        Ok(self
            .threads
            .lock()
            .await
            .iter()
            .filter(|t| t.channel_id == channel_id)
            .cloned()
            .collect())
    }

    async fn get_thread(&self, id: Uuid) -> Result<Thread, ChannelError> {
        self.threads
            .lock()
            .await
            .iter()
            .find(|t| t.id == id)
            .cloned()
            .ok_or(ChannelError::NotFound(id))
    }

    async fn create_thread(
        &self,
        channel_id: Uuid,
        title: String,
        author: String,
    ) -> Result<Thread, ChannelError> {
        let thread = Thread {
            id: Uuid::new_v4(),
            channel_id,
            title,
            author,
            created_at: Utc::now(),
        };
        self.threads.lock().await.push(thread.clone());
        Ok(thread)
    }

    async fn list_posts(&self, thread_id: Uuid) -> Result<Vec<Post>, ChannelError> {
        Ok(self
            .posts
            .lock()
            .await
            .iter()
            .filter(|p| p.thread_id == thread_id)
            .cloned()
            .collect())
    }

    async fn create_post(
        &self,
        thread_id: Uuid,
        author: String,
        content: String,
    ) -> Result<Post, ChannelError> {
        let post = Post {
            id: Uuid::new_v4(),
            thread_id,
            author,
            content,
            created_at: Utc::now(),
        };
        self.posts.lock().await.push(post.clone());
        Ok(post)
    }
}

// ---------------------------------------------------------------------------
// In-memory IssueStore
// ---------------------------------------------------------------------------

struct InMemoryIssueStore {
    issues: Mutex<Vec<Issue>>,
    events: Mutex<Vec<IssueEvent>>,
}

impl InMemoryIssueStore {
    fn new() -> Self {
        Self {
            issues: Mutex::new(Vec::new()),
            events: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl IssueStore for InMemoryIssueStore {
    async fn list_issues(&self) -> Result<Vec<Issue>, IssueError> {
        Ok(self.issues.lock().await.clone())
    }

    async fn get_issue(&self, id: Uuid) -> Result<Issue, IssueError> {
        self.issues
            .lock()
            .await
            .iter()
            .find(|i| i.id == id)
            .cloned()
            .ok_or(IssueError::NotFound(id))
    }

    async fn create_issue(
        &self,
        title: String,
        description: String,
    ) -> Result<Issue, IssueError> {
        let now = Utc::now();
        let issue = Issue {
            id: Uuid::new_v4(),
            title,
            description,
            status: "backlog".to_string(),
            assignee: None,
            created_at: now,
            updated_at: now,
        };
        self.issues.lock().await.push(issue.clone());
        Ok(issue)
    }

    async fn update_issue_status(
        &self,
        id: Uuid,
        status: String,
        note: Option<String>,
    ) -> Result<IssueEvent, IssueError> {
        let mut issues = self.issues.lock().await;
        let issue = issues
            .iter_mut()
            .find(|i| i.id == id)
            .ok_or(IssueError::NotFound(id))?;
        let from_status = issue.status.clone();
        issue.status = status.clone();
        issue.updated_at = Utc::now();
        let event = IssueEvent {
            id: Uuid::new_v4(),
            issue_id: id,
            from_status,
            to_status: status,
            note,
            timestamp: Utc::now(),
        };
        drop(issues);
        self.events.lock().await.push(event.clone());
        Ok(event)
    }

    async fn list_events(&self, issue_id: Uuid) -> Result<Vec<IssueEvent>, IssueError> {
        Ok(self
            .events
            .lock()
            .await
            .iter()
            .filter(|e| e.issue_id == issue_id)
            .cloned()
            .collect())
    }
}

// ---------------------------------------------------------------------------
// In-memory ArtifactStore
// ---------------------------------------------------------------------------

struct InMemoryArtifactStore {
    artifacts: Mutex<Vec<Artifact>>,
}

impl InMemoryArtifactStore {
    fn new() -> Self {
        Self {
            artifacts: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl ArtifactStore for InMemoryArtifactStore {
    async fn list_artifacts(&self) -> Result<Vec<Artifact>, ArtifactError> {
        Ok(self.artifacts.lock().await.clone())
    }

    async fn get_artifact(&self, id: Uuid) -> Result<Artifact, ArtifactError> {
        self.artifacts
            .lock()
            .await
            .iter()
            .find(|a| a.id == id)
            .cloned()
            .ok_or(ArtifactError::NotFound(id))
    }

    async fn create_artifact(
        &self,
        name: String,
        version: String,
        source_type: String,
        source_location: String,
        content_type: String,
    ) -> Result<Artifact, ArtifactError> {
        let artifact = Artifact {
            id: Uuid::new_v4(),
            name,
            version,
            source_type,
            source_location,
            content_type,
            created_at: Utc::now(),
        };
        self.artifacts.lock().await.push(artifact.clone());
        Ok(artifact)
    }
}

// ===========================================================================
// Channel tests
// ===========================================================================

#[tokio::test]
async fn channel_create_and_get() {
    let store = InMemoryChannelStore::new();
    let channel = store
        .create_channel("general".into(), "General chat".into())
        .await
        .unwrap();
    assert_eq!(channel.name, "general");
    assert_eq!(channel.description, "General chat");

    let fetched = store.get_channel(channel.id).await.unwrap();
    assert_eq!(fetched.id, channel.id);
    assert_eq!(fetched.name, "general");
}

#[tokio::test]
async fn channel_list() {
    let store = InMemoryChannelStore::new();
    assert!(store.list_channels().await.unwrap().is_empty());

    store
        .create_channel("a".into(), "A".into())
        .await
        .unwrap();
    store
        .create_channel("b".into(), "B".into())
        .await
        .unwrap();
    assert_eq!(store.list_channels().await.unwrap().len(), 2);
}

#[tokio::test]
async fn channel_not_found() {
    let store = InMemoryChannelStore::new();
    let result = store.get_channel(Uuid::new_v4()).await;
    assert!(matches!(result, Err(ChannelError::NotFound(_))));
}

#[tokio::test]
async fn thread_create_and_list() {
    let store = InMemoryChannelStore::new();
    let channel = store
        .create_channel("ch".into(), "desc".into())
        .await
        .unwrap();

    let thread = store
        .create_thread(channel.id, "Topic".into(), "alice".into())
        .await
        .unwrap();
    assert_eq!(thread.channel_id, channel.id);
    assert_eq!(thread.title, "Topic");
    assert_eq!(thread.author, "alice");

    let threads = store.list_threads(channel.id).await.unwrap();
    assert_eq!(threads.len(), 1);
    assert_eq!(threads[0].id, thread.id);
}

#[tokio::test]
async fn thread_get() {
    let store = InMemoryChannelStore::new();
    let channel = store
        .create_channel("ch".into(), "d".into())
        .await
        .unwrap();
    let thread = store
        .create_thread(channel.id, "T".into(), "bob".into())
        .await
        .unwrap();

    let fetched = store.get_thread(thread.id).await.unwrap();
    assert_eq!(fetched.id, thread.id);
    assert_eq!(fetched.title, "T");
}

#[tokio::test]
async fn thread_not_found() {
    let store = InMemoryChannelStore::new();
    let result = store.get_thread(Uuid::new_v4()).await;
    assert!(matches!(result, Err(ChannelError::NotFound(_))));
}

#[tokio::test]
async fn post_create_and_list() {
    let store = InMemoryChannelStore::new();
    let channel = store
        .create_channel("ch".into(), "d".into())
        .await
        .unwrap();
    let thread = store
        .create_thread(channel.id, "T".into(), "alice".into())
        .await
        .unwrap();

    let post = store
        .create_post(thread.id, "bob".into(), "Hello!".into())
        .await
        .unwrap();
    assert_eq!(post.thread_id, thread.id);
    assert_eq!(post.author, "bob");
    assert_eq!(post.content, "Hello!");

    let posts = store.list_posts(thread.id).await.unwrap();
    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].id, post.id);
}

#[tokio::test]
async fn posts_filtered_by_thread() {
    let store = InMemoryChannelStore::new();
    let ch = store
        .create_channel("ch".into(), "d".into())
        .await
        .unwrap();
    let t1 = store
        .create_thread(ch.id, "T1".into(), "a".into())
        .await
        .unwrap();
    let t2 = store
        .create_thread(ch.id, "T2".into(), "b".into())
        .await
        .unwrap();

    store
        .create_post(t1.id, "x".into(), "msg1".into())
        .await
        .unwrap();
    store
        .create_post(t2.id, "y".into(), "msg2".into())
        .await
        .unwrap();

    assert_eq!(store.list_posts(t1.id).await.unwrap().len(), 1);
    assert_eq!(store.list_posts(t2.id).await.unwrap().len(), 1);
}

// ===========================================================================
// Issue tests
// ===========================================================================

#[tokio::test]
async fn issue_create_and_get() {
    let store = InMemoryIssueStore::new();
    let issue = store
        .create_issue("Bug".into(), "Something broke".into())
        .await
        .unwrap();
    assert_eq!(issue.title, "Bug");
    assert_eq!(issue.status, "backlog");
    assert!(issue.assignee.is_none());

    let fetched = store.get_issue(issue.id).await.unwrap();
    assert_eq!(fetched.id, issue.id);
}

#[tokio::test]
async fn issue_list() {
    let store = InMemoryIssueStore::new();
    assert!(store.list_issues().await.unwrap().is_empty());

    store
        .create_issue("A".into(), "a".into())
        .await
        .unwrap();
    store
        .create_issue("B".into(), "b".into())
        .await
        .unwrap();
    assert_eq!(store.list_issues().await.unwrap().len(), 2);
}

#[tokio::test]
async fn issue_not_found() {
    let store = InMemoryIssueStore::new();
    let result = store.get_issue(Uuid::new_v4()).await;
    assert!(matches!(result, Err(IssueError::NotFound(_))));
}

#[tokio::test]
async fn issue_status_transitions() {
    let store = InMemoryIssueStore::new();
    let issue = store
        .create_issue("Task".into(), "Do stuff".into())
        .await
        .unwrap();
    assert_eq!(issue.status, "backlog");

    let ev1 = store
        .update_issue_status(issue.id, "in_progress".into(), None)
        .await
        .unwrap();
    assert_eq!(ev1.from_status, "backlog");
    assert_eq!(ev1.to_status, "in_progress");

    let ev2 = store
        .update_issue_status(issue.id, "done".into(), Some("Finished".into()))
        .await
        .unwrap();
    assert_eq!(ev2.from_status, "in_progress");
    assert_eq!(ev2.to_status, "done");
    assert_eq!(ev2.note.as_deref(), Some("Finished"));

    let ev3 = store
        .update_issue_status(issue.id, "blocked".into(), Some("Dependency".into()))
        .await
        .unwrap();
    assert_eq!(ev3.from_status, "done");
    assert_eq!(ev3.to_status, "blocked");

    let updated = store.get_issue(issue.id).await.unwrap();
    assert_eq!(updated.status, "blocked");
}

#[tokio::test]
async fn issue_update_not_found() {
    let store = InMemoryIssueStore::new();
    let result = store
        .update_issue_status(Uuid::new_v4(), "done".into(), None)
        .await;
    assert!(matches!(result, Err(IssueError::NotFound(_))));
}

#[tokio::test]
async fn issue_list_events() {
    let store = InMemoryIssueStore::new();
    let issue = store
        .create_issue("T".into(), "d".into())
        .await
        .unwrap();
    store
        .update_issue_status(issue.id, "in_progress".into(), None)
        .await
        .unwrap();
    store
        .update_issue_status(issue.id, "done".into(), None)
        .await
        .unwrap();

    let events = store.list_events(issue.id).await.unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].to_status, "in_progress");
    assert_eq!(events[1].to_status, "done");
}

#[tokio::test]
async fn issue_events_filtered_by_issue() {
    let store = InMemoryIssueStore::new();
    let i1 = store
        .create_issue("A".into(), "a".into())
        .await
        .unwrap();
    let i2 = store
        .create_issue("B".into(), "b".into())
        .await
        .unwrap();

    store
        .update_issue_status(i1.id, "in_progress".into(), None)
        .await
        .unwrap();
    store
        .update_issue_status(i2.id, "done".into(), None)
        .await
        .unwrap();

    assert_eq!(store.list_events(i1.id).await.unwrap().len(), 1);
    assert_eq!(store.list_events(i2.id).await.unwrap().len(), 1);
}

// ===========================================================================
// Artifact tests
// ===========================================================================

#[tokio::test]
async fn artifact_create_and_get() {
    let store = InMemoryArtifactStore::new();
    let artifact = store
        .create_artifact(
            "doc.pdf".into(),
            "1".into(),
            "upload".into(),
            "/files/doc.pdf".into(),
            "application/pdf".into(),
        )
        .await
        .unwrap();
    assert_eq!(artifact.name, "doc.pdf");
    assert_eq!(artifact.version, "1");
    assert_eq!(artifact.source_type, "upload");
    assert_eq!(artifact.content_type, "application/pdf");

    let fetched = store.get_artifact(artifact.id).await.unwrap();
    assert_eq!(fetched.id, artifact.id);
    assert_eq!(fetched.name, "doc.pdf");
}

#[tokio::test]
async fn artifact_list() {
    let store = InMemoryArtifactStore::new();
    assert!(store.list_artifacts().await.unwrap().is_empty());

    store
        .create_artifact("a".into(), "1".into(), "s".into(), "l".into(), "c".into())
        .await
        .unwrap();
    store
        .create_artifact("b".into(), "1".into(), "s".into(), "l".into(), "c".into())
        .await
        .unwrap();
    assert_eq!(store.list_artifacts().await.unwrap().len(), 2);
}

#[tokio::test]
async fn artifact_not_found() {
    let store = InMemoryArtifactStore::new();
    let result = store.get_artifact(Uuid::new_v4()).await;
    assert!(matches!(result, Err(ArtifactError::NotFound(_))));
}

#[tokio::test]
async fn artifact_versioning() {
    let store = InMemoryArtifactStore::new();
    let v1 = store
        .create_artifact("doc".into(), "1".into(), "s".into(), "l".into(), "c".into())
        .await
        .unwrap();
    let v2 = store
        .create_artifact("doc".into(), "2".into(), "s".into(), "l".into(), "c".into())
        .await
        .unwrap();
    let v3 = store
        .create_artifact("doc".into(), "3".into(), "s".into(), "l".into(), "c".into())
        .await
        .unwrap();

    assert_eq!(v1.version, "1");
    assert_eq!(v2.version, "2");
    assert_eq!(v3.version, "3");
    assert_ne!(v1.id, v2.id);
    assert_ne!(v2.id, v3.id);

    let all = store.list_artifacts().await.unwrap();
    assert_eq!(all.len(), 3);
    assert!(all.iter().all(|a| a.name == "doc"));
}
