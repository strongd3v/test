//! Settings → Providers: install and enable harnesses, one card row per agent.
//!
//! The state lives on the engine (`harness-prefs.json` in its data dir):
//! CLI installs are local, so enablement is too.
//!
//! Enablement follows DETECTION: every harness whose CLI probe passes is on
//! unless the user switched it off, so installing an agent is all it takes
//! for it to appear here and in the composer. A harness whose CLI is missing
//! renders dimmed with an install hint and is never
//! enabled (enabling an agent that can't run would only manufacture
//! NotInstalled errors at send time); an ENABLED agent can always be turned
//! OFF except the last one standing — the composer needs something to run.
//! Catalogs from engines predating the detection model can still stamp
//! enabled-but-uninstalled rows; the hint covers that too. The engine
//! enforces the same gates where the state lives, so a raced or stale toggle
//! self-corrects from the RPC reply.

use gpui::{
    AnyElement, Context, Entity, IntoElement, Render, SharedString, Task, Window, div, prelude::*,
    px,
};

use space_engine::registry::{HarnessDescriptor, descriptor_enabled};

use space_proto::{
    HarnessId, HarnessUpdateFailureKind, HarnessUpdatePhase, HarnessUpdatePolicy,
    HarnessUpdateStatus,
};

use space_rpc::methods;

use crate::motion;
use crate::pickers::visible_harnesses;
use crate::popover::{self, Loadable};
use crate::settings::accounts::{self, AccountsPage};
use crate::settings::search::{SearchAnchor, SettingId, Target};
use crate::settings::widgets;
use crate::shell::SettingsSection;
use crate::state::AppState;
use crate::theme::Theme;

#[path = "completion.rs"]
mod completion;
#[path = "delegation.rs"]
mod delegation;

/// Left inset of an expanded provider's details: the header trigger's
/// padding, brand tile and gap, so the details start on the title's edge.
const DETAILS_INSET: f32 = 4.0 + 36.0 + 12.0;

/// Update policies in menu order: the short label, and the note shown under
/// the row title for the chosen one.
const UPDATE_POLICIES: [(HarnessUpdatePolicy, &str, &str); 3] = [
    (
        HarnessUpdatePolicy::Notify,
        "Notify",
        "Install only when you choose Update.",
    ),
    (
        HarnessUpdatePolicy::AutoWhenIdle,
        "Auto when idle",
        "Install automatically after active runs finish.",
    ),
    (
        HarnessUpdatePolicy::Off,
        "Off",
        "Don't check for new versions.",
    ),
];

fn offers_install(harness: HarnessId, installed: bool, can_install: bool) -> bool {
    harness != HarnessId::Mock && !installed && can_install
}

fn install_hint(harness: HarnessId, enabled: bool, can_install: bool) -> String {
    if harness == HarnessId::Antigravity {
        return if can_install {
            "Install Antigravity to enable"
        } else {
            "Set ANTIGRAVITY_ACP_EXECUTABLE to enable Antigravity"
        }
        .into();
    }
    let hint = if enabled {
        format!(
            "{} CLI not installed — turn it off or install it",
            cli_name(harness)
        )
    } else {
        format!("Install the {} CLI to enable", cli_name(harness))
    };
    if !can_install && let Some(command) = space_harness::install::manual_command(harness) {
        format!("{hint}. Install with `{command}`")
    } else {
        hint
    }
}

/// "1 account", "3 accounts".
fn account_count_label(count: usize) -> String {
    if count == 1 {
        "1 account".into()
    } else {
        format!("{count} accounts")
    }
}

fn install_label(name: &str) -> String {
    format!("Installing {name}…")
}

fn install_params(harness: HarnessId) -> serde_json::Value {
    serde_json::json!({"harness": harness})
}

/// The CLI named in the not-installed hint.
pub fn cli_name(harness: HarnessId) -> &'static str {
    match harness {
        HarnessId::ClaudeCode => "claude",
        HarnessId::Codex => "codex",
        HarnessId::Cursor => "cursor-agent",
        HarnessId::Devin => "devin",
        HarnessId::Grok => "grok",
        HarnessId::Hermes => "hermes",
        HarnessId::Fx => "fx",
        HarnessId::Pi => "pi",
        HarnessId::Opencode => "opencode",
        HarnessId::Antigravity => "Antigravity",
        HarnessId::Muse => "muse",
        HarnessId::Mock => "mock",
    }
}

/// The failure was the update itself (its release is still known, so it can
/// be retried), not a failed check.
fn install_failed(status: &HarnessUpdateStatus) -> bool {
    status
        .error
        .as_ref()
        .is_some_and(|error| error.kind == HarnessUpdateFailureKind::Install)
}

/// A provider's version line: the installed version, plus only what the
/// Update button doesn't already say (progress, a failure), all in one muted
/// tone. Up-to-date providers, checks in flight and providers whose updates
/// aren't monitored show just the version, so rows don't flash through
/// states while checks run.
fn harness_update_label(status: &HarnessUpdateStatus, theme: &Theme) -> (SharedString, gpui::Hsla) {
    let installed = status
        .installed_version
        .as_deref()
        .map(|version| format!("v{version}"));
    let with = |detail: &str| match &installed {
        Some(version) => format!("{version} · {detail}"),
        None => detail.to_owned(),
    };
    let version_only = || installed.clone().unwrap_or_default();
    // An installed release this Space can't run explains failing runs: it
    // outranks everything but an update in flight.
    if let Some(compatibility) = &status.compatibility
        && matches!(
            status.phase,
            HarnessUpdatePhase::Dormant
                | HarnessUpdatePhase::Checking
                | HarnessUpdatePhase::Current
                | HarnessUpdatePhase::Available
                | HarnessUpdatePhase::ManualActionRequired
                | HarnessUpdatePhase::Failed
        )
    {
        return (compatibility.message.clone().into(), theme.danger_muted);
    }
    if status.phase == HarnessUpdatePhase::Current
        && let Some(held) = &status.held_back
    {
        return (
            with(&format!("v{} held back: not supported yet", held.version)).into(),
            theme.text_muted.opacity(0.75),
        );
    }
    let label = match status.phase {
        HarnessUpdatePhase::Dormant
        | HarnessUpdatePhase::Checking
        | HarnessUpdatePhase::Current => version_only(),
        // The row's Update button already says it; one Space can only
        // describe (a command to run yourself) is just the version too.
        HarnessUpdatePhase::Available | HarnessUpdatePhase::ManualActionRequired
            if status.can_apply =>
        {
            version_only()
        }
        HarnessUpdatePhase::Available => match status.latest_version.as_deref() {
            Some(latest) => with(&format!("v{latest} available")),
            None => with("Update available"),
        },
        HarnessUpdatePhase::ManualActionRequired => version_only(),
        HarnessUpdatePhase::WaitingForIdle => with("Waiting for agent to be idle"),
        HarnessUpdatePhase::Preparing => with("Preparing update…"),
        HarnessUpdatePhase::Downloading => with("Downloading…"),
        HarnessUpdatePhase::Installing => with("Installing…"),
        HarnessUpdatePhase::Verifying => with("Verifying…"),
        HarnessUpdatePhase::Updated => with("Updated"),
        HarnessUpdatePhase::Failed if install_failed(status) => with("Update failed"),
        HarnessUpdatePhase::Failed => with("Couldn't check for updates"),
    };
    let color = match status.phase {
        HarnessUpdatePhase::Failed => theme.danger_muted,
        _ => theme.text_muted.opacity(0.75),
    };
    (label.into(), color)
}

