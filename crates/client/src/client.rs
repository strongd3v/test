//! [`Client`] — the demo-profile-scoped root object.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock, RwLock};
use std::time::Duration;

use chrono::{TimeZone, Utc};
use tokio_util::sync::CancellationToken;
use space_doc::RegistryDoc;
use space_proto::{Chat, ChatConfig, SidebarPinChange, SidebarSectionChange};

use crate::attachments::{self, AttachmentCache};
use crate::catalog::{HarnessInfo, ModelInfo};
use crate::config::{ClientConfig, Credentials};
use crate::connectivity::{
    Connectivity, ConnectivityState, ConnectivityTracker, RawConnectivity, SendState,
};
use crate::demo::DemoHost;
use crate::error::{ClientError, Result};
use crate::events::{ClientEvent, ClientListener, EventPump};
use crate::rpc::{FolderListing, ProgressFn, QUEUED_ATTACHMENTS_MIN, RepoRef, capability};
use crate::session::{HostCapabilities, OutgoingAttachment, SessionCore, SessionHandle};
use crate::workspace::{SearchHit, WorkspaceSnapshot, WorkspaceStore};
use crate::{lock, now_ms, read, write};

/// Attachment read cache budget.
const ATTACHMENT_CACHE_BYTES: usize = 48 * 1024 * 1024;
/// Time-driven re-derivation cadence (staleness, presence, send grace).
const TICK: Duration = Duration::from_secs(1);
/// Detached sessions kept warm before the least recently used is evicted.
/// On-screen sessions, streaming ones and ones with unadopted sends are never
/// evicted.
pub const WARM_SESSION_CAP: usize = 6;
/// Sessions `preload_sessions` warms (front page order).
pub const PRELOAD_CAP: usize = 4;

/// Where a new session runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionTarget {
    /// In a project: host = the project's owning device, cwd = its folder
    /// (or `cwd` override, e.g. a picked worktree).
    Project { space_id: String },
    /// No project: runs in the chosen host's home folder (`~`).
    Projectless { device_id: String },
}

#[derive(Debug, Clone, PartialEq)]
pub struct NewSession {
    pub target: SessionTarget,
    pub config: Option<ChatConfig>,
    /// Branch to stamp from the first frame (picked ref).
    pub branch: Option<String>,
    /// Checkout override (an existing worktree's path).
    pub cwd: Option<String>,
    pub title: Option<String>,
}

pub(crate) enum Backend {
    Demo(Arc<DemoHost>),
}

pub(crate) struct ClientInner {
    pub(crate) config: ClientConfig,
    pub(crate) credentials: Credentials,
    pub(crate) events: Arc<EventPump>,
    pub(crate) workspace: WorkspaceStore,
    pub(crate) attachment_cache: AttachmentCache,
    backend: OnceLock<Backend>,
    sessions: Mutex<HashMap<String, Arc<SessionCore>>>,
    pub(crate) cancel: CancellationToken,
    connectivity_tracker: Mutex<ConnectivityTracker>,
    connectivity: RwLock<Connectivity>,
    path_online: AtomicBool,
    foreground: AtomicBool,
    synced: AtomicBool,
    pub(crate) harness_catalogs: Mutex<HashMap<String, Vec<HarnessInfo>>>,
    /// Wakes terminal streams waiting to reconnect (foreground, network back).
    pub(crate) terminals_wake: Arc<tokio::sync::Notify>,
}

impl Drop for ClientInner {
    fn drop(&mut self) {
        // Released without `shutdown()`: stop every background task.
        self.cancel.cancel();
    }
}

impl ClientInner {
    pub(crate) fn backend(&self) -> &Backend {
        self.backend.get().expect("backend initialized")
    }

    pub(crate) fn demo(&self) -> &Arc<DemoHost> {
        match self.backend() {
            Backend::Demo(demo) => demo,
        }
    }

    pub(crate) fn is_demo(&self) -> bool {
        true
    }

    pub(crate) fn session_core(&self, chat_id: &str) -> Option<Arc<SessionCore>> {
        lock(&self.sessions).get(chat_id).cloned()
    }

    pub(crate) fn cores(&self) -> Vec<Arc<SessionCore>> {
        lock(&self.sessions).values().cloned().collect()
    }

    pub(crate) fn network_online(&self) -> bool {
        self.path_online.load(Ordering::Acquire)
    }

    /// Re-derive the workspace snapshot; on change, notify and refresh every
    /// open composer.
    pub(crate) fn recompute_workspace(self: &Arc<Self>) {
        let send_states: HashMap<String, SendState> = self
            .cores()
            .iter()
            .filter_map(|core| core.send_state().map(|s| (core.chat_id.clone(), s)))
            .collect();
        let synced = self.synced.load(Ordering::Acquire);
        if let Some(revision) =
            self.workspace
                .recompute(&self.config.device_id, &send_states, synced)
        {
            self.events.workspace(revision);
            for core in self.cores() {
                core.refresh();
            }
        }
    }

