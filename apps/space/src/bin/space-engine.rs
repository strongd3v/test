//! `space-engine` — the desktop app's background engine, as a process of its
//! own ("Space Engine" in Activity Monitor): the app starts it and attaches;
//! sessions keep running with the app closed. Without the UI linked in.
//!
//! It also answers what the engine runs its own executable for: `mcp` (the
//! Space MCP server injected into agents) and `--noop-browser` (browser
//! suppression for agents' sign-in flows).

#![cfg_attr(windows, windows_subsystem = "windows")]

fn main() -> anyhow::Result<()> {
    match std::env::args().nth(1).as_deref() {
        Some("--noop-browser") => Ok(()),
        Some("mcp") => {
            space_app::init_logging(false, "mcp", true);
            space_app::run_mcp()
        }
        Some("--version" | "-V") => {
            println!("space-engine {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        _ => space_app::background::run(),
    }
}
