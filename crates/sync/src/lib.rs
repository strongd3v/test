//! space-sync — local document storage and durable sync-job bookkeeping.
//!
//! - [`DocsStore`]: SQLite snapshot persistence (the doc IS the outbox — commands
//!   and user entries flush immediately) and the processed-command ledger with
//!   mark-BEFORE-execute semantics.
//! - [`budget`]: process-wide admission for data traffic.
//! - [`wake`]: suspend/resume and network-path event detection.

pub mod budget;
mod store;
mod sync_jobs;
mod types;
pub mod wake;

pub use store::{DocsStore, StoreError};
pub use types::SyncError;
