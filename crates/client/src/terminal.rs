//! Terminals on a chat's host: the demo host's PTY sessions, followed as a
//! local stream. Output carries sequence numbers and a replay window, so a
//! viewer resuming after a drop gets exactly what it missed.

use std::sync::{Arc, Weak};
use std::time::Duration;

use serde_json::json;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use space_proto::TerminalEvent;
use space_rpc::methods;

use crate::b64;
use crate::client::{Client, ClientInner};
use crate::error::{ClientError, Result};

/// An open terminal and where it runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalInfo {
    pub id: String,
    pub device_id: String,
    pub cwd: String,
    /// Shell basename (`zsh`, `bash`, …).
    pub shell: String,
    /// The host answers the terminal's queries (device attributes, cursor
    /// position…) itself. A host from before that leaves them to the
    /// viewer, which then writes its emulator's answers back.
    pub host_replies: bool,
}

/// The host takes at most this much input per write (`MAX_INPUT_BYTES`).
const MAX_WRITE: usize = 64 * 1024;

/// What a link reports as it runs.
#[derive(Debug, Clone, PartialEq)]
pub enum TerminalUpdate {
    /// The output stream is (re)attached.
    Connected,
    /// PTY output, raw bytes (not necessarily valid UTF-8).
    Output(Vec<u8>),
    /// The shell exited; the stream is over.
    Exited { code: i32, signal: Option<String> },
    /// Every path dropped; the next attempt is `retry_in_ms` away (or
    /// sooner: the network coming back, the app back in front).
    Reconnecting { retry_in_ms: u64 },
    /// The host no longer has this terminal (it exited long ago, or the
    /// host restarted).
    Gone,
}

/// Receives a link's updates on the client runtime.
pub trait TerminalSink: Send + Sync {
    fn update(&self, update: TerminalUpdate);
}

/// One terminal's writes, forwarded to the host in the order they were typed.
enum Command {
    Input(Vec<u8>),
    Resize { cols: u16, rows: u16 },
}

/// One attached terminal. Dropping it detaches (the shell keeps running on
/// the host); [`TerminalLink::close`] ends it — the close goes out even if
/// the link is dropped right after.
pub struct TerminalLink {
    info: TerminalInfo,
    inner: Weak<ClientInner>,
    commands: mpsc::UnboundedSender<Command>,
    stream: CancellationToken,
}

/// Writes the terminal's own query answers back (hosts without host
/// replies). Doesn't keep the link alive.
#[derive(Clone)]
pub struct TerminalReplies {
    commands: mpsc::UnboundedSender<Command>,
}

fn queue(commands: &mpsc::UnboundedSender<Command>, bytes: Vec<u8>) {
    for chunk in bytes.chunks(MAX_WRITE).filter(|c| !c.is_empty()) {
        let _ = commands.send(Command::Input(chunk.to_vec()));
    }
}

impl TerminalReplies {
    pub fn write(&self, bytes: Vec<u8>) {
        queue(&self.commands, bytes);
    }
}

/// Forward this terminal's writes one at a time, in typed order.
async fn write_loop(
    weak: Weak<ClientInner>,
    device_id: String,
    terminal_id: String,
    cancel: CancellationToken,
    mut commands: mpsc::UnboundedReceiver<Command>,
) {
    loop {
        let command = tokio::select! {
            _ = cancel.cancelled() => return,
            command = commands.recv() => match command { Some(command) => command, None => return },
        };
        let Some(inner) = weak.upgrade() else { return };
        let (method, params) = match command {
            Command::Input(bytes) => (
                methods::WRITE_TERMINAL,
                json!({ "terminalId": terminal_id, "data": b64::encode(&bytes) }),
            ),
            Command::Resize { cols, rows } => (
                methods::RESIZE_TERMINAL,
                json!({ "terminalId": terminal_id, "cols": cols, "rows": rows }),
            ),
        };
        let _ = inner.host_rpc(&device_id, method, params).await;
    }
}

impl TerminalLink {
    pub fn info(&self) -> &TerminalInfo {
        &self.info
    }

    /// Bytes for the shell (typed keys, a paste). Queued in order.
    pub fn write(&self, bytes: Vec<u8>) {
        queue(&self.commands, bytes);
    }

    /// The viewport's grid size changed.
    pub fn resize(&self, cols: u16, rows: u16) {
        let _ = self.commands.send(Command::Resize {
            cols: cols.max(2),
            rows: rows.max(1),
        });
    }

    /// End the shell on the host. The stream reports the exit when it lands.
    pub fn close(&self) {
        let (inner, device_id, id) = (
            self.inner.clone(),
            self.info.device_id.clone(),
            self.info.id.clone(),
        );
        crate::runtime::shared().spawn(async move {
            let Some(inner) = inner.upgrade() else { return };
            let params = json!({ "terminalId": id });
            for attempt in 0..5u64 {
                if attempt > 0 {
                    tokio::time::sleep(Duration::from_millis(300 * attempt)).await;
                }
                match inner
                    .host_rpc(&device_id, methods::CLOSE_TERMINAL, params.clone())
                    .await
                {
                    Ok(_) | Err(ClientError::HostError(_) | ClientError::Unsupported(_)) => break,
                    Err(_) => {}
                }
            }
        });
    }