pub struct HarnessesPage {
    state: Entity<AppState>,
    scroll: widgets::PageScroll,
    harnesses: Loadable<Vec<HarnessDescriptor>>,
    updates: Loadable<Vec<HarnessUpdateStatus>>,
    policy_selects: std::collections::HashMap<HarnessId, widgets::SelectState>,
    /// Last refused/failed toggle (engine guards), shown in the error strip.
    error: Option<String>,
    load_task: Option<Task<()>>,
    toggle_task: Option<Task<()>>,
    /// A toggle the engine hasn't answered yet: the switch shows it at once.
    pending_toggle: Option<(HarnessId, bool)>,
    installing: Option<HarnessId>,
    install_task: Option<Task<()>>,

    expanded_harness: Option<HarnessId>,
    /// A transcript's "Log in": open this provider and start its sign-in
    /// once the rows land.
    pending_sign_in: Option<HarnessId>,
    /// The expanded provider's Accounts section — one page, retargeted as
    /// providers expand, so every provider shares the same sign-in flow.
    accounts_page: Option<Entity<AccountsPage>>,
    /// Repaints the rows' account counts as the open Accounts section adds
    /// or forgets one.
    _observe_accounts: Option<gpui::Subscription>,
    update_task: Option<Task<()>>,
    update_action_task: Option<Task<()>>,
    /// "Check now" is running: the button says so, since rows no longer
    /// show "Checking…" while a check is in flight.
    checking_updates: bool,
    /// Just the rows, in a scroll of their own: first-run setup's agents
    /// step shows this device's providers exactly as Settings does.
    embedded: bool,
    /// Native subagent preferences; an error (an engine predating them, say)
    /// is what the Delegation section shows instead.
    native_subagents: Loadable<space_engine::registry::NativeSubagentSettings>,
    native_subagents_task: Option<Task<()>>,
    native_subagents_toggle: Option<Task<()>>,
    native_subagents_saving: bool,
}

impl HarnessesPage {
    pub fn new(state: Entity<AppState>, cx: &mut Context<Self>) -> Self {
        let mut page = Self {
            state,
            scroll: widgets::PageScroll::default(),
            harnesses: Loadable::Idle,
            updates: Loadable::Idle,
            policy_selects: Default::default(),
            error: None,
            load_task: None,
            toggle_task: None,
            pending_toggle: None,
            installing: None,
            install_task: None,

            expanded_harness: None,
            pending_sign_in: None,
            accounts_page: None,
            _observe_accounts: None,
            update_task: None,
            update_action_task: None,
            checking_updates: false,
            embedded: false,
            native_subagents: Loadable::Idle,
            native_subagents_task: None,
            native_subagents_toggle: None,
            native_subagents_saving: false,
        };
        page.load(cx);
        page
    }

    /// This device's rows alone, for a host that brings its own heading.
    pub fn new_embedded(state: Entity<AppState>, cx: &mut Context<Self>) -> Self {
        let mut page = Self::new(state, cx);
        page.embedded = true;
        page
    }

    pub(crate) fn harnesses(&self) -> &Loadable<Vec<HarnessDescriptor>> {
        &self.harnesses
    }

    /// Re-probe the CLIs, keeping painted rows until the answer lands.
    pub(crate) fn reload(&mut self, cx: &mut Context<Self>) {
        let painted = matches!(self.harnesses, Loadable::Ready(_))
            .then(|| std::mem::replace(&mut self.harnesses, Loadable::Idle));
        let painted_updates = matches!(self.updates, Loadable::Ready(_))
            .then(|| std::mem::replace(&mut self.updates, Loadable::Idle));
        self.load(cx);
        if let Some(painted) = painted {
            self.harnesses = painted;
        }
        if let Some(painted) = painted_updates.filter(|_| matches!(self.updates, Loadable::Loading))
        {
            self.updates = painted;
        }
        cx.notify();
    }

    /// The refresh button: look for newly installed CLIs and check every
    /// agent for updates, the rows staying painted meanwhile.
    fn refresh(&mut self, cx: &mut Context<Self>) {
        self.reload(cx);
        self.check_updates(cx);
    }