    /// After any local registry write, re-derive. The local-only replica
    /// already holds the write.
    pub(crate) fn after_registry_write(self: &Arc<Self>) {
        self.recompute_workspace();
    }

    pub(crate) fn registry_write<R>(
        self: &Arc<Self>,
        f: impl FnOnce(&mut RegistryDoc) -> std::result::Result<R, space_doc::DocError>,
    ) -> Result<R> {
        let result = self.workspace.mutate(f)?;
        self.after_registry_write();
        Ok(result)
    }

    pub(crate) fn mark_seen(self: &Arc<Self>, chat_id: &str) {
        let _ = self.registry_write(|doc| doc.set_chat_seen(chat_id, Utc::now()));
    }

    pub(crate) fn recompute_connectivity(self: &Arc<Self>) {
        let raw = RawConnectivity {
            path_offline: !self.network_online(),
        };
        let next = lock(&self.connectivity_tracker).compute(&raw, now_ms());
        let changed = {
            let mut current = write(&self.connectivity);
            if *current == next {
                false
            } else {
                *current = next.clone();
                true
            }
        };
        if changed {
            self.events.ordered(ClientEvent::ConnectivityChanged(next));
            for core in self.cores() {
                core.refresh();
            }
            if self.network_online() {
                self.demo().network_recovered();
            }
        }
    }

    pub(crate) fn connectivity(&self) -> Connectivity {
        read(&self.connectivity).clone()
    }

    /// Whether a send to this chat would queue rather than deliver promptly
    /// (legacy `chatDeliveryDegraded`): OS offline or the host presence-dark.
    pub(crate) fn chat_delivery_degraded(&self, chat_id: &str) -> bool {
        if self.connectivity().state == ConnectivityState::Offline {
            return true;
        }
        self.workspace
            .snapshot()
            .session(chat_id)
            .is_some_and(|row| !row.device_online)
    }

    pub(crate) fn host_capabilities(
        &self,
        device_id: &str,
        harness: Option<&str>,
    ) -> HostCapabilities {
        let workspace = self.workspace.snapshot();
        let Some(device) = workspace.device(device_id) else {
            return HostCapabilities::default();
        };
        let has = |cap: &str| device.capabilities.iter().any(|c| c == cap);
        let mid_turn_steering = harness.and_then(|harness| {
            lock(&self.harness_catalogs)
                .get(device_id)
                .and_then(|list| list.iter().find(|h| h.id == harness))
                .and_then(HarnessInfo::mid_turn_steering)
        });
        HostCapabilities {
            message_queue: has(capability::MESSAGE_QUEUE_V1),
            queue_actions: has(capability::MESSAGE_QUEUE_ACTIONS_V1),
            queue_attachments: has(capability::MESSAGE_QUEUE_ATTACHMENTS_V1),
            clean_attachment_text: has(capability::MESSAGE_QUEUE_CLEAN_ATTACHMENT_TEXT_V1),
            queue_edit_lease: has(capability::MESSAGE_QUEUE_EDIT_LEASE_V1),
            queued_attachments: device
                .version
                .as_deref()
                .and_then(space_proto::version_triple)
                .is_some_and(|v| v >= QUEUED_ATTACHMENTS_MIN),
            mid_turn_steering,
            provider_switch: has(capability::PROVIDER_SWITCH_V1),
        }
    }

    /// Mint `pending://` refs for picked images and keep their bytes local
    /// (the echo renders them immediately).
    pub(crate) fn stage_attachments(
        &self,
        _chat_id: &str,
        device_id: &str,
        attachments: &[OutgoingAttachment],
    ) -> Result<Vec<String>> {
        if let Some(too_big) = attachments
            .iter()
            .find(|a| a.data.len() > attachments::MAX_ATTACHMENT_BYTES)
        {
            return Err(ClientError::InvalidArgument(format!(
                "{} is larger than 24 MB",
                too_big.name
            )));
        }
        attachments
            .iter()
            .map(|attachment| {
                let upload_id = crate::new_id();
                let name = attachments::upload_file_name(&attachment.name);
                let reference = attachments::pending_ref(&upload_id, &name);
                self.attachment_cache
                    .put(device_id, &reference, Arc::new(attachment.data.clone()));
                Ok(reference)
            })
            .collect()
    }

    /// A command/queue row was written: wake the simulated host.
    pub(crate) fn after_command(
        self: &Arc<Self>,
        core: &Arc<SessionCore>,
        _has_attachments: bool,
    ) {
        self.demo().on_command(&core.chat_id);
    }

