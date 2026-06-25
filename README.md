# Switchboard

Unified API gateway that normalizes channels, issues, and artifacts across multiple platform adapters behind a single REST interface.

## Architecture

Switchboard uses a trait-based adapter pattern. The core crate defines platform-agnostic traits (`ChannelStore`, `IssueStore`, `ArtifactStore`) and domain types. Each adapter crate implements one trait by calling a specific platform's API. The server crate wires the configured adapters together behind Axum routes under `/api/v1/`.

```
switchboard-server (Axum)
  └── /api/v1/
        ├── channels, threads, posts  → ChannelStore impl
        ├── issues, events            → IssueStore impl
        └── artifacts                 → ArtifactStore impl

switchboard-core
  └── Traits + domain types (no platform deps)

switchboard-adapters/
  ├── klondike-adapters/   (channels, issues, artifacts — implemented via reqwest)
  ├── slack-adapters/      (channels — stub)
  ├── discord-adapters/    (channels — stub)
  ├── github-adapters/     (issues — stub)
  ├── jira-adapters/       (issues — stub)
  └── sharepoint-adapters/ (artifacts — stub)
```

## Adapters

| Adapter | Trait | Status |
|---------|-------|--------|
| klondike-channels | ChannelStore | Implemented |
| klondike-issues | IssueStore | Implemented |
| klondike-artifacts | ArtifactStore | Implemented |
| slack-channels | ChannelStore | Stub |
| discord-channels | ChannelStore | Stub |
| github-issues | IssueStore | Stub |
| jira-issues | IssueStore | Stub |
| sharepoint-artifacts | ArtifactStore | Stub |

## Running

```sh
# Local
cargo run -p switchboard-server

# Docker
docker compose up --build
```

Set `KLONDIKE_URL` to point to your Klondike instance (defaults to `http://localhost:3000`).
