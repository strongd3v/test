//! Shared error surface for local sync admission.

/// Errors surfaced by local sync admission.
#[derive(Debug, Clone, thiserror::Error)]
pub enum SyncError {
    #[error("websocket: {0}")]
    WebSocket(String),
    #[error("protocol: {0}")]
    Protocol(String),
    #[error("join refused: {0}")]
    JoinRefused(String),
    #[error("loro: {0}")]
    Loro(String),
    #[error("auth: {0}")]
    Auth(String),
    #[error("temporarily unavailable: {0}")]
    TemporarilyUnavailable(String),
    #[error("client is shut down")]
    Closed,
}