    pub(crate) async fn host_rpc(
        self: &Arc<Self>,
        device_id: &str,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        self.demo().host_rpc(device_id, method, params).await
    }

    /// A host stream (terminal output, a subagent's transcript).
    pub(crate) async fn host_subscribe(
        self: &Arc<Self>,
        device_id: &str,
        method: &str,
        params: serde_json::Value,
    ) -> Result<crate::host_stream::HostStream> {
        self.demo().host_subscribe(device_id, method, params).await
    }

    /// Drop the least recently used detached, quiet sessions past the cap.
    pub(crate) fn evict_sessions(&self) {
        let mut sessions = lock(&self.sessions);
        let mut idle: Vec<(i64, String)> = sessions
            .values()
            .filter(|core| {
                !core.view_attached() && !core.has_pending_sends() && !core.snapshot().streaming
            })
            .map(|core| (core.touched_ms(), core.chat_id.clone()))
            .collect();
        if idle.len() <= WARM_SESSION_CAP {
            return;
        }
        idle.sort();
        let excess = idle.len() - WARM_SESSION_CAP;
        for (_, chat_id) in idle.into_iter().take(excess) {
            tracing::debug!(chat = %chat_id, "evicting warm session");
            sessions.remove(&chat_id);
        }
    }

    fn tick(self: &Arc<Self>) {
        self.recompute_connectivity();
        self.recompute_workspace();
        for core in self.cores() {
            if core.has_pending_sends() {
                core.refresh();
            }
        }
    }
}

/// One signed-in account (or Demo mode). Cheap to clone.
#[derive(Clone)]
pub struct Client {
    pub(crate) inner: Arc<ClientInner>,
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("device_id", &self.inner.config.device_id)
            .field("demo", &self.inner.is_demo())
            .finish()
    }
}

impl Client {
    /// Dedicated ephemeral call handle; never uses durable commands or RPC retries.
    pub async fn voice_transport(
        &self,
        _device_id: &str,
    ) -> Result<Arc<space_voice_session::RpcTransport>> {
        Err(ClientError::Unsupported(
            "remote voice is unavailable in demo mode".into(),
        ))
    }
    /// Sign-out/shutdown closes every call created by this client.
    pub fn voice_cancellation(&self) -> CancellationToken {
        self.inner.cancel.child_token()
    }

    /// Build and start. The registry replica folds writes straight into its
    /// rows and the demo dataset is seeded before the first snapshot.
    pub fn new(
        config: ClientConfig,
        credentials: Credentials,
        listener: Arc<dyn ClientListener>,
    ) -> Result<Self> {
        if config.device_id.trim().is_empty() {
            return Err(ClientError::InvalidArgument("device_id is required".into()));
        }
        let Credentials::Demo(options) = &credentials;
        let options = options.clone();
        let events = EventPump::new(listener);
        let mut registry = RegistryDoc::new(config.device_id.clone());
        registry.set_local_only(true);
        let inner = Arc::new(ClientInner {
            workspace: WorkspaceStore::new(registry),
            events: events.clone(),
            attachment_cache: AttachmentCache::new(ATTACHMENT_CACHE_BYTES),
            backend: OnceLock::new(),
            sessions: Mutex::new(HashMap::new()),
            cancel: CancellationToken::new(),
            connectivity_tracker: Mutex::new(ConnectivityTracker::default()),
            connectivity: RwLock::new(Connectivity {
                state: ConnectivityState::Connected,
                ..Default::default()
            }),
            path_online: AtomicBool::new(true),
            foreground: AtomicBool::new(true),
            synced: AtomicBool::new(false),
            harness_catalogs: Mutex::new(HashMap::new()),
            terminals_wake: Arc::new(tokio::sync::Notify::new()),
            credentials,
            config,
        });
        let demo = DemoHost::new(&inner, options);
        demo.seed(&inner)?;
        inner.synced.store(true, Ordering::Release);
        let _ = inner.backend.set(Backend::Demo(demo));
        events.start(inner.cancel.clone());
        inner.recompute_workspace();
        inner.demo().start(&inner, inner.cancel.clone());
        let ticker = Arc::downgrade(&inner);
        let cancel = inner.cancel.clone();
        crate::runtime::shared().spawn(async move {
            let mut interval = tokio::time::interval(TICK);
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            loop {
                tokio::select! {
                    _ = cancel.cancelled() => return,
                    _ = interval.tick() => {}
                }
                let Some(inner) = ticker.upgrade() else {
                    return;
                };
                if inner.foreground.load(Ordering::Acquire) {
                    inner.tick();
                }
            }
        });
        Ok(Self { inner })
    }

    pub fn config(&self) -> &ClientConfig {
        &self.inner.config
    }