    /// For answering the terminal's queries when the host doesn't.
    pub fn replies(&self) -> TerminalReplies {
        TerminalReplies {
            commands: self.commands.clone(),
        }
    }
}

impl Drop for TerminalLink {
    /// Detach: the output stream stops (the shell keeps running on the host).
    fn drop(&mut self) {
        self.stream.cancel();
    }
}

impl Client {
    /// Open a shell for `chat_id` on its host, in the chat's working folder.
    pub async fn open_terminal(&self, chat_id: &str, cols: u16, rows: u16) -> Result<TerminalInfo> {
        let device_id = self.chat_host(chat_id)?;
        let reply = self
            .inner
            .host_rpc(
                &device_id,
                methods::OPEN_TERMINAL,
                json!({ "chatId": chat_id, "cols": cols.max(2), "rows": rows.max(1) }),
            )
            .await?;
        let session: space_proto::TerminalSession = serde_json::from_value(reply)
            .map_err(|e| ClientError::HostError(format!("OpenTerminal: {e}")))?;
        Ok(TerminalInfo {
            id: session.id,
            device_id,
            cwd: session.cwd,
            shell: session.shell,
            host_replies: session.host_replies,
        })
    }

    /// Attach to an open terminal: replays everything after `after_seq`
    /// (0 = the host's whole replay window), then follows live output.
    pub fn attach_terminal(
        &self,
        info: TerminalInfo,
        after_seq: u64,
        sink: Arc<dyn TerminalSink>,
    ) -> TerminalLink {
        let stream = CancellationToken::new();
        let (commands, incoming) = mpsc::unbounded_channel();
        let weak = Arc::downgrade(&self.inner);
        crate::runtime::shared().spawn(follow(
            weak.clone(),
            info.device_id.clone(),
            info.id.clone(),
            after_seq,
            sink,
            stream.clone(),
        ));
        crate::runtime::shared().spawn(write_loop(
            weak,
            info.device_id.clone(),
            info.id.clone(),
            stream.clone(),
            incoming,
        ));
        TerminalLink {
            info,
            inner: Arc::downgrade(&self.inner),
            commands,
            stream,
        }
    }

    /// The device hosting `chat_id`.
    pub(crate) fn chat_host(&self, chat_id: &str) -> Result<String> {
        self.inner
            .workspace
            .chat(chat_id)
            .map(|c| c.device_id)
            .ok_or_else(|| ClientError::NotFound(chat_id.to_owned()))
    }
}

/// Follow the terminal's output, resubscribing from the last sequence
/// whenever the stream drops (0.4s → 5s, cut short by the network coming
/// back or the app returning to the foreground).
async fn follow(
    weak: Weak<ClientInner>,
    device_id: String,
    terminal_id: String,
    mut last_seq: u64,
    sink: Arc<dyn TerminalSink>,
    cancel: CancellationToken,
) {
    let mut backoff = Duration::from_millis(400);
    loop {
        let Some(inner) = weak.upgrade() else { return };
        let subscribe = inner.host_subscribe(
            &device_id,
            methods::SUBSCRIBE_TERMINAL,
            json!({ "terminalId": terminal_id, "afterSeq": last_seq }),
        );
        let wake = inner.terminals_wake.clone();
        let subscribed = tokio::select! {
            _ = cancel.cancelled() => return,
            subscribed = subscribe => subscribed,
        };
        match subscribed {
            Ok(mut stream) => {
                sink.update(TerminalUpdate::Connected);
                loop {
                    tokio::select! {
                        _ = cancel.cancelled() => return,
                        item = stream.recv() => {
                            let Some(item) = item else { break };
                            backoff = Duration::from_millis(400);
                            match serde_json::from_value::<TerminalEvent>(item) {
                                Ok(TerminalEvent::Data { seq, data }) => {
                                    if seq <= last_seq {
                                        continue;
                                    }
                                    last_seq = seq;
                                    if let Some(bytes) = b64::decode(&data) {
                                        sink.update(TerminalUpdate::Output(bytes));
                                    }
                                }
                                Ok(TerminalEvent::Exit { exit_code, signal, .. }) => {
                                    sink.update(TerminalUpdate::Exited {
                                        code: exit_code,
                                        signal,
                                    });
                                    return;
                                }
                                Err(_) => {}
                            }
                        }
                    }
                }
            }
            Err(ClientError::HostError(message)) if message.contains("Terminal not found") => {
                sink.update(TerminalUpdate::Gone);
                return;
            }
            Err(ClientError::Unsupported(_)) => {
                sink.update(TerminalUpdate::Gone);
                return;
            }
            Err(_) => {}
        }
        sink.update(TerminalUpdate::Reconnecting {
            retry_in_ms: backoff.as_millis() as u64,
        });
        tokio::select! {
            _ = cancel.cancelled() => return,
            _ = tokio::time::sleep(backoff) => backoff = (backoff * 2).min(Duration::from_secs(5)),
            _ = wake.notified() => backoff = Duration::from_millis(400),
        }
    }
}
