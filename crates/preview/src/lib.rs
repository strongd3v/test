//! Project-scoped HTTP discovery, stable local routing, and authenticated peers.
//! Application bytes use bounded multiplexed streams over local proxy routes
//! and direct WebRTC connections; previews stay local to this device's projects.
pub mod catalog;
pub mod discovery;
pub mod login;
pub mod mux;
pub mod peer;
pub mod proxy;
mod runtime;

pub mod service;
pub use service::{PreviewService, Scope};