    pub fn is_demo(&self) -> bool {
        self.inner.is_demo()
    }

    pub fn device_id(&self) -> &str {
        &self.inner.config.device_id
    }

    pub fn user_id(&self) -> &str {
        self.inner.credentials.user_id()
    }

    pub fn org_id(&self) -> &str {
        self.inner.credentials.org_id()
    }

    /// Stop every background task (sign-out). Snapshots stay readable.
    pub fn shutdown(&self) {
        self.inner.demo().stop();
        self.inner.cancel.cancel();
        lock(&self.inner.sessions).clear();
        self.inner.attachment_cache.clear();
    }

    // ── workspace reads ────────────────────────────────────────────────────

    /// O(1): the current derived workspace.
    pub fn workspace(&self) -> Arc<WorkspaceSnapshot> {
        self.inner.workspace.snapshot()
    }

    pub fn connectivity(&self) -> Connectivity {
        self.inner.connectivity()
    }

    /// The chat's run configuration (harness/model/effort/options/sandbox).
    pub fn session_config(&self, chat_id: &str) -> Option<ChatConfig> {
        self.inner.workspace.chat(chat_id).and_then(|c| c.config)
    }

    pub fn search(&self, query: &str, limit: usize) -> Vec<SearchHit> {
        self.workspace().search(query, limit)
    }

    // ── workspace writes ───────────────────────────────────────────────────

    /// Mint a chat row (born in a local doc: `roomGen: 2`). The row is written
    /// before any session doc opens, so the host never reads it as legacy.
    pub fn create_session(&self, new: NewSession) -> Result<String> {
        let (state, _) = self.inner.workspace.state();
        let id = crate::new_id();
        let now = Utc::now();
        let (device_id, space_id, cwd) = match &new.target {
            SessionTarget::Project { space_id } => {
                let space = state
                    .spaces
                    .iter()
                    .find(|s| &s.id == space_id)
                    .ok_or_else(|| ClientError::NotFound(space_id.clone()))?;
                (
                    space.device_id.clone(),
                    Some(space.id.clone()),
                    new.cwd.clone().unwrap_or_else(|| space.path.clone()),
                )
            }
            SessionTarget::Projectless { device_id } => {
                let host = state
                    .devices
                    .iter()
                    .find(|d| &d.id == device_id)
                    .ok_or_else(|| ClientError::NotFound(device_id.clone()))?;
                if matches!(host.platform.as_str(), "ios" | "android" | "ipados") {
                    return Err(ClientError::InvalidArgument(
                        "that device can't host sessions".into(),
                    ));
                }
                // Its own folder on the host, created when the first run
                // spawns; a host that predates scratch folders runs it in `~`.
                let cwd = if host
                    .capabilities
                    .iter()
                    .any(|c| c == crate::rpc::capability::PROJECTLESS_SCRATCH_V1)
                {
                    space_proto::projectless::scratch_cwd(&id, now)
                } else {
                    "~".to_owned()
                };
                (device_id.clone(), None, cwd)
            }
        };
        let chat = Chat {
            id,
            device_id,
            title: new.title.filter(|t| !t.trim().is_empty()),
            archived: false,
            cwd: Some(cwd),
            branch: new.branch,
            checkout_id: None,
            source_context: None,
            config: new.config,
            last_message_preview: None,
            last_message_at: None,
            created_at: now,
            harness_session_id: None,
            harness_session_cwd: None,
            harness_session_harness: None,
            space_id,
            last_seen_at: None,
            room_gen: Some(2),
            parent_chat_id: None,
        };
        let id = chat.id.clone();
        self.inner.registry_write(|doc| doc.upsert_chat(&chat))?;
        Ok(id)
    }

    /// A fresh side chat under `parent_id`: same host, folder, checkout and
    /// agent settings as its parent, an empty conversation (the desktop's
    /// "New side chat"; agent-spawned side chats have the same shape).
    pub fn create_side_chat(&self, parent_id: &str) -> Result<String> {
        let parent = self
            .inner
            .workspace
            .chat(parent_id)
            .ok_or_else(|| ClientError::NotFound(parent_id.to_owned()))?;
        // One level deep: a side chat's own side chat hangs off the root.
        let root = parent.parent_chat_id.clone().unwrap_or_else(|| parent.id.clone());
        let mut chat = parent;
        chat.id = crate::new_id();
        chat.parent_chat_id = Some(root);
        chat.title = None;
        chat.archived = false;
        chat.created_at = Utc::now();
        chat.last_message_at = None;
        chat.last_message_preview = None;
        chat.last_seen_at = None;
        chat.harness_session_id = None;
        chat.harness_session_cwd = None;
        chat.harness_session_harness = None;
        chat.room_gen = Some(2);
        let id = chat.id.clone();
        self.inner.registry_write(|doc| doc.upsert_chat(&chat))?;
        Ok(id)
    }

