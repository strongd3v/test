### IMPORTANT

We do not accept AI generated security reports. We receive a large number of these and we absolutely do not have the resources to review them all. If you submit one you will be banned from the project.

---

### Threat Model

#### Overview

Space is a native desktop application for controlling coding agents. It runs locally on your machine and drives agent CLIs that have access to powerful tools, including shell execution, file operations, and network access. A local engine owns sessions, transcripts, terminals, and workspace state.

#### No Sandbox

Space does **not** sandbox the agents it drives. The permission system exists as a UX feature to help users stay aware of what actions an agent is taking: it prompts for confirmation before executing commands, writing files, and similar operations. It is not designed to provide security isolation.

If you need true isolation, run your agents and workspaces inside a container or virtual machine.

#### Out of Scope

| Category                          | Rationale                                                                                        |
| --------------------------------- | ------------------------------------------------------------------------------------------------- |
| **Sandbox escapes**               | The permission system is not a sandbox (see above)                                                 |
| **LLM provider data handling**    | Data sent to your configured LLM provider is governed by their policies                            |
| **MCP server behavior**           | External MCP servers you configure are outside our trust boundary                                  |
| **Malicious config or workspace** | Users control their own config and workspace; modifying them is not an attack vector                |
| **Agent behavior**                | Bugs or harmful actions originating inside a driven agent CLI belong to that project                 |

---

### Supported Versions

The latest release is the only supported version. Security fixes are published on the releases page, so update to the newest release before reporting a version-specific issue.

---

### Reporting Security Issues

We appreciate your efforts to responsibly disclose your findings, and will make every effort to acknowledge your contributions.

To report a security issue, please use the GitHub Security Advisory ["Report a Vulnerability"](https://github.com/strongd3v/test/security/advisories/new) tab.

The team will send a response indicating the next steps in handling your report. After the initial reply to your report, the security team will keep you informed of the progress towards a fix and full announcement, and may ask for additional information or guidance.

#### Escalation

If you do not receive an acknowledgement of your report within 6 business days, you may send an email to space@pentomake.com