    /// The page header's refresh control. First-run setup shows the same
    /// control over its rows.
    pub(crate) fn render_toolbar(&mut self, theme: &Theme, cx: &mut Context<Self>) -> AnyElement {
        let busy = self.checking_updates || matches!(self.harnesses, Loadable::Loading);
        let refresh = div()
            .id("harnesses-refresh")
            .flex_none()
            .size(px(28.0))
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(6.0))
            .tab_index(0)
            .role(gpui::Role::Button)
            .aria_label("Check for agents and updates")
            .tooltip(widgets::text_tooltip("Check for agents and updates"))
            .focus_visible(|s| s.border_2().border_color(theme.accent))
            .cursor_pointer()
            .hover(|style| style.bg(crate::theme::ink(0.05)))
            .on_click(cx.listener(|this, _, _, cx| this.refresh(cx)))
            .child(if busy {
                crate::loaders::ring_spinner("harnesses-refresh-spinner", 12.0, theme.text_muted)
                    .into_any_element()
            } else {
                crate::icons::icon(crate::icons::REFRESH)
                    .size(px(16.0))
                    .text_color(theme.text_muted)
                    .into_any_element()
            });
        div().child(refresh).into_any_element()
    }

    fn toggle_agent_details(&mut self, harness: HarnessId, cx: &mut Context<Self>) {
        if !self.harnesses.ready().is_some_and(|items| {
            items
                .iter()
                .any(|item| item.id == harness && descriptor_enabled(item))
        }) {
            return;
        }
        if self.expanded_harness == Some(harness) {
            self.expanded_harness = None;
        } else {
            self.expanded_harness = Some(harness);
            if accounts::signs_in(harness) {
                if let Some(accounts) = &self.accounts_page {
                    accounts.update(cx, |page, cx| page.set_embedded_harness(harness, cx));
                } else {
                    let state = self.state.clone();
                    let page = cx.new(|cx| AccountsPage::new_embedded(state, harness, cx));
                    self._observe_accounts = Some(cx.observe(&page, |_, _, cx| cx.notify()));
                    self.accounts_page = Some(page);
                }
            }
        }
        cx.notify();
    }

    /// A failed run's "Log in": show `harness` open and start its first
    /// sign-in option.
    pub(crate) fn sign_in(&mut self, harness: HarnessId, cx: &mut Context<Self>) {
        self.pending_sign_in = Some(harness);
        self.run_pending_sign_in(cx);
    }

    fn run_pending_sign_in(&mut self, cx: &mut Context<Self>) {
        let Some(harness) = self.pending_sign_in else {
            return;
        };
        if !matches!(self.harnesses, Loadable::Ready(_)) {
            return;
        }
        self.pending_sign_in = None;
        if self.expanded_harness != Some(harness) {
            self.toggle_agent_details(harness, cx);
        }
        if self.expanded_harness != Some(harness) {
            return;
        }
        if let Some(accounts) = &self.accounts_page {
            accounts.update(cx, |page, cx| page.start_first_login(harness, cx));
        }
    }

    fn render_agent_details(
        &self,
        harness: HarnessId,
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let accounts = accounts::signs_in(harness)
            .then(|| self.accounts_page.clone())
            .flatten()
            .map(|page| page.into_any_element());
        // No box of its own: the details continue the provider row on the
        // block's fill, indented to the row's title so they read as its
        // children, with switches and actions on the row's control edge.
        let content = div()
            .mx(px(16.0))
            .pl(px(DETAILS_INSET))
            .pb(px(16.0))
            .flex()
            .flex_col()
            .gap(px(20.0))
            .child(self.render_completion_for(harness, theme, cx))
            .children(self.render_native_subagents_for(harness, theme, cx))
            .children(self.render_updates_for(harness, theme, cx))
            .when_some(accounts, |details, accounts| details.child(accounts));
        if motion::reduced_motion(cx) {
            content.into_any_element()
        } else {
            motion::menu_in(format!("agent-details-{harness:?}"), content).into_any_element()
        }
    }

    /// The expanded provider's Updates section: the update policy, short
    /// labels in the menu and the chosen one explained under the title.
    fn render_updates_for(
        &self,
        harness: HarnessId,
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let Loadable::Ready(statuses) = &self.updates else {
            return None;
        };
        let status = statuses.iter().find(|status| status.harness == harness)?;
        let selected = UPDATE_POLICIES
            .iter()
            .position(|(policy, ..)| *policy == status.policy)
            .unwrap_or(0);
        let closed = widgets::SelectState::default();
        let policy_select = widgets::select(
            format!("harness-update-policy-{harness:?}"),
            "Update policy",
            theme,
            move |page: &mut Self| page.policy_selects.entry(harness).or_default(),
        )
        .options(
            UPDATE_POLICIES
                .iter()
                .map(|(_, label, _)| widgets::SelectOption::new(*label)),
            selected,
        )
        // Fits the longest label, so switching never resizes the trigger.
        .width(136.0)
        .on_select(move |page, selected, _, cx| {
            if let Some((policy, ..)) = UPDATE_POLICIES.get(selected) {
                page.set_update_policy(harness, *policy, cx);
            }
        })
        .render(self.policy_selects.get(&harness).unwrap_or(&closed), cx);
        let row = div()
            .min_h(px(52.0))
            .py(px(10.0))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(16.0))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .child(widgets::row_title(theme, "Update policy"))
                    .child(widgets::meta_line(
                        theme,
                        vec![div().child(UPDATE_POLICIES[selected].2).into_any_element()],
                    )),
            )
            .child(policy_select);
        Some(
            div()
                .flex()
                .flex_col()
                .child(widgets::details_label(theme, "Updates"))
                .child(row.search_anchor(SettingId::UpdatePolicy))
                .into_any_element(),
        )
    }

    /// Whether a dialog [`Self::dismiss_on_escape`] would close is open.
    pub(crate) fn has_escape_surface(&self, cx: &gpui::App) -> bool {
        self.expanded_harness.is_some()
            && self
                .accounts_page
                .as_ref()
                .is_some_and(|accounts| accounts.read(cx).has_escape_surface())
    }

    /// `harness` as the only provider, expanded, with every section its
    /// details can show loaded: what the settings search must find there.
    #[cfg(test)]
    pub(crate) fn show_provider_details(&mut self, harness: HarnessId, cx: &mut Context<Self>) {
        self.harnesses = Loadable::Ready(vec![HarnessDescriptor {
            id: harness,
            name: format!("{harness:?}"),
            supports_steering: false,
            steering_mode: space_proto::SteeringMode::TurnBoundary,
            reasoning_levels: Vec::new(),
            installed: true,
            can_install: false,
            enabled: Some(true),
            access_modes: Vec::new(),
            supports_plan: false,
        }]);
        self.updates = Loadable::Ready(vec![
            serde_json::from_value(serde_json::json!({ "harness": harness, "phase": "current" }))
                .unwrap(),
        ]);
        self.native_subagents = Loadable::Ready(space_engine::registry::NativeSubagentSettings {
            switchable: vec![harness],
            off: Vec::new(),
        });
        self.expanded_harness = None;
        self.toggle_agent_details(harness, cx);
    }

    /// Escape that reached Settings unclaimed goes to the expanded provider's
    /// accounts (an open login) first. Returns whether it was consumed.
    pub(crate) fn dismiss_on_escape(&mut self, cx: &mut Context<Self>) -> bool {
        self.expanded_harness.is_some()
            && self
                .accounts_page
                .as_ref()
                .is_some_and(|accounts| accounts.update(cx, |page, cx| page.dismiss_on_escape(cx)))
    }

    fn supports_updates(&self, cx: &Context<Self>) -> bool {
        let Some(engine) = self.state.read(cx).engine() else {
            return false;
        };
        engine
            .engine_info()
            .supports(space_proto::capabilities::HARNESS_UPDATES_V1)
    }

    fn can_control_updates(&self, cx: &Context<Self>) -> bool {
        self.supports_updates(cx) && matches!(self.updates, Loadable::Ready(_))
    }

    /// `ListHarnesses` (installed probe + enabled set both come from where
    /// the CLIs actually live).
    fn load(&mut self, cx: &mut Context<Self>) {
        let Some(engine) = self.state.read(cx).engine().cloned() else {
            return;
        };
        let params = serde_json::json!({});
        let update_params = params.clone();
        let supports_updates = self.supports_updates(cx);
        self.request_native_subagents(None, cx);

        self.harnesses = Loadable::Loading;
        self.updates = if supports_updates {
            Loadable::Loading
        } else {
            Loadable::Ready(Vec::new())
        };
        self.update_task = supports_updates.then(|| {
            let update_engine = engine.clone();
            cx.spawn(async move |this, cx| {
                let mut retry = 1;
                loop {
                    let subscribed = update_engine
                        .client()
                        .subscribe_checked(methods::WATCH_HARNESS_UPDATES, update_params.clone())
                        .await;
                    match subscribed {
                        Ok(mut stream) => {
                            while let Some(value) = stream.recv().await {
                                retry = 1;
                                let parsed =
                                    serde_json::from_value::<Vec<HarnessUpdateStatus>>(value)
                                        .map_err(|error| error.to_string());
                                if this
                                    .update(cx, |page, cx| {
                                        page.updates = match parsed {
                                            Ok(statuses) => Loadable::Ready(statuses),
                                            Err(error) => Loadable::Error(error),
                                        };
                                        cx.notify();
                                    })
                                    .is_err()
                                {
                                    return;
                                }
                            }
                        }
                        Err(error) => {
                            this.update(cx, |page, cx| {
                                page.updates = Loadable::Error(error.to_string());
                                cx.notify();
                            })
                            .ok();
                        }
                    }
                    if this
                        .update(cx, |page, cx| {
                            if !matches!(page.updates, Loadable::Error(_)) {
                                page.updates = Loadable::Error(
                                    "Connection closed. Reconnecting…".into(),
                                );
                            }
                            cx.notify();
                        })
                        .is_err()
                    {
                        return;
                    }
                    cx.background_executor()
                        .timer(std::time::Duration::from_secs(retry))
                        .await;
                    retry = (retry * 2).min(15);
                }
            })
        });
        let load_params = serde_json::json!({});
        self.load_task = Some(cx.spawn(async move |this, cx| {
            let result = engine.client().call(methods::LIST_HARNESSES, load_params).await;
            this.update(cx, |page, cx| {
                page.harnesses = match result {
                    Ok(value) => match serde_json::from_value::<Vec<HarnessDescriptor>>(value) {
                        Ok(list) => Loadable::Ready(list),
                        Err(err) => Loadable::Error(err.to_string()),
                    },
                    Err(err) => Loadable::Error(err.to_string()),
                };
                page.run_pending_sign_in(cx);
                cx.notify();
            })
            .ok();
        }));
    }

    /// Flip one harness. The reply carries the fresh catalog, so the rows
    /// repaint from the authoritative state in one round trip; refusals
    /// (engine guards) land in the error strip.
    fn toggle(&mut self, harness: HarnessId, enabled: bool, cx: &mut Context<Self>) {
        self.set_enabled(harness, enabled, cx);
    }

    fn cancel_install(&mut self, cx: &mut Context<Self>) {
        let Some(harness) = self.installing else {
            return;
        };
        let Some(engine) = self.state.read(cx).engine().cloned() else {
            return;
        };
        let params = install_params(harness);
        cx.spawn(async move |this, cx| {
            if let Err(error) = engine.client().call(methods::CANCEL_INSTALL, params).await {
                this.update(cx, |page, cx| {
                    if page.installing == Some(harness) {
                        page.error = Some(format!("Cancellation failed — {error}"));
                        cx.notify();
                    }
                })
                .ok();
            }
        })
        .detach();
    }

    fn install(&mut self, harness: HarnessId, cx: &mut Context<Self>) {
        let Some(engine) = self.state.read(cx).engine().cloned() else {
            return;
        };
        if self.installing.is_some() {
            return;
        }
        let params = install_params(harness);
        self.installing = Some(harness);
        self.error = None;
        self.install_task = Some(cx.spawn(async move |this, cx| {
            let result = engine
                .client()
                .call(methods::INSTALL_HARNESS, params)
                .await
                .map_err(|error| error.to_string())
                .and_then(|value| {
                    serde_json::from_value::<Vec<HarnessDescriptor>>(value)
                        .map_err(|error| error.to_string())
                });
            this.update(cx, |page, cx| {
                page.installing = None;
                match result {
                    Ok(list) => {
                        page.harnesses = Loadable::Ready(list);
                        crate::pickers::bump_harness_catalog(cx);
                    }
                    Err(error) => page.error = Some(format!("Installation failed — {error}")),
                }
                cx.notify();
            })
            .ok();
        }));
        cx.notify();
    }

    fn set_enabled(&mut self, harness: HarnessId, enabled: bool, cx: &mut Context<Self>) {
        let Some(engine) = self.state.read(cx).engine().cloned() else {
            return;
        };
        let params = serde_json::json!({
            "harness": harness,
            "enabled": enabled,
        });
        self.error = None;
        self.pending_toggle = Some((harness, enabled));
        cx.notify();
        self.toggle_task = Some(cx.spawn(async move |this, cx| {
            let result = engine
                .client()
                .call(methods::SET_HARNESS_ENABLED, params)
                .await;
            this.update(cx, |page, cx| {
                if page.pending_toggle == Some((harness, enabled)) {
                    page.pending_toggle = None;
                }
                match result {
                    Ok(value) => {
                        if let Ok(list) = serde_json::from_value::<Vec<HarnessDescriptor>>(value) {
                            page.harnesses = Loadable::Ready(list);
                        }
                        if !enabled && page.expanded_harness == Some(harness) {
                            page.expanded_harness = None;
                        }
                        // The composer caches its catalog per space — poke
                        // every Pickers to re-fetch, or the rail keeps the
                        // old set until restart.
                        crate::pickers::bump_harness_catalog(cx);
                    }
                    Err(err) => page.error = Some(err.to_string()),
                }
                cx.notify();
            })
            .ok();
        }));
    }

    fn check_updates(&mut self, cx: &mut Context<Self>) {
        if !self.can_control_updates(cx) {
            return;
        }
        let Some(engine) = self.state.read(cx).engine().cloned() else {
            return;
        };
        let params = serde_json::json!({});
        self.error = None;
        self.checking_updates = true;
        cx.notify();
        self.update_action_task = Some(cx.spawn(async move |this, cx| {
            let result = engine
                .client()
                .call(methods::CHECK_HARNESS_UPDATES, params)
                .await;
            this.update(cx, |page, cx| {
                page.checking_updates = false;
                match result {
                    Ok(value) => match serde_json::from_value::<Vec<HarnessUpdateStatus>>(value) {
                        Ok(statuses) => page.updates = Loadable::Ready(statuses),
                        Err(error) => page.error = Some(error.to_string()),
                    },
                    Err(error) => page.error = Some(error.to_string()),
                }
                cx.notify();
            })
            .ok();
        }));
    }

    fn apply_harness_update(&mut self, harness: HarnessId, cx: &mut Context<Self>) {
        if !self.can_control_updates(cx) {
            return;
        }
        let Some(engine) = self.state.read(cx).engine().cloned() else {
            return;
        };
        let params = serde_json::json!({ "harness": harness });
        self.error = None;
        self.update_action_task = Some(cx.spawn(async move |this, cx| {
            let result = engine
                .client()
                .call(methods::APPLY_HARNESS_UPDATE, params)
                .await;
            this.update(cx, |page, cx| {
                if let Err(error) = result {
                    page.error = Some(error.to_string());
                }
                cx.notify();
            })
            .ok();
        }));
    }

    fn cancel_harness_update(&mut self, harness: HarnessId, cx: &mut Context<Self>) {
        if !self.can_control_updates(cx) {
            return;
        }
        let Some(engine) = self.state.read(cx).engine().cloned() else {
            return;
        };
        let params = serde_json::json!({ "harness": harness });
        self.update_action_task = Some(cx.spawn(async move |this, cx| {
            let result = engine
                .client()
                .call(methods::CANCEL_HARNESS_UPDATE, params)
                .await;
            this.update(cx, |page, cx| {
                if let Err(error) = result {
                    page.error = Some(error.to_string());
                }
                cx.notify();
            })
            .ok();
        }));
    }

    fn set_update_policy(
        &mut self,
        harness: HarnessId,
        policy: HarnessUpdatePolicy,
        cx: &mut Context<Self>,
    ) {
        if !self.can_control_updates(cx) {
            return;
        }
        let Some(engine) = self.state.read(cx).engine().cloned() else {
            return;
        };
        let params = serde_json::json!({
            "harness": harness,
            "policy": policy,
        });
        self.error = None;
        self.update_action_task = Some(cx.spawn(async move |this, cx| {
            let result = engine
                .client()
                .call(methods::SET_HARNESS_UPDATE_POLICY, params)
                .await;
            this.update(cx, |page, cx| {
                if let Err(error) = result {
                    page.error = Some(error.to_string());
                }
                cx.notify();
            })
            .ok();
        }));
    }

    fn rows(&self, cx: &mut Context<Self>) -> Vec<gpui::AnyElement> {
        let theme = Theme::of(cx).for_settings_surface();
        let Loadable::Ready(list) = &self.harnesses else {
            return Vec::new();
        };
        let descriptors = visible_harnesses(list);
        let enabled_count = descriptors.iter().filter(|d| descriptor_enabled(d)).count();
        descriptors
            .into_iter()
            .enumerate()
            .map(|(ix, descriptor)| {
                let harness = descriptor.id;
                let installed = descriptor.installed;
                        let has_account = descriptor_enabled(&descriptor);
                let enabled = match self.pending_toggle {
                    Some((pending, on)) if pending == harness => on,
                    _ => has_account,
                };
                // The one enabled harness left can't be switched off — the
                // composer needs something to run — but only when it could
                // actually run: an uninstalled last harness stays togglable
                // (its hint says to turn it off) and the composer handles the
                // resulting empty set (mirrors the engine guard).
                let last_enabled = enabled && enabled_count == 1 && installed;
                // Turning OFF never needs the CLI (a default-on agent the
                // user doesn't want must not be stuck on because it isn't
                // installed); turning ON still does.
                let interactive = !last_enabled && (enabled || installed);
                let (icon_path, tint) = crate::pickers::harness_brand_icon(harness);

                let update = match &self.updates {
                    Loadable::Ready(statuses) => {
                        statuses.iter().find(|status| status.harness == harness)
                    }
                    _ => None,
                };

                let mut meta: Vec<gpui::AnyElement> = Vec::new();
                // Installing REPLACES the not-installed hint in place, so the
                // row's text never shifts while the install runs.

                if self.installing == Some(harness) {
                    meta.push(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(6.0))
                            .child(crate::loaders::ring_spinner(
                                format!("harness-install-spinner-{harness:?}"),
                                12.0,
                                theme.text_muted,
                            ))
                            .child(SharedString::from(install_label(&descriptor.name)))
                            .into_any_element(),
                    );
                } else if !installed {
                    meta.push(
                        div()
                            .text_color(theme.text_muted.opacity(0.75))
                            .child(SharedString::from(install_hint(
                                harness,
                                enabled,
                                descriptor.can_install,
                            )))
                            .into_any_element(),
                    );
                }
                if installed && harness == HarnessId::Fx {
                    meta.push(
                        div()
                            .text_color(theme.text_muted)
                            .child("Configure your model provider with fx login or fx setup.")
                            .into_any_element(),
                    );
                }
                // Not installed: the hint above says all there is.
                if let Some(status) = update.filter(|_| installed) {
                    let (text, color) = harness_update_label(status, &theme);
                    // Next to an Update button the version reads as the
                    // jump it would make: v1.2 → v1.3.
                    let next = status
                        .latest_version
                        .as_deref()
                        .filter(|_| {
                            status.can_apply
                                && matches!(
                                    status.phase,
                                    HarnessUpdatePhase::Available
                                        | HarnessUpdatePhase::ManualActionRequired
                                )
                        })
                        .filter(|latest| status.installed_version.as_deref() != Some(*latest));
                    if let Some(next) = next.filter(|_| !text.is_empty()) {
                        meta.push(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(4.0))
                                .text_color(color)
                                .child(text)
                                .child(
                                    crate::icons::icon(crate::icons::ARROW_RIGHT)
                                        .size(px(11.0))
                                        .text_color(color),
                                )
                                .child(format!("v{next}"))
                                .into_any_element(),
                        );
                    } else if !text.is_empty() {
                        meta.push(div().text_color(color).child(text).into_any_element());
                    }
                }
                // widgets::row_tile with the brand tint honored (the Claude
                // mark keeps its orange, like the picker rail).
                let tile = div()
                    .flex_none()
                    .size(px(36.0))
                    .rounded(px(10.0))
                    .bg(theme.wash(0.06))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        crate::icons::icon(icon_path)
                            .size(px(16.0))
                            .text_color(tint.unwrap_or(theme.text_muted)),
                    );
                let expanded = enabled && self.expanded_harness == Some(harness);
                // The row's one update action sits inside the trigger, before
                // the chevron, so it appearing never moves the chevron; the
                // update policy lives in the expanded details.
                let update_action = update.and_then(|status| {
                    let (id, label, primary) = match status.phase {
                        HarnessUpdatePhase::Available
                        | HarnessUpdatePhase::ManualActionRequired
                            if status.can_apply =>
                        {
                            ("harness-update", "Update", true)
                        }
                        HarnessUpdatePhase::Failed
                            if status.can_apply && install_failed(status) =>
                        {
                            ("harness-update", "Retry update", true)
                        }
                        HarnessUpdatePhase::WaitingForIdle
                        | HarnessUpdatePhase::Preparing
                        | HarnessUpdatePhase::Downloading => {
                            ("harness-update-cancel", "Cancel", false)
                        }
                        _ => return None,
                    };
                    Some(
                        div()
                            .id((id, ix))
                            .flex_none()
                            .px(px(9.0))
                            .py(px(5.0))
                            .rounded(px(6.0))
                            .text_size(crate::typography::ui_rems(11.0))
                            .cursor_pointer()
                            .map(|button| {
                                if primary {
                                    button
                                        .bg(theme.accent_wash)
                                        .font_weight(gpui::FontWeight::MEDIUM)
                                        .text_color(theme.accent)
                                        .hover(|style| style.bg(theme.accent.opacity(0.16)))
                                } else {
                                    button
                                        .text_color(theme.text_muted)
                                        .hover(|style| style.bg(crate::theme::ink(0.05)))
                                }
                            })
                            .on_click(cx.listener(move |this, _, _, cx| {
                                // Inside the details trigger: act, don't expand.
                                cx.stop_propagation();
                                if primary {
                                    this.apply_harness_update(harness, cx)
                                } else {
                                    this.cancel_harness_update(harness, cx)
                                }
                            }))
                            .child(label),
                    )
                });
                let header = widgets::card_row(&theme, ix == 0)
                    .id(("harness-row", ix))
                    .when(!installed && self.installing != Some(harness), |el| {
                        el.opacity(0.55)
                    })
                    .child(
                        div()
                            .id(("harness-details-trigger", ix))
                            .group(format!("harness-details-{ix}"))
                            .flex_1()
                            .min_w(px(180.0))
                            .min_h(px(44.0))
                            .px(px(4.0))
                            .rounded(px(8.0))
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(12.0))
                            .when(enabled, |el| {
                                el.role(gpui::Role::Button)
                                    .aria_label(format!("{} preferences", descriptor.name))
                                    .aria_expanded(expanded)
                                    .tab_index(0)
                                    .cursor_pointer()
                                    // No hover slab (it stopped short of the
                                    // toggle and boxed the row); the chevron
                                    // lifts instead, like a list disclosure.
                                    .focus_visible(|s| s.border_2().border_color(theme.accent))
                                    .on_click(cx.listener(move |page, _, _, cx| {
                                        page.toggle_agent_details(harness, cx)
                                    }))
                                    .on_key_down(cx.listener(
                                        move |page, event: &gpui::KeyDownEvent, _, cx| {
                                            if !event.is_held
                                                && matches!(
                                                    event.keystroke.key.as_str(),
                                                    "enter" | "space"
                                                )
                                            {
                                                page.toggle_agent_details(harness, cx);
                                                cx.stop_propagation();
                                            }
                                        },
                                    ))
                            })
                            .child(tile)
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .flex()
                                    .flex_col()
                                    .child(widgets::row_title(&theme, descriptor.name.clone()))
                                    .when(!meta.is_empty(), |label| {
                                        label.child(widgets::meta_line(&theme, meta))
                                    }),
                            )
                            .children(update_action)
                            // Install / Cancel sit with the row's other
                            // action, before the caret, so carets line up.
                            .when(
                                offers_install(harness, installed, descriptor.can_install)
                                    && self.installing != Some(harness),
                                |el| {
                                    el.child(
                                        widgets::ghost_action(&theme)
                                            .id(("harness-install", ix))
                                            .flex_none()
                                            .when(self.installing.is_none(), |el| {
                                                el.on_click(cx.listener(move |this, _, _, cx| {
                                                    cx.stop_propagation();
                                                    this.install(harness, cx)
                                                }))
                                            })
                                            .child("Install"),
                                    )
                                },
                            )
                            .when(self.installing == Some(harness), |el| {
                                el.child(
                                    widgets::ghost_action(&theme)
                                        .id(("harness-cancel-install", ix))
                                        .flex_none()
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            cx.stop_propagation();
                                            this.cancel_install(cx)
                                        }))
                                        .child("Cancel"),
                                )
                            })
                            // Always present so every row's controls hold
                            // their place; inert while the provider is off.
                            .child(
                                crate::icons::icon(if expanded {
                                    crate::icons::ALT_ARROW_DOWN
                                } else {
                                    crate::icons::ALT_ARROW_RIGHT
                                })
                                .size(px(14.0))
                                .flex_none()
                                .when(enabled, |caret| {
                                    caret
                                        .text_color(theme.text_muted)
                                        .group_hover(format!("harness-details-{ix}"), |s| {
                                            s.text_color(theme.text)
                                        })
                                })
                                .when(!enabled, |caret| {
                                    caret.text_color(theme.text_muted.opacity(0.35))
                                }),
                            ),
                    )
                    .child(
                        widgets::toggle_switch(
                                &theme,
                                enabled,
                                format!("harness-switch-{harness:?}"),
                            )
                            .id(("harness-toggle", ix))
                            .when(!interactive && !enabled, |el| el.opacity(0.55))
                            .when(interactive, |el| {
                                el.cursor_pointer()
                                    .tab_index(0)
                                    .role(gpui::Role::Switch)
                                    .aria_label(descriptor.name.clone())
                                    .aria_toggled(if enabled {
                                        gpui::Toggled::True
                                    } else {
                                        gpui::Toggled::False
                                    })
                                    .focus_visible(|s| {
                                        s.border_2().border_color(theme.accent).opacity(1.0)
                                    })
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.toggle(harness, !enabled, cx);
                                    }))
                                    .on_key_down(cx.listener(
                                        move |this, event: &gpui::KeyDownEvent, _, cx| {
                                            if !event.is_held
                                                && matches!(
                                                    event.keystroke.key.as_str(),
                                                    "enter" | "space"
                                                )
                                            {
                                                this.toggle(harness, !enabled, cx);
                                                cx.stop_propagation();
                                            }
                                        },
                                    ))
                            }),
                    );
                div()
                    .flex()
                    .flex_col()
                    .child(header)
                    .when(expanded, |row| {
                        row.child(self.render_agent_details(harness, &theme, cx))
                    })
                    .into_any_element()
            })
            .collect()
    }
    fn on_scroll_hovered(&mut self, hovered: &bool, _: &mut Window, cx: &mut Context<Self>) {
        if self.scroll.set_list_hovered(*hovered) {
            cx.notify();
        }
    }
}