    /// Fork `source_id` through its latest completed reply into a side chat
    /// (`ForkSideChat`, answered by the source's host, which copies the
    /// conversation). A side chat forks as a sibling, under the same parent.
    pub async fn fork_side_chat(&self, source_id: &str) -> Result<String> {
        let source = self
            .inner
            .workspace
            .chat(source_id)
            .ok_or_else(|| ClientError::NotFound(source_id.to_owned()))?;
        let parent = source.parent_chat_id.clone().unwrap_or_else(|| source.id.clone());
        let chat_id = crate::new_id();
        let reply = self
            .inner
            .host_rpc(
                &source.device_id,
                space_rpc::methods::FORK_SIDE_CHAT,
                serde_json::json!({
                    "chatId": chat_id,
                    "sourceChatId": source.id,
                    "parentChatId": parent,
                }),
            )
            .await?;
        // The host writes the fork's row; it reaches this device through
        // registry sync on its own time. Hold the reply's copy now so the
        // fork opens at once (the same row converges when sync lands).
        match serde_json::from_value::<Chat>(reply) {
            Ok(chat) => {
                if self.inner.workspace.chat(&chat.id).is_none() {
                    self.inner.registry_write(|doc| doc.upsert_chat(&chat))?;
                }
                Ok(chat.id)
            }
            Err(_) => Ok(chat_id),
        }
    }

    fn chat_write(
        &self,
        chat_id: &str,
        f: impl FnOnce(&mut RegistryDoc) -> std::result::Result<bool, space_doc::DocError>,
    ) -> Result<()> {
        if self.inner.registry_write(f)? {
            Ok(())
        } else {
            Err(ClientError::NotFound(chat_id.to_owned()))
        }
    }

    pub fn archive_session(&self, chat_id: &str) -> Result<()> {
        self.chat_write(chat_id, |doc| doc.set_chat_archived(chat_id, true))
    }

    pub fn unarchive_session(&self, chat_id: &str) -> Result<()> {
        self.chat_write(chat_id, |doc| doc.set_chat_archived(chat_id, false))
    }

    pub fn rename_session(&self, chat_id: &str, title: &str) -> Result<()> {
        self.chat_write(chat_id, |doc| doc.rename_chat(chat_id, title.trim()))
    }

    pub fn mark_seen(&self, chat_id: &str) {
        self.inner.mark_seen(chat_id);
    }

    pub fn set_session_config(&self, chat_id: &str, config: &ChatConfig) -> Result<()> {
        // A provider change needs a host that switches: an older one would
        // resume the chat's stored session under the new provider.
        if let Some(chat) = self.inner.workspace.chat(chat_id)
            && chat
                .config
                .as_ref()
                .is_some_and(|current| current.harness != config.harness)
            && !self
                .inner
                .host_capabilities(&chat.device_id, None)
                .provider_switch
        {
            return Err(ClientError::Unsupported(
                "This chat's computer needs a newer Space to switch provider.".into(),
            ));
        }
        self.chat_write(chat_id, |doc| doc.set_chat_config(chat_id, config))
    }

    /// Remove the chat row and drop its local session.
    pub fn delete_session(&self, chat_id: &str) -> Result<()> {
        self.chat_write(chat_id, |doc| doc.delete_chat(chat_id))?;
        self.forget_local_session(chat_id);
        Ok(())
    }

    /// Drop a deleted chat's open session.
    fn forget_local_session(&self, chat_id: &str) {
        lock(&self.inner.sessions).remove(chat_id);
    }

    /// Mid-session ref switch: retarget onto the ref's existing worktree (row
    /// writes only), else `git checkout` in the session's own folder.
    pub async fn switch_session_ref(&self, chat_id: &str, reference: RepoRef) -> Result<()> {
        let chat = self
            .inner
            .workspace
            .chat(chat_id)
            .ok_or_else(|| ClientError::NotFound(chat_id.to_owned()))?;
        let cwd = chat
            .cwd
            .clone()
            .ok_or_else(|| ClientError::InvalidArgument("session has no working folder".into()))?;
        if let Some(worktree) = reference.worktree_path.clone() {
            if worktree != cwd {
                self.inner.registry_write(|doc| {
                    doc.set_chat_cwd(chat_id, &worktree)?;
                    doc.set_chat_branch(chat_id, &reference.name)
                })?;
            }
            return Ok(());
        }
        self.switch_ref(&chat.device_id, &cwd, &reference.name)
            .await?;
        self.inner
            .registry_write(|doc| doc.set_chat_branch(chat_id, &reference.name))?;
        Ok(())
    }

