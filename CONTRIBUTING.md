### Contributing to Space

Space is developed in the open, and contributions are welcome. The project values clear code, focused changes, reliable behavior, and a consistent native desktop experience.

Contributions can improve any part of Space, including:

* Bug fixes and reliability improvements
* Harness drivers for more agent CLIs
* Transcript, terminal, and diff rendering
* Workspace panels: files, terminal, changes, pull requests, browser, and the command palette
* MCP support
* Voice and speech-to-text workflows
* Performance improvements
* Platform and packaging fixes for Linux, macOS, and Windows
* Tests and development tooling
* Documentation

Before starting significant work, check the existing issues and codebase to make sure the problem has not already been addressed. For larger features or changes to the core product experience, open an issue and discuss the proposed direction with the maintainers before implementation.

The goal is to solve real problems while keeping Space maintainable, coherent, and consistent as the project grows.

---

### Development Setup

Space is written in Rust and targets a stable toolchain with the `rustfmt` and `clippy` components. The workspace uses the 2024 edition, so a recent stable Rust is required; `rust-toolchain.toml` pins the expectation and `rustup` installs it automatically.

Clone the repository and enter the project directory:

```bash
git clone https://github.com/strongd3v/test.git
cd space
```

Run the desktop app from the source tree during development:

```bash
cargo run
```

Build all workspace crates, not just the app:

```bash
cargo build --workspace
```

Run the complete test suite:

```bash
cargo test --workspace
```

Run Rust's static analysis checks:

```bash
cargo clippy --workspace
```

Format the Rust source code:

```bash
cargo fmt
```

Before opening a pull request, make sure the project builds successfully, relevant tests pass, clippy reports no new issues, and changed Rust files are formatted.

The `vendor/` directory is excluded from the workspace and holds the vendored GPUI framework and its dependencies; do not modify it as part of a contribution.

If your change requires additional development tooling, document it clearly rather than assuming that contributors already have it installed.

---

### Pull Requests

Pull requests should represent one focused change.

A good pull request should make it easy for a maintainer to understand:

1. What problem does this change solve?
2. What exactly changed?
3. How was the change verified?

Keep unrelated refactoring, formatting changes, dependency updates, or feature work out of the same pull request unless they are directly required by the change.

For non-trivial changes, reference an existing issue or open one before implementation. This gives the project a place to discuss the problem and helps prevent duplicated work.

Use `Fixes #123` or `Closes #123` in your PR description to link the issue.

Small, self-contained fixes may be submitted without opening a separate issue when the purpose and scope of the change are immediately clear.

---

### Interface Changes

Space is a native desktop application rendered with GPUI, so changes to the interface should be reviewed with the same care as changes to engine behavior.

For changes that affect the interface, interaction model, layout, rendering, keyboard controls, themes, or user workflow, include screenshots or a short recording when useful.

If a change introduces a new interaction or modifies existing behavior, explain the expected workflow in the pull request.

Significant changes to the overall Space interface or interaction model should be discussed with the maintainers before implementation.

---

### Harness Drivers

Harness drivers are how Space talks to agent CLIs such as Claude Code, Codex, Cursor, Pi, and OpenCode. They live in `crates/harness`, with the shared protocol model in `crates/proto`.

When adding or changing a driver:

* Follow the existing driver architecture rather than introducing a separate implementation pattern.
* Keep harness-specific behavior isolated behind the shared driver interface.
* Handle streaming output, tool calls, and errors consistently with existing drivers.
* Do not mutate a user's harness installation; Space surfaces update notices rather than performing upgrades behind the user's back.
* Add tests using the built-in mock harness rather than live agent credentials.
* Never hard-code API keys, credentials, or other secrets into the repository.

---

### Sync and CRDT schema

`crates/sync` holds local document storage and durable sync-job bookkeeping. Changes here get particular care because they affect persisted state.

* Discuss changes to the wire format or CRDT schema with the maintainers before implementation, since they affect compatibility between released versions.

---

### Rust Code

Follow standard Rust conventions and the style already established in the codebase.

Use `rustfmt` for formatting and prefer straightforward Rust over unnecessary abstraction. Keep functions and crates focused, and introduce traits when they provide a meaningful boundary rather than simply because a trait is possible.

Handle errors explicitly with the existing `anyhow` and `thiserror` patterns. Do not silently discard errors unless the behavior is intentional and appropriate for the surrounding code.

