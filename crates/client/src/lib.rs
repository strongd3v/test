//! space-client — the engine-free thin client ("viewer device").
//!
//! A phone (or any frontend without an engine) mirrors the workspace registry,
//! renders the session docs, and drives a simulated host through the durable
//! command ledger (`space_doc::SessionDoc::queue_command`). No agent ever runs
//! here.
//!
//! # Shape
//!
//! - [`Client`] — one offline Demo profile. Owns the registry mirror, open
//!   sessions, and background tasks on the crate's shared tokio runtime
//!   ([`runtime::shared`]). Every snapshot read is an `Arc` clone — never a
//!   whole-doc decode.
//! - [`WorkspaceSnapshot`] — the derived view models (front page with pins and
//!   sections, projects, PRs, archived, devices, search), rebuilt only when
//!   the registry, presence, or time-dependent status actually changes.
//! - [`SessionHandle`] — one chat: an incrementally maintained transcript
//!   ([`SessionSnapshot`], per-entry `Arc`s that stay pointer-equal while
//!   unchanged) plus the composer-facing [`ComposerState`] and every command.
//! - [`ClientListener`] — coalesced change notifications; platforms pull the
//!   new snapshot after an event.
//!
//! Demo mode ([`Credentials::Demo`]) runs the *same* registry/session code
//! against an in-process simulated host, so it exercises the real pipeline.

mod b64;
pub mod attachments;
pub mod catalog;
mod client;
pub mod config;
pub mod connectivity;
mod demo;
pub mod error;
pub mod events;
mod host_stream;
pub mod providers;
pub mod rpc;
pub mod runtime;
pub mod session;
pub mod subagents;
pub mod terminal;
pub mod workspace;

pub use client::{Client, NewSession, PRELOAD_CAP, PushPrefs, SessionTarget, WARM_SESSION_CAP};
pub use config::{ClientConfig, Credentials, DemoFixture, DemoOptions, StreamSpeed, TranscriptScale};
pub use connectivity::{Connectivity, ConnectivityState, SendState};
pub use error::ClientError;
pub use events::{ClientEvent, ClientListener};
pub use session::{
    AppendHint, BusyPolicy, ComposerState, Entry, HostCapabilities, HostInfo, InputRequest,
    LocalEcho, OutgoingAttachment, PendingKind, PendingSend, QueueEditAction, QueueEditFinish,
    QueueEditLease, QueueEditStart, QueueGate, QueueItem, SendOutcome, SendRequest, SessionHandle,
    SessionSnapshot, SnapshotDelta, SnapshotWatch,
};
pub use subagents::{SubagentInfo, SubagentSink, SubagentState, SubagentWatch};
pub use terminal::{TerminalInfo, TerminalLink, TerminalReplies, TerminalSink, TerminalUpdate};
pub use workspace::{
    DeviceView, FrontPage, ProjectRef, ProjectView, PullRequestGroups, SearchField, SearchHit,
    SectionView, SessionRow, WorkspaceSnapshot, project_color_index, relative_time_label,
};

/// Re-exported so consumers (the layout engine) name the exact doc types the
/// transcript carries without a direct `space-doc` dependency.
pub use space_doc::{MessagePart, MessageRole, MessageStatus, SessionMessageEntry};
pub use space_proto::{
    ChangeRequestState, ChangeRequestSummary, ChatConfig, ChatIndicator, ContextUsage,
    UserInputAnswer, UserInputQuestion, WorktreeSpec,
};

/// Wall clock in epoch millis.
pub(crate) fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

pub(crate) fn lock<T>(mutex: &std::sync::Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

pub(crate) fn read<T>(lock: &std::sync::RwLock<T>) -> std::sync::RwLockReadGuard<'_, T> {
    lock.read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

pub(crate) fn write<T>(lock: &std::sync::RwLock<T>) -> std::sync::RwLockWriteGuard<'_, T> {
    lock.write()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Lowercase v4 uuid (the id format every writer in the fleet mints).
pub(crate) fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}