    fn pin_change(&self, change: SidebarPinChange) -> Result<()> {
        if !self.workspace().pins_ready {
            return Err(ClientError::Unsupported("pins are not synced yet".into()));
        }
        self.inner
            .registry_write(|doc| doc.change_sidebar_pin(&change))
    }

    pub fn pin_session(&self, chat_id: &str) -> Result<()> {
        let after = self
            .inner
            .workspace
            .state()
            .1
            .and_then(|p| p.pinned_session_ids.last().cloned())
            .filter(|last| last != chat_id);
        self.pin_change(SidebarPinChange::Pin {
            session_id: chat_id.to_owned(),
            after,
            before: None,
        })
    }

    /// Pin between neighbours (either may be `None` at the ends).
    pub fn pin_session_at(
        &self,
        chat_id: &str,
        after: Option<String>,
        before: Option<String>,
    ) -> Result<()> {
        self.pin_change(SidebarPinChange::Pin {
            session_id: chat_id.to_owned(),
            after,
            before,
        })
    }

    pub fn unpin_session(&self, chat_id: &str) -> Result<()> {
        self.pin_change(SidebarPinChange::Unpin {
            session_id: chat_id.to_owned(),
        })
    }

    /// Reorder a pin between neighbours (either may be `None` at the ends).
    pub fn move_pin(
        &self,
        chat_id: &str,
        after: Option<String>,
        before: Option<String>,
    ) -> Result<()> {
        self.pin_change(SidebarPinChange::Move {
            session_id: chat_id.to_owned(),
            after,
            before,
        })
    }

    fn section_change(&self, change: SidebarSectionChange) -> Result<()> {
        self.pin_change(SidebarPinChange::Section { change })
    }

    pub fn create_section(&self, name: &str) -> Result<String> {
        let id = crate::new_id();
        self.section_change(SidebarSectionChange::Create {
            id: id.clone(),
            name: name.trim().to_owned(),
        })?;
        Ok(id)
    }

    pub fn rename_section(&self, section_id: &str, name: &str) -> Result<()> {
        self.section_change(SidebarSectionChange::Rename {
            id: section_id.to_owned(),
            name: name.trim().to_owned(),
        })
    }

    /// Delete a section (its sessions return to the recent list).
    pub fn delete_section(&self, section_id: &str) -> Result<()> {
        self.section_change(SidebarSectionChange::Delete {
            id: section_id.to_owned(),
        })
    }

    pub fn set_section_collapsed(&self, section_id: &str, collapsed: bool) -> Result<()> {
        self.section_change(SidebarSectionChange::Collapse {
            id: section_id.to_owned(),
            collapsed,
        })
    }

    /// Move a session into a section (`None` = back to the recent list).
    pub fn assign_section(&self, chat_id: &str, section_id: Option<String>) -> Result<()> {
        self.place_session(chat_id, section_id, None, None)
    }

    /// Move a session into a section between its members (either neighbour
    /// may be `None` at the ends; both `None` appends).
    pub fn place_session(
        &self,
        chat_id: &str,
        section_id: Option<String>,
        after: Option<String>,
        before: Option<String>,
    ) -> Result<()> {
        self.section_change(SidebarSectionChange::Assign {
            session_id: chat_id.to_owned(),
            section_id,
            after,
            before,
        })
    }

    /// Create a project on `device_id` (deduped on device + path).
    pub async fn create_project(
        &self,
        device_id: &str,
        path: &str,
        git_detected: bool,
    ) -> Result<String> {
        let (state, _) = self.inner.workspace.state();
        if let Some(existing) = state
            .spaces
            .iter()
            .find(|s| s.device_id == device_id && s.path == path)
        {
            return Ok(existing.id.clone());
        }
        let space = space_proto::Space {
            id: crate::new_id(),
            device_id: device_id.to_owned(),
            path: path.to_owned(),
            name: None,
            git_detected,
            git_checked_at: None,
            checkout_id: None,
            repository_id: None,
            github_repo: None,
            created_at: Utc::now(),
        };
        let id = space.id.clone();
        self.inner.registry_write(|doc| doc.upsert_space(&space))?;
        Ok(id)
    }

    pub fn rename_project(&self, space_id: &str, name: Option<&str>) -> Result<()> {
        let name = name.map(str::trim).filter(|n| !n.is_empty());
        if self
            .inner
            .registry_write(|doc| doc.rename_space(space_id, name))?
        {
            Ok(())
        } else {
            Err(ClientError::NotFound(space_id.to_owned()))
        }
    }

    /// Delete a project and cascade its chats' rows.
    pub fn delete_project(&self, space_id: &str) -> Result<()> {
        let deleted = self.inner.registry_write(|doc| doc.delete_space(space_id))?;
        let mut sessions = lock(&self.inner.sessions);
        for chat_id in deleted.chat_ids {
            sessions.remove(&chat_id);
        }
        Ok(())
    }

