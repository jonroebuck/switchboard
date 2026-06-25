mod routes;

use std::sync::Arc;

use axum::Router;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let klondike_url =
        std::env::var("KLONDIKE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());

    let channel_store = Arc::new(klondike_channels::KlondikeChannelStore::new(&klondike_url));
    let issue_store = Arc::new(klondike_issues::KlondikeIssueStore::new(&klondike_url));
    let artifact_store = Arc::new(klondike_artifacts::KlondikeArtifactStore::new(&klondike_url));

    let app = Router::new().nest(
        "/api/v1",
        Router::new()
            .merge(routes::channels::router(channel_store))
            .merge(routes::issues::router(issue_store))
            .merge(routes::artifacts::router(artifact_store)),
    );

    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("switchboard listening on 0.0.0.0:8080");
    axum::serve(listener, app).await.unwrap();
}