impl popover::ScrollRailHost for HarnessesPage {
    fn rail_bar(&mut self) -> &mut popover::MenuScrollbarState {
        self.scroll.rail_bar()
    }

    fn rail_scroll(&self) -> Option<gpui::ScrollHandle> {
        self.scroll.rail_scroll()
    }
}

impl Render for HarnessesPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::of(cx).for_settings_surface();
        let embedded = self.embedded;
        let body: gpui::AnyElement = match &self.harnesses {
            Loadable::Idle | Loadable::Loading => widgets::section_card(&theme)
                .when(embedded, |card| card.mt_0())
                .h(px(240.0))
                .items_center()
                .justify_center()
                .child(crate::loaders::ring_spinner(
                    "harnesses-loading",
                    20.0,
                    theme.text_muted,
                ))
                .into_any_element(),
            Loadable::Error(message) => {
                let message = message.clone();
                div()
                    .child(widgets::error_strip(&theme, message))
                    .child(
                        widgets::ghost_action(&theme)
                            .id("harnesses-retry")
                            .mt(px(8.0))
                            .tab_index(0)
                            .role(gpui::Role::Button)
                            .focus_visible(|s| s.border_2().border_color(theme.accent).opacity(1.0))
                            .on_click(cx.listener(|page, _, _, cx| {
                                page.load(cx);
                                cx.notify();
                            }))
                            .child(SharedString::from("Retry")),
                    )
                    .into_any_element()
            }
            Loadable::Ready(_) => {
                let rows = self.rows(cx);
                widgets::section_card(&theme)
                    .when(embedded, |card| card.mt_0())
                    .children(rows)
                    .into_any_element()
            }
        };
        // A provider's own rows show once it is expanded: until then, a hit
        // on one lands on the list, to pick the provider.
        let body = body
            .search_anchor(Target::Page(SettingsSection::Harnesses))
            .block();
        crate::toast::take_error(&mut self.error, cx);
        if embedded {
            let scrollbar = popover::rail(self, "harnesses-embedded-scrollbar", &theme, cx);
            return div()
                .id("harnesses-embedded-host")
                .relative()
                .size_full()
                .on_hover(cx.listener(Self::on_scroll_hovered))
                .child(
                    crate::edge_fade::edge_faded(
                        16.0,
                        true,
                        true,
                        div()
                            .id("harnesses-embedded")
                            .size_full()
                            .overflow_y_scroll()
                            .track_scroll(&self.scroll.scroll)
                            .child(body),
                    )
                    .fade_overflow_y(&self.scroll.scroll),
                )
                .children(scrollbar)
                .into_any_element();
        }
        let update_error = match &self.updates {
            Loadable::Error(message) => Some(
                div()
                    .child(widgets::error_strip(
                        &theme,
                        format!("Agent updates: {message}"),
                    ))
                    .child(
                        widgets::ghost_action(&theme)
                            .id("harness-updates-retry")
                            .mt(px(8.0))
                            .on_click(cx.listener(|page, _, _, cx| {
                                page.load(cx);
                                cx.notify();
                            }))
                            .child("Retry updates"),
                    )
                    .into_any_element(),
            ),
            _ => None,
        };
        let toolbar = self.render_toolbar(&theme, cx);

        let scrollbar = popover::rail(self, "harnesses-page-scrollbar", &theme, cx);

        div()
            .id("harnesses-page-host")
            .relative()
            .size_full()
            .on_hover(cx.listener(Self::on_scroll_hovered))
            .child(
                crate::edge_fade::edge_faded(
                    16.0,
                    true,
                    true,
                    div()
                        .id("harnesses-page")
                        .size_full()
                        .overflow_y_scroll()
                        .track_scroll(&self.scroll.scroll)
                        .child(
                            widgets::page_column()
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .justify_between()
                                        .child(widgets::page_header(&theme, "Providers", None))
                                        .child(toolbar),
                                )
                                .children(update_error)
                                .child(body),
                        ),
                )
                .fade_overflow_y(&self.scroll.scroll),
            )
            .child(widgets::page_reveal(&self.scroll.scroll))
            .children(scrollbar)
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    /// A failed run's "Log in" waits for the rows, then opens that agent's
    /// Accounts section (where its sign-in starts).
    #[gpui::test]
    fn sign_in_opens_the_agent_once_its_rows_land(cx: &mut gpui::TestAppContext) {
        use gpui::AppContext;
        let dir = tempfile::tempdir().unwrap();
        cx.update(|cx| {
            crate::settings::init(Default::default(), dir.path(), cx);
            gpui_base::init(cx);
            cx.set_global(crate::theme::Theme::default());
        });
        let window = cx.add_window(|_, cx| {
            let state = cx.new(|_| crate::state::AppState::new());
            super::HarnessesPage::new(state, cx)
        });
        window
            .update(cx, |page, _, cx| {
                page.harnesses = super::Loadable::Loading;
                page.sign_in(space_proto::HarnessId::Codex, cx);
                assert_eq!(page.expanded_harness, None, "nothing to open yet");
                page.harnesses =
                    super::Loadable::Ready(vec![space_engine::registry::HarnessDescriptor {
                        id: space_proto::HarnessId::Codex,
                        name: "Codex".into(),
                        supports_steering: false,
                        steering_mode: space_proto::SteeringMode::TurnBoundary,
                        reasoning_levels: Vec::new(),
                        installed: true,
                        can_install: false,
                        enabled: Some(true),
                        access_modes: Vec::new(),
                        supports_plan: false,
                    }]);
                page.run_pending_sign_in(cx);
                assert_eq!(page.expanded_harness, Some(space_proto::HarnessId::Codex));
                assert_eq!(
                    page.accounts_page
                        .as_ref()
                        .unwrap()
                        .read(cx)
                        .embedded_harness(),
                    Some(space_proto::HarnessId::Codex)
                );
                assert_eq!(page.pending_sign_in, None, "started once");
            })
            .unwrap();
    }

    #[gpui::test]
    fn expanded_agent_preferences_render_inside_the_agent_row(cx: &mut gpui::TestAppContext) {
        use gpui::AppContext;
        let dir = tempfile::tempdir().unwrap();
        cx.update(|cx| {
            crate::settings::init(Default::default(), dir.path(), cx);
            gpui_base::init(cx);
            cx.set_global(crate::theme::Theme::default());
        });
        let window = cx.add_window(|_, cx| {
            let state = cx.new(|_| crate::state::AppState::new());
            super::HarnessesPage::new(state, cx)
        });
        let descriptor = |id, name: &str| space_engine::registry::HarnessDescriptor {
            id,
            name: name.into(),
            supports_steering: false,
            steering_mode: space_proto::SteeringMode::TurnBoundary,
            reasoning_levels: Vec::new(),
            installed: true,
            can_install: false,
            enabled: Some(true),
            access_modes: Vec::new(),
            supports_plan: false,
        };
        window
            .update(cx, |page, _, cx| {
                page.harnesses = super::Loadable::Ready(vec![
                    descriptor(space_proto::HarnessId::ClaudeCode, "Claude Code"),
                    descriptor(space_proto::HarnessId::Codex, "Codex"),
                ]);
                page.toggle_agent_details(space_proto::HarnessId::ClaudeCode, cx);
                assert_eq!(
                    page.expanded_harness,
                    Some(space_proto::HarnessId::ClaudeCode)
                );
                assert_eq!(
                    page.accounts_page
                        .as_ref()
                        .unwrap()
                        .read(cx)
                        .embedded_harness(),
                    Some(space_proto::HarnessId::ClaudeCode)
                );
            })
            .unwrap();
        cx.update_window(window.into(), |_, window, cx| window.draw(cx).clear())
            .unwrap();
        window
            .update(cx, |page, _, cx| {
                page.toggle_agent_details(space_proto::HarnessId::Codex, cx);
                assert_eq!(page.expanded_harness, Some(space_proto::HarnessId::Codex));
                assert_eq!(
                    page.accounts_page
                        .as_ref()
                        .unwrap()
                        .read(cx)
                        .embedded_harness(),
                    Some(space_proto::HarnessId::Codex)
                );
            })
            .unwrap();
        cx.update_window(window.into(), |_, window, cx| window.draw(cx).clear())
            .unwrap();
        // Antigravity signs in through the very same Accounts section.
        window
            .update(cx, |page, _, cx| {
                page.harnesses = super::Loadable::Ready(vec![descriptor(
                    space_proto::HarnessId::Antigravity,
                    "Antigravity",
                )]);
                page.toggle_agent_details(space_proto::HarnessId::Antigravity, cx);
                assert_eq!(
                    page.accounts_page
                        .as_ref()
                        .unwrap()
                        .read(cx)
                        .embedded_harness(),
                    Some(space_proto::HarnessId::Antigravity)
                );
            })
            .unwrap();
        cx.update_window(window.into(), |_, window, cx| window.draw(cx).clear())
            .unwrap();
    }

}

