<p align="center">
  <a href="https://space.pentomake.com">
    <picture>
      <img src="docs\media\readme\space.png" alt="Space">
    </picture>
  </a>
</p>

<p align="center">
  <a href="https://github.com/strongd3v/test/blob/test/LICENSE">
    <img src="https://img.shields.io/github/license/strongd3v/test?style=for-the-badge&color=red" alt="License">
  </a>
  <a href="https://github.com/strongd3v/test/releases">
    <img src="https://img.shields.io/github/v/release/strongd3v/test?style=for-the-badge&color=green" alt="Latest Release">
  </a>
  <a href="https://github.com/strongd3v/test/blob/test/Cargo.toml">
    <img src="https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fraw.githubusercontent.com%2Fstrongd3v%2Ftest%2Ftest%2FCargo.toml&query=%24.workspace.package.edition&label=Rust&style=for-the-badge&color=blue" alt="Rust Edition">
  </a>
  <br>
  <a href="README-DE.md">Deutsch</a> | <strong>English</strong> | <a href="README-ES.md">Español</a> | <a href="README-FR.md">Français</a> | <a href="README-PT.md">Português</a> | <a href="README-TR.md">Türkçe</a> | <a href="README-AR.md">العربية</a> | <a href="README-HI.md">हिन्दी</a> | <a href="README-ZH.md">中文</a> | <a href="README-JA.md">日本語</a>
</p>

Space is a native desktop controller for coding agents: one interface for the agent CLIs, tools, projects, and devices you work with every day. Instead of living in a browser tab or locking you to a single provider, Space drives the coding agents already installed on your machines, from Claude Code to Cursor, while every session, artifact, and setting stays on your own hardware.

Under the interface, Space pairs a fast native app with a real engine: a local service that owns sessions, transcripts, terminals, and workspace state, reachable from the app or an MCP client. Linux, macOS, and Windows builds are published on the [Releases](https://github.com/strongd3v/test/releases) page.

---

### Why Space

* **Bring your own agent.** Antigravity, Claude Code, Codex, Cursor, Devin, fx, Grok, Hermes, Muse Code, OpenCode, and Pi are all driven through their native protocols or ACP behind one interface. You choose the harness and model per conversation while Space keeps the workflow consistent no matter which agent is behind it.

* **Native by default.** Space is written in Rust and rendered with GPUI, the UI framework behind Zed that is vendored and extended here. No Electron, no browser wrapper: streaming transcripts, syntax-highlighted diffs, and live terminal panes stay responsive even on long sessions.

* **Local-first.** Sessions, settings, and workspace state live on disk under your own data directory, and the engine starts in-process without requiring an account. Unplugging the network does not stop your work; sync is something you opt into, not a dependency.

* **Built to survive real work.** Runs ride a durable command queue and a crash-recovery journal, so an interrupted conversation picks up where it left off instead of losing progress to a restart.

* **A full workspace, not a chat box.** Alongside the conversation, Space ships a file explorer, an embedded PTY terminal, git changes and diff views, a pull-request pane, an embedded browser, and a command palette that searches actions, threads, files, and content.

* **Speech, when you want it.** Dictation and a full-window voice stage run on a local speech-to-text model, so your audio never has to leave the machine.

---

### Supported Agents

Space talks to each agent CLI through the protocol it actually speaks:

* **Antigravity** (Google): driven over ACP.
* **Claude Code** (Anthropic): driven through its native streaming protocol.
* **Codex** (OpenAI): driven through its native app-server protocol.
* **Cursor**: the Cursor CLI, driven over the Agent Client Protocol (ACP).
* **Devin** (Cognition): driven over ACP.
* **fx** (Vercel): driven through its native ACP server.
* **Grok** (xAI): the Grok Build agent, driven over ACP.
* **Hermes** (Nous Research): driven over ACP.
* **Muse Code** (Meta): driven natively over the Muse Session Protocol (MSP).
* **OpenCode** (SST): driven natively over its HTTP/SSE server protocol.
* **Pi**: driven through its native JSONL RPC protocol.

Each harness stays an independent installation. Space surfaces update notices in the sidebar rather than mutating anything behind your back, and the composer's harness and model picker switches agents per conversation, so planning can stay with one model and implementation with another.

---

### Conversations

Space stores every session locally, so development can move across days rather than die inside one window: history is searchable and resumable, conversations can be renamed or forked into side chats to explore an alternative without losing the original thread, and drafts survive restarts.

The transcript itself is virtualized and incrementally parsed: markdown, syntax highlighting, LaTeX math, folded tool groups, and sticky-to-bottom streaming stay smooth at thousands of messages. The composer's Send/Steer/Stop flow lets you queue a message behind a running turn, redirect the agent mid-flight with a correction, or interrupt it outright. When an agent asks you a question, the answer happens inline, and images or captured screenshots can be attached right where the conversation is.

---

### Workspace

Space treats the project as part of the conversation rather than a separate program:

* **Files panel**: explorer with git status, renames, and quick edits.
* **Terminal**: embedded PTY tabs with bounded replay, rendered natively.
* **Changes**: side-by-side diffs and the git changes view, next to the chat.
* **Pull requests**: a dedicated pane for review, comments, and todo tracking.
* **Browser**: a real tabbed webview next to the work, for previews and docs.
* **Appshots**: capture any window on your desktop and attach it with the app's accessibility context, so the agent sees what you see.
* **Command palette**: one keystroke over actions, conversations, workspace files, and file content.

The right pane switches between files, diffs, terminals, side chats, subagent transcripts, and the browser without moving the conversation out of view.

---

### Voice

Dictation and a full-window voice stage run on a local speech-to-text model: you speak, Space types, and captions and live transcription render in the same interface. Your audio never has to leave the machine.

---

### MCP

`space mcp` runs a Model Context Protocol stdio server that proxies the engine's local API, letting external tools and other agents drive your Space sessions through a standardized interface.

---

### Security

Found a vulnerability? Please report it privately rather than opening a public issue; the reporting process and contact details live in [SECURITY.md](./SECURITY.md).

---

### Contributing

Space is developed in the open, and contributions are welcome. Harness drivers for more agent CLIs, fixes to the transcript and terminal, packaging, and documentation are all useful. Smaller focused pull requests are the easiest to review.

Before starting a larger change, read the [contributing guidelines](./CONTRIBUTING.md) and open an issue to discuss the direction so the work stays consistent with the project's local-first, native-desktop focus. Everyone participating agrees to follow the [Code of Conduct](./CODE_OF_CONDUCT.md).

---

### Sponsor

Space is built and maintained by a small team, and sponsorship keeps it that way, funding the native engine and more agent harnesses. If Space saves your team time, reach out at [space@pentomake.com](mailto:space@pentomake.com) to become a sponsor.

---

### License

Space is licensed under the [GPL-3.0 license](./LICENSE.md).