    // ── sessions ───────────────────────────────────────────────────────────

    /// Open (or return the already-open) session. Instant: the local
    /// snapshot renders first.
    pub fn open_session(&self, chat_id: &str) -> Result<SessionHandle> {
        if let Some(core) = self.inner.session_core(chat_id) {
            return Ok(SessionHandle { core });
        }
        if self.inner.workspace.chat(chat_id).is_none() {
            return Err(ClientError::NotFound(chat_id.to_owned()));
        }
        let doc = self.inner.demo().session_doc(chat_id)?;
        let core = SessionCore::new(chat_id, &self.inner, doc);
        let core = {
            let mut sessions = lock(&self.inner.sessions);
            sessions
                .entry(chat_id.to_owned())
                .or_insert_with(|| core.clone())
                .clone()
        };
        core.set_hydrated();
        core.touch();
        core.refresh();
        self.inner.demo().session_opened(&core);
        self.inner.evict_sessions();
        Ok(SessionHandle { core })
    }

    /// An already-open session, if any.
    pub fn session(&self, chat_id: &str) -> Option<SessionHandle> {
        self.inner
            .session_core(chat_id)
            .map(|core| SessionHandle { core })
    }

    /// The view closed. The session stays warm (streaming continues) until
    /// the eviction policy reclaims it.
    pub fn close_session(&self, chat_id: &str) {
        if let Some(handle) = self.session(chat_id) {
            handle.set_view_attached(false);
        }
        self.inner.evict_sessions();
    }

    // ── host RPC ───────────────────────────────────────────────────────────

    /// Harness catalog of an execution device. Cached in memory for
    /// capability gating (mid-turn steering).
    pub async fn list_harnesses(&self, device_id: &str) -> Vec<HarnessInfo> {
        let list = crate::providers::with_demo_toggles(
            self.inner.demo().list_harnesses(device_id).await,
        );
        lock(&self.inner.harness_catalogs).insert(device_id.to_owned(), list.clone());
        self.inner.recompute_workspace();
        for core in self.inner.cores() {
            core.recompute_composer(&self.inner);
        }
        list
    }

    /// Model catalog for `harness` on `device_id`.
    pub async fn list_models(&self, _device_id: &str, harness: &str) -> Vec<ModelInfo> {
        self.inner.demo().list_models(harness).await
    }

    // ── push notifications ────────────────────────────────────────────────

    /// Push registration is a no-op in demo mode.
    pub async fn register_push_target(
        &self,
        _token: &str,
        _environment: &str,
        _prefs: PushPrefs,
    ) -> Result<()> {
        Ok(())
    }

    /// Push unregistration is a no-op in demo mode.
    pub async fn unregister_push_target(&self) -> Result<()> {
        Ok(())
    }

    pub async fn list_refs(&self, _device_id: &str, repo_path: &str) -> Result<Vec<RepoRef>> {
        self.inner.demo().list_refs(repo_path).await
    }

    /// Workspace file search for composer `@` mentions (host `SearchFiles`,
    /// gitignore-aware; an empty query returns the chat's featured files).
    pub async fn search_files(
        &self,
        _device_id: &str,
        _chat_id: Option<String>,
        _space_id: Option<String>,
        query: &str,
    ) -> Result<Vec<space_proto::FileSearchMatch>> {
        const FILES: &[&str] = &[
            "crates/mobile/src/layout/mod.rs",
            "crates/mobile/src/layout/markdown.rs",
            "crates/mobile/src/layout/rows.rs",
            "crates/text/src/layout.rs",
            "crates/text/src/prepare.rs",
            "crates/markdown/src/parser.rs",
            "apps/ios/Space/Transcript/TranscriptListView.swift",
            "apps/ios/Space/Composer/ComposerBar.swift",
            "docs/mobile-rewrite.md",
            "README.md",
        ];
        let q = query.to_lowercase();
        Ok(FILES
            .iter()
            .filter(|p| q.is_empty() || p.to_lowercase().contains(&q))
            .map(|p| space_proto::FileSearchMatch {
                path: (*p).to_owned(),
                is_dir: false,
            })
            .collect())
    }

    /// Browse folders on a device (`None` = its home folder).
    pub async fn list_folders(
        &self,
        device_id: &str,
        path: Option<String>,
    ) -> Result<FolderListing> {
        self.inner.demo().list_folders(device_id, path).await
    }

    /// `git checkout <ref>` in `repo_path` on the device.
    pub async fn switch_ref(
        &self,
        _device_id: &str,
        repo_path: &str,
        ref_name: &str,
    ) -> Result<()> {
        self.inner.demo().switch_ref(repo_path, ref_name).await
    }

