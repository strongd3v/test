//! One stream from the demo host. Dropping it ends the stream.

use serde_json::Value;

pub(crate) enum HostStream {
    Local(tokio::sync::mpsc::Receiver<Value>),
}

impl HostStream {
    /// The next item; `None` once the stream has ended.
    pub(crate) async fn recv(&mut self) -> Option<Value> {
        match self {
            Self::Local(rx) => rx.recv().await,
        }
    }
}
