//! A computer's agents from the phone: which harnesses it offers and the CLI
//! logins each one runs on — what desktop Settings → Providers shows for a
//! device picked in its switcher. The demo host owns its own CLI logins.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use space_proto::{
    AgentAccount, AgentAccountsSnapshot, AgentAuthKind, AgentLoginStart, HarnessId,
};

use crate::Client;
use crate::catalog::HarnessInfo;
use crate::error::{ClientError, Result};
use crate::lock;

impl Client {
    /// `StartAgentLogin` on `device_id` for `harness`. Unavailable in demo.
    pub async fn device_login_start(
        &self,
        _device_id: &str,
        _harness: &str,
        _provider: Option<&str>,
        _done_url: &str,
    ) -> Result<AgentLoginStart> {
        Err(ClientError::Unsupported(
            "Signing a computer's agent in needs that computer.".into(),
        ))
    }

    /// `PollAgentLogin`: unavailable in demo.
    pub async fn device_login_poll(
        &self,
        _device_id: &str,
        _login_id: &str,
        _done_url: &str,
    ) -> Result<space_proto::AgentLoginPoll> {
        Err(ClientError::Unsupported("No computer in the demo.".into()))
    }

    /// `CompleteAgentLogin`: the code Claude's page showed, pasted back.
    pub async fn device_login_complete(
        &self,
        device_id: &str,
        login_id: &str,
        code: &str,
    ) -> Result<AgentAccountsSnapshot> {
        self.accounts_call(
            device_id,
            space_rpc::methods::COMPLETE_AGENT_LOGIN,
            serde_json::json!({ "loginId": login_id, "code": code.trim() }),
        )
        .await
    }

    /// Give up on a computer's sign-in.
    pub fn device_login_cancel(&self, _device_id: &str, _login_id: &str) {}

    /// `SetHarnessEnabled` on `device_id`: offer `harness` in its pickers or
    /// not. Replies with the device's fresh catalog.
    pub async fn set_harness_enabled(
        &self,
        device_id: &str,
        harness: &str,
        enabled: bool,
    ) -> Result<Vec<HarnessInfo>> {
        lock(&DEMO_DISABLED).insert(harness.to_owned(), !enabled);
        let list = with_demo_toggles(self.inner.demo().list_harnesses(device_id).await);
        lock(&self.inner.harness_catalogs).insert(device_id.to_owned(), list.clone());
        self.inner.recompute_workspace();
        Ok(list)
    }

    /// `ListAgentAccounts` on `device_id`: its CLI logins per agent.
    pub async fn device_agent_accounts(&self, device_id: &str) -> Result<AgentAccountsSnapshot> {
        self.accounts_call(
            device_id,
            space_rpc::methods::LIST_AGENT_ACCOUNTS,
            serde_json::json!({}),
        )
        .await
    }

    /// `ActivateAgentAccount`: the login `harness` runs on from now on.
    pub async fn device_activate_account(
        &self,
        device_id: &str,
        harness: &str,
        account_id: &str,
    ) -> Result<AgentAccountsSnapshot> {
        lock(&DEMO_ACTIVE).insert(harness.to_owned(), account_id.to_owned());
        self.accounts_call(
            device_id,
            space_rpc::methods::ACTIVATE_AGENT_ACCOUNT,
            serde_json::json!({ "harness": harness, "accountId": account_id }),
        )
        .await
    }

    /// `ForgetAgentAccount`: drop a saved login from that computer.
    pub async fn device_forget_account(
        &self,
        device_id: &str,
        harness: &str,
        account_id: &str,
    ) -> Result<AgentAccountsSnapshot> {
        lock(&DEMO_FORGOTTEN).push(account_id.to_owned());
        self.accounts_call(
            device_id,
            space_rpc::methods::FORGET_AGENT_ACCOUNT,
            serde_json::json!({ "harness": harness, "accountId": account_id }),
        )
        .await
    }

    async fn accounts_call(
        &self,
        _device_id: &str,
        _method: &str,
        _params: serde_json::Value,
    ) -> Result<AgentAccountsSnapshot> {
        Ok(demo_accounts())
    }
}

/// Demo mode's toggles and switches, so the screens respond offline.
static DEMO_DISABLED: LazyLock<Mutex<HashMap<String, bool>>> = LazyLock::new(Default::default);
static DEMO_ACTIVE: LazyLock<Mutex<HashMap<String, String>>> = LazyLock::new(Default::default);
static DEMO_FORGOTTEN: LazyLock<Mutex<Vec<String>>> = LazyLock::new(Default::default);

/// The demo catalog with the toggles applied.
pub(crate) fn with_demo_toggles(mut list: Vec<HarnessInfo>) -> Vec<HarnessInfo> {
    let disabled = lock(&DEMO_DISABLED);
    for harness in &mut list {
        if disabled.get(&harness.id).copied().unwrap_or(false) {
            harness.enabled = Some(false);
        }
    }
    list
}

fn demo_accounts() -> AgentAccountsSnapshot {
    let window = |label: &str, used: f32, hours: i64| space_proto::AgentUsageWindow {
        label: label.into(),
        used_fraction: used,
        resets_at: Some(chrono::Utc::now() + chrono::Duration::hours(hours)),
    };
    let account = |id: &str, harness: HarnessId, email: &str, plan: &str| AgentAccount {
        id: id.into(),
        harness,
        email: Some(email.into()),
        plan_label: Some(plan.into()),
        active: false,
        usage_windows: vec![window("Session", 0.34, 3), window("Week", 0.12, 96)],
        usage_fetched_at: None,
        usage_error: None,
        display_name: None,
        organization: None,
        auth_kind: Some(AgentAuthKind::Oauth),
        switchable: true,
        saved_at: None,
        provider: None,
    };
    let mut accounts = vec![
        account(
            "demo-claude-1",
            HarnessId::ClaudeCode,
            "you@acme.dev",
            "Max",
        ),
        account(
            "demo-claude-2",
            HarnessId::ClaudeCode,
            "you@personal.dev",
            "Pro",
        ),
        account("demo-codex-1", HarnessId::Codex, "you@acme.dev", "Plus"),
    ];
    let forgotten = lock(&DEMO_FORGOTTEN);
    accounts.retain(|a| !forgotten.contains(&a.id));
    let active = lock(&DEMO_ACTIVE);
    for harness in [HarnessId::ClaudeCode, HarnessId::Codex] {
        let key = serde_json::to_value(harness)
            .ok()
            .and_then(|v| v.as_str().map(str::to_owned))
            .unwrap_or_default();
        let chosen = active.get(&key).cloned();
        let mut picked = false;
        for a in accounts.iter_mut().filter(|a| a.harness == harness) {
            a.active = match &chosen {
                Some(id) => *id == a.id,
                None => !picked,
            };
            picked |= a.active;
        }
    }
    AgentAccountsSnapshot {
        accounts,
        warnings: Vec::new(),
    }
}