#[cfg(test)]
#[test]
fn account_counts_read_naturally() {
    assert_eq!(account_count_label(1), "1 account");
    assert_eq!(account_count_label(3), "3 accounts");
}

#[cfg(test)]
#[test]
fn install_visibility_and_hint_follow_target_capabilities() {
    for id in [
        HarnessId::Antigravity,
        HarnessId::Codex,
        HarnessId::Opencode,
        HarnessId::ClaudeCode,
        HarnessId::Cursor,
        HarnessId::Pi,
        HarnessId::Grok,
        HarnessId::Hermes,
        HarnessId::Devin,
        HarnessId::Mock,
    ] {
        for installed in [false, true] {
            for available in [false, true] {
                assert_eq!(
                    offers_install(id, installed, available),
                    id != HarnessId::Mock && !installed && available
                );
            }
        }
    }
    assert_eq!(cli_name(HarnessId::Antigravity), "Antigravity");
    assert_eq!(
        install_hint(HarnessId::Antigravity, false, true),
        "Install Antigravity to enable"
    );
    assert!(
        install_hint(HarnessId::Antigravity, false, false).contains("ANTIGRAVITY_ACP_EXECUTABLE")
    );
}

#[cfg(test)]
#[test]
fn install_phase_copy_and_cancel_target_match_install() {
    assert_eq!(install_label("Claude Code"), "Installing Claude Code…");
    assert_eq!(install_label("Pi"), "Installing Pi…");
    let params = install_params(HarnessId::Pi);
    assert_eq!(params["harness"], "pi");
    assert_eq!(
        install_hint(HarnessId::Codex, false, false),
        "Install the codex CLI to enable. Install with `npm install -g @openai/codex`"
    );
    assert!(
        install_hint(HarnessId::Codex, true, false)
            .starts_with("codex CLI not installed — turn it off or install it.")
    );
}