Preserve useful error context when returning errors. Avoid introducing global state when a narrower scope or explicit dependency is practical.

Public items should have clear names and should only be made public when they are part of an intentional API.

When modifying existing code, prefer consistency with the surrounding implementation over introducing a new style for a single change.

---

### Dependencies

Avoid adding dependencies when the standard library or an existing workspace dependency already provides the required functionality.

When a new dependency is necessary, explain why it is needed in the pull request.

Changes to `Cargo.toml` and `Cargo.lock` should be intentional. Avoid unrelated dependency upgrades in feature or bug-fix pull requests.

Keep the dependency surface of Space under control. New dependencies should provide meaningful value relative to their maintenance, compatibility, and security costs.

---

### Testing

Tests should verify behavior rather than implementation details whenever practical.

Bug fixes should include a regression test when the affected behavior can be tested reliably.

For changes involving the engine, harness drivers, sync, sessions, configuration, or other stateful systems, consider both successful and failure paths.

Not every change requires a new test, particularly for documentation or purely mechanical changes, but contributors should still verify that the change does not introduce regressions.

---

### Commit and Pull Request Titles

Pull request titles should use a Conventional Commit-style format:

* `feat:` for new functionality
* `fix:` for bug fixes
* `docs:` for documentation changes
* `refactor:` for internal restructuring without intentional behavior changes
* `test:` for test changes
* `perf:` for performance improvements
* `chore:` for maintenance and project housekeeping

Examples:

* `feat: add a harness driver for a new agent CLI`
* `fix: resolve crash on startup`
* `docs: update contributing guidelines`
* `refactor: simplify transcript parsing`
* `test: add a workspace doc regression test`
* `perf: reduce transcript rendering overhead`
* `chore: bump dependency versions`

Keep titles concise and describe the result of the change.

---

### Feature Proposals

New features that significantly affect the behavior, architecture, or scope of Space should begin with an issue or design discussion.

A useful feature proposal should explain:

* The problem being solved.
* The expected behavior.
* Why the functionality belongs in Space.
* Any relevant alternatives or existing functionality.
* A proposed implementation, if one already exists.

A proposal does not need to contain a complete technical design. The purpose of the initial discussion is to determine whether the problem and direction are aligned before significant implementation work begins.

Once the direction is agreed upon, implementation can proceed through a pull request.

---

### Issues

Use the appropriate issue template when one is provided.

#### Bug Reports

A bug report should include enough information to reproduce or investigate the problem.

Where relevant, include:

* Space version or commit
* Operating system and architecture
* Steps to reproduce
* Expected behavior
* Actual behavior
* Relevant logs or error messages
* The harness and model in use when relevant

Remove API keys, credentials, personal information, and other sensitive data before posting logs or transcripts.

#### Feature Requests

Feature requests should describe the problem and the desired outcome rather than only prescribing an implementation.

Explain why the current behavior is insufficient and what you would like Space to make possible.

#### Questions

Questions should include the specific behavior, problem, or development task you are trying to understand. Providing relevant context usually makes it easier to give a useful answer.

Incomplete issues may be closed when there is not enough information to investigate the request.

---

### AI-Assisted Contributions

AI-assisted development is allowed.

Contributors are responsible for everything they submit. Code generated or assisted by an AI system must be reviewed, understood, tested, and adapted to the project before it is submitted.

Do not submit large amounts of unreviewed generated code, boilerplate, or documentation simply because it was produced by an AI system.

Contributors should be able to explain the purpose and behavior of their changes and verify that those changes work as described.

The standard is the quality, correctness, and maintainability of the contribution, not the tool used to produce it.

---

### Before Opening a Pull Request

Before submitting a pull request, verify that:

* The change addresses a specific problem or clearly defined improvement.
* Existing issues and relevant parts of the codebase have been checked.
* Significant feature changes have been discussed with maintainers beforehand.
* Rust files are formatted with `cargo fmt`.
* `cargo test --workspace` passes.
* `cargo clippy --workspace` does not report new issues.
* `cargo build --workspace` succeeds.
* Relevant behavior has been tested manually when appropriate.
* New or changed behavior has appropriate tests where practical.
* Documentation has been updated when necessary.
* No secrets or sensitive information are included.
* No unrelated changes are included.
* The pull request explains what changed and how it was verified.

Thank you for contributing to Space.
