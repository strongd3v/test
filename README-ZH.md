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
  <a href="README-DE.md">Deutsch</a> | <a href="README.md">English</a> | <a href="README-ES.md">Español</a> | <a href="README-FR.md">Français</a> | <a href="README-PT.md">Português</a> | <a href="README-TR.md">Türkçe</a> | <a href="README-AR.md">العربية</a> | <a href="README-HI.md">हिन्दी</a> | <strong>中文</strong> | <a href="README-JA.md">日本語</a>
</p>

Space 是一个面向编码代理的原生桌面控制中心：把你日常使用的代理 CLI、工具、项目和设备集中到同一个界面。与其住在浏览器标签页里，或被锁死在单一供应商上，不如让 Space 驱动你机器上已经装好的编码代理，从 Claude Code 到 Cursor，同时每场会话、每个产物和每项设置都保留在你自己的硬件上。

在界面之下，Space 将一个快速的原生应用与一个真正的引擎配对：一个本地服务，掌管会话、转录、终端和工作区状态，可从应用或 MCP 客户端访问。Linux、macOS 和 Windows 的构建发布在 [Releases](https://github.com/strongd3v/test/releases) 页面上。

---

### 为什么选择 Space

* **自带代理。** Antigravity、Claude Code、Codex、Cursor、Devin、fx、Grok、Hermes、Muse Code、OpenCode 和 Pi 全部通过它们的原生协议或 ACP 在同一个界面背后被驱动。你按会话选择 harness 和模型，而无论背后是哪个代理，Space 都让工作流程保持一致。

* **原生为本。** Space 用 Rust 编写，并以 GPUI 渲染；GPUI 是 Zed 背后的界面框架，已在项目中 vendored 并扩展。没有 Electron，没有浏览器外壳：流式转录、带语法高亮的 diff 和实时终端面板即使面对长会话依然流畅。

* **本地优先。** 会话、配置和工作区状态保存在你自己的数据目录下的磁盘里，引擎随应用启动，不需要账号。断网不会中断你的工作：同步是你主动选择加入的内容，而不是一种依赖。

* **为真实工作而构建。** 执行运行靠的是持久化的命令队列和崩溃恢复日志，被打断的对话会从断点继续，而不是因重启而丢失进度。

* **是完整的工作区，不是聊天框。** 在对话旁边，Space 附带了文件浏览器、内嵌 PTY 终端、git 变更和 diff 视图、拉取请求面板、内嵌浏览器，以及一支可搜索操作、会话、工作区文件和文件内容的命令面板。

* **需要时就用语音。** 听写和全窗口语音场景运行在本地语音转文字模型上，你的音频无需完全离开这台机器。

---

### 支持的代理

Space 用每个代理 CLI 真正使用的协议与它通信：

* **Antigravity**（Google）：通过 ACP 驱动。
* **Claude Code**（Anthropic）：通过其原生流式协议驱动。
* **Codex**（OpenAI）：通过其原生 app-server 协议驱动。
* **Cursor**：Cursor CLI，通过 Agent Client Protocol（ACP）驱动。
* **Devin**（Cognition）：通过 ACP 驱动。
* **fx**（Vercel）：通过其原生 ACP 服务器驱动。
* **Grok**（xAI）：Grok Build 代理，通过 ACP 驱动。
* **Hermes**（Nous Research）：通过 ACP 驱动。
* **Muse Code**（Meta）：通过 Muse Session Protocol（MSP）原生驱动。
* **OpenCode**（SST）：通过其 HTTP/SSE 服务器协议原生驱动。
* **Pi**：通过其原生 JSONL RPC 协议驱动。

每个 harness 都保持独立安装：Space 只会在侧边栏提示更新，而不会在你的背后擅自改动什么；composer 里的 harness 和模型选择器支持按会话切换代理：用一个模型做规划，用另一个模型做实现。

---

### 对话

Space 将每场会话保存在本地，开发因此可以跨越数天推进，而不是死在一个窗口里：历史可搜索、可恢复，对话可以重命名或分叉为侧边会话，便于探索另一种方案而不丢原线程，草稿在重启之后依然存在。

转录本身经过虚拟化与增量解析：markdown、语法高亮、LaTeX 数学、折叠的工具组和贴底流式渲染在成千上万条消息下依然流畅。Composer 的 Send/Steer/Stop 流程让你把消息排入正在运行的回合之后的队列，中途用一句纠正来改变代理的方向，或者直接打断它。当代理向你提问时，回答直接在会话中完成，图片或截屏也可以就地附着在对话上。

---

### 工作区

Space 把项目当作对话的一部分，而不是另一个独立程序：

* **文件面板**：带 git 状态、重命名和快速编辑的浏览。
* **终端**：内嵌的 PTY 标签页，带有限回放，原生渲染。
* **变更**：并排 diff 和 git 变更视图，就在聊天旁边。
* **Pull request**：一个专门用于评审、评论和待办跟踪的面板。
* **浏览器**：与工作并置的真实多标签 webview，用于预览和文档。
* **Appshots**：截取桌面上的任意窗口，并附带应用的辅助功能上下文，让代理看到你所看到的。
* **命令面板**：一键搜索操作、会话、工作区文件和文件内容。

右侧面板可以在文件、diff、终端、侧边会话、子代理转录和浏览器之间切换，而不让对话离开视线。

---

### 语音

听写和全窗口语音场景运行在本地语音转文字模型上：你负责说，Space 负责打字，字幕和实时转写也在同一个界面中渲染。你的音频永远不会离开这台机器。

---

### MCP

`space mcp` 运行一个 Model Context Protocol stdio 服务器，代理引擎的本地 API，让外部工具和其他代理通过标准化接口驱动你的 Space 会话。

---

### 安全

发现了漏洞？请私下报告而不是开公开 issue：报告流程和联系方式在 [SECURITY.md](./SECURITY.md)。

---

### 参与贡献

Space 在公开状态下开发，欢迎一切贡献。支持更多代理 CLI 的 harness 驱动、转录与终端的修复、打包以及文档都很有价值。小的、聚焦的 pull request 最容易评审。

在开始一个较大的改动之前，请先阅读[贡献指南](./CONTRIBUTING.md)，并开一个 issue 讨论方向，使工作与项目本地优先、原生桌面的定位保持一致。每位参与者都同意遵守[行为准则](./CODE_OF_CONDUCT.md)。

---

### 赞助

Space 由一个小团队构建和维护，赞助能让它保持这样：赞助款项用于原生引擎和更多的代理 harness。如果 Space 为你的团队节省了时间，欢迎通过 [space@pentomake.com](mailto:space@pentomake.com) 联系我们成为赞助者。

---

### 许可证

Space 采用 [GPL-3.0 许可证](./LICENSE.md)。