    /// Create a worktree for `branch` (an existing ref); returns its path.
    pub async fn create_worktree(
        &self,
        _device_id: &str,
        _space_id: &str,
        repo_path: &str,
        branch: &str,
    ) -> Result<String> {
        self.inner.demo().create_worktree(repo_path, branch).await
    }

    /// Chunked upload of one file to the host; returns its durable path there.
    pub async fn upload_attachment(
        &self,
        device_id: &str,
        name: &str,
        data: Vec<u8>,
        progress: Option<ProgressFn>,
    ) -> Result<String> {
        if data.len() > attachments::MAX_ATTACHMENT_BYTES {
            return Err(ClientError::InvalidArgument(format!(
                "{name} is larger than 24 MB"
            )));
        }
        self.inner
            .demo()
            .upload(&self.inner, device_id, name, data, progress)
            .await
    }

    /// Attachment bytes (LRU-cached). Accepts host paths and this device's
    /// own `pending://` refs.
    pub async fn read_attachment(&self, device_id: &str, path: &str) -> Result<Arc<Vec<u8>>> {
        if let Some(hit) = self.inner.attachment_cache.get(device_id, path) {
            return Ok(hit);
        }
        let bytes = self.inner.demo().read_attachment(path)?;
        self.inner
            .attachment_cache
            .put(device_id, path, bytes.clone());
        Ok(bytes)
    }

    // ── lifecycle ──────────────────────────────────────────────────────────

    /// OS network path (NWPathMonitor): `false` only for a definitive
    /// "unsatisfied" — ambiguity stays online.
    pub fn set_network_online(&self, online: bool) {
        let was = self.inner.path_online.swap(online, Ordering::AcqRel);
        if was != online {
            self.inner.recompute_connectivity();
            if online {
                self.network_path_changed();
            }
        }
    }

    /// The OS network path changed (another interface, a new address).
    pub fn network_path_changed(&self) {
        self.inner.terminals_wake.notify_waiters();
    }

    /// App returned to the foreground: resume ticking.
    pub fn on_foreground(&self) {
        self.inner.foreground.store(true, Ordering::Release);
        self.inner.terminals_wake.notify_waiters();
        self.inner.tick();
    }

    /// App is backgrounding: pause time-driven work.
    pub fn on_background(&self) {
        self.inner.foreground.store(false, Ordering::Release);
    }

    /// Warm the most relevant sessions (front page order: pinned, sections,
    /// recent), up to [`PRELOAD_CAP`]. Opening is instant (local snapshot).
    pub fn preload_sessions(&self) {
        let workspace = self.workspace();
        let front = &workspace.front;
        let candidates = front
            .pinned
            .iter()
            .chain(front.sections.iter().flat_map(|s| s.sessions.iter()))
            .chain(front.recent.iter())
            .filter(|row| row.room_gen >= 2);
        for row in candidates.take(PRELOAD_CAP) {
            if self.inner.session_core(&row.id).is_none() {
                let _ = self.open_session(&row.id);
            }
        }
    }

    /// Ids of the sessions currently open (warm), most recently used first.
    pub fn open_session_ids(&self) -> Vec<String> {
        let mut cores = self.inner.cores();
        cores.sort_by_key(|c| std::cmp::Reverse(c.touched_ms()));
        cores.iter().map(|c| c.chat_id.clone()).collect()
    }
}

/// Seed helpers shared with the demo host.
pub(crate) fn ms(at: i64) -> chrono::DateTime<Utc> {
    Utc.timestamp_millis_opt(at)
        .single()
        .unwrap_or_else(Utc::now)
}

/// Which session notifications a device wants (the desktop's three).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PushPrefs {
    /// A run finished.
    pub done: bool,
    /// A session is waiting on you (question / permission).
    pub input: bool,
    /// A run failed.
    pub failed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_replica_is_local_only_and_never_queues() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = ClientConfig::new(dir.path());
        config.device_id = "ios-test".into();
        let client = Client::new(
            config,
            Credentials::Demo(Default::default()),
            Arc::new(crate::events::NullListener),
        )
        .unwrap();
        let chat_id = client
            .inner
            .workspace
            .mutate(|doc| doc.read_chats())
            .unwrap()
            .into_iter()
            .find(|c| c.parent_chat_id.is_none())
            .expect("seeded chat")
            .id;
        for i in 0..20 {
            client
                .rename_session(&chat_id, &format!("title {i}"))
                .unwrap();
        }
        let (pending, title) = client.inner.workspace.mutate(|doc| {
            (
                doc.pending_len(),
                doc.chat(&chat_id).unwrap().unwrap().title,
            )
        });
        assert_eq!(pending, 0);
        assert_eq!(title.as_deref(), Some("title 19"));
    }
}
