use chrono::Utc;
use klondike_artifacts::KlondikeArtifactStore;
use serde_json::json;
use switchboard_core::artifacts::ArtifactStore;
use uuid::Uuid;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn create_artifact() {
    let server = MockServer::start().await;
    let id = Uuid::new_v4();
    let now = Utc::now();

    Mock::given(method("POST"))
        .and(path("/api/v1/artifacts"))
        .and(body_json(json!({
            "name": "doc.pdf",
            "version": "1",
            "source_type": "upload",
            "source_location": "/files/doc.pdf",
            "content_type": "application/pdf"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": id,
            "name": "doc.pdf",
            "version": "1",
            "source_type": "upload",
            "source_location": "/files/doc.pdf",
            "content_type": "application/pdf",
            "created_at": now
        })))
        .expect(1)
        .mount(&server)
        .await;

    let store = KlondikeArtifactStore::new(&server.uri());
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
    assert_eq!(artifact.id, id);
    assert_eq!(artifact.name, "doc.pdf");
    assert_eq!(artifact.version, "1");
    assert_eq!(artifact.content_type, "application/pdf");
}

#[tokio::test]
async fn get_artifact() {
    let server = MockServer::start().await;
    let id = Uuid::new_v4();
    let now = Utc::now();

    Mock::given(method("GET"))
        .and(path(format!("/api/v1/artifacts/{id}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": id,
            "name": "doc.pdf",
            "version": "2",
            "source_type": "upload",
            "source_location": "/files/doc.pdf",
            "content_type": "application/pdf",
            "created_at": now
        })))
        .expect(1)
        .mount(&server)
        .await;

    let store = KlondikeArtifactStore::new(&server.uri());
    let artifact = store.get_artifact(id).await.unwrap();
    assert_eq!(artifact.id, id);
    assert_eq!(artifact.name, "doc.pdf");
    assert_eq!(artifact.version, "2");
}

#[tokio::test]
async fn list_artifacts() {
    let server = MockServer::start().await;
    let now = Utc::now();

    Mock::given(method("GET"))
        .and(path("/api/v1/artifacts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "id": Uuid::new_v4(),
                "name": "a",
                "version": "1",
                "source_type": "s",
                "source_location": "l",
                "content_type": "c",
                "created_at": now
            },
            {
                "id": Uuid::new_v4(),
                "name": "b",
                "version": "1",
                "source_type": "s",
                "source_location": "l",
                "content_type": "c",
                "created_at": now
            }
        ])))
        .expect(1)
        .mount(&server)
        .await;

    let store = KlondikeArtifactStore::new(&server.uri());
    let artifacts = store.list_artifacts().await.unwrap();
    assert_eq!(artifacts.len(), 2);
    assert_eq!(artifacts[0].name, "a");
    assert_eq!(artifacts[1].name, "b");
}

#[tokio::test]
async fn versioning_across_writes() {
    let server = MockServer::start().await;
    let now = Utc::now();

    for v in 1..=3u32 {
        let version = v.to_string();
        Mock::given(method("POST"))
            .and(path("/api/v1/artifacts"))
            .and(body_json(json!({
                "name": "report",
                "version": version,
                "source_type": "upload",
                "source_location": "/files/report",
                "content_type": "text/plain"
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": Uuid::new_v4(),
                "name": "report",
                "version": version,
                "source_type": "upload",
                "source_location": "/files/report",
                "content_type": "text/plain",
                "created_at": now
            })))
            .expect(1)
            .mount(&server)
            .await;
    }

    let store = KlondikeArtifactStore::new(&server.uri());

    let v1 = store
        .create_artifact(
            "report".into(),
            "1".into(),
            "upload".into(),
            "/files/report".into(),
            "text/plain".into(),
        )
        .await
        .unwrap();
    let v2 = store
        .create_artifact(
            "report".into(),
            "2".into(),
            "upload".into(),
            "/files/report".into(),
            "text/plain".into(),
        )
        .await
        .unwrap();
    let v3 = store
        .create_artifact(
            "report".into(),
            "3".into(),
            "upload".into(),
            "/files/report".into(),
            "text/plain".into(),
        )
        .await
        .unwrap();

    assert_eq!(v1.version, "1");
    assert_eq!(v2.version, "2");
    assert_eq!(v3.version, "3");
    assert_eq!(v1.name, "report");
    assert_eq!(v2.name, "report");
    assert_eq!(v3.name, "report");
}
