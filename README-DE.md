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
  <strong>Deutsch</strong> | <a href="README.md">English</a> | <a href="README-ES.md">Español</a> | <a href="README-FR.md">Français</a> | <a href="README-PT.md">Português</a> | <a href="README-TR.md">Türkçe</a> | <a href="README-AR.md">العربية</a> | <a href="README-HI.md">हिन्दी</a> | <a href="README-ZH.md">中文</a> | <a href="README-JA.md">日本語</a>
</p>

Space ist ein nativer Desktop-Controller für Coding-Agents: eine einzige Oberfläche für die Agent-CLIs, Tools, Projekte und Geräte, mit denen man täglich arbeitet. Statt in einem Browser-Tab zu leben oder an einen einzigen Anbieter gebunden zu sein, steuert Space die Coding-Agents, die bereits auf deinen Maschinen installiert sind, von Claude Code bis Cursor; jede Session, jedes Artefakt und jede Einstellung bleibt auf deiner eigenen Hardware.

Unter der Oberfläche kombiniert Space eine schnelle native App mit einer echten Engine: einem lokalen Dienst, der Sessions, Transkripte, Terminals und den Zustand des Arbeitsbereichs verwaltet und von der App oder einem MCP-Client erreichbar ist. Builds für Linux, macOS und Windows erscheinen auf der [Releases](https://github.com/strongd3v/test/releases)-Seite.

---

### Warum Space

* **Bring deinen eigenen Agent mit.** Antigravity, Claude Code, Codex, Cursor, Devin, fx, Grok, Hermes, Muse Code, OpenCode und Pi werden allesamt über ihre nativen Protokolle oder ACP hinter einer Oberfläche angetrieben. Du wählst Harness und Modell pro Konversation, während Space den Workflow konsistent hält, egal welcher Agent dahinter arbeitet.

* **Nativer Aufbau.** Space ist in Rust geschrieben und mit GPUI gerendert, dem UI-Framework hinter Zed, das in dieses Projekt gepflegt (vendored) und erweitert wurde. Kein Electron, kein Browser-Wrapper: Streaming-Transkripte, Diffs mit Syntaxhervorhebung und live Terminal-Panels bleiben auch bei langen Sessions flüssig.

* **Local-first.** Sessions, Einstellungen und Arbeitsbereich-State liegen auf der Festplatte in deinem eigenen Datenverzeichnis, und die Engine startet im Prozess, ohne ein Konto zu verlangen. Das Trennen des Netzes stoppt deine Arbeit nicht; Synchronisierung ist etwas, dem du zustimmst, keine Abhängigkeit.

* **Gebaut für echte Arbeit.** Läufe fahren auf einer dauerhaften Befehlswarteschlange und einem Crash-Recovery-Journal: ein unterbrochenes Gespräch setzt dort fort, wo es aufgehört hat, statt Fortschritt durch einen Neustart zu verlieren.

* **Ein vollständiger Arbeitsbereich, keine Chatbox.** Neben dem Gespräch bietet Space einen Datei-Explorer, ein eingebettetes PTY-Terminal, Git-Change- und Diff-Ansichten, ein Pull-Request-Panel, einen eingebetteten Browser und eine Befehlspalette, die Aktionen, Konversationen, Arbeitsbereichsdateien und Dateiinhalte durchsucht.

* **Sprache, wann immer du sie willst.** Diktat und eine Vollfenster-Sprachansicht laufen auf einem lokalen Speech-to-Text-Modell, damit dein Audio niemals den Rechner verlassen muss.

---

### Unterstützte Agents

Space spricht jede Agent-CLI über das Protokoll an, das sie tatsächlich verwendet:

* **Antigravity** (Google): über ACP angetrieben.
* **Claude Code** (Anthropic): über das native Streaming-Protokoll angetrieben.
* **Codex** (OpenAI): über das native App-Server-Protokoll angetrieben.
* **Cursor**: die Cursor-CLI, über das Agent Client Protocol (ACP) angetrieben.
* **Devin** (Cognition): über ACP angetrieben.
* **fx** (Vercel): über seinen nativen ACP-Server angetrieben.
* **Grok** (xAI): der Grok Build Agent, über ACP angetrieben.
* **Hermes** (Nous Research): über ACP angetrieben.
* **Muse Code** (Meta): nativ über das Muse Session Protocol (MSP) angetrieben.
* **OpenCode** (SST): nativ über das HTTP/SSE-Serverprotokoll angetrieben.
* **Pi**: über das native JSONL-RPC-Protokoll angetrieben.

Jeder Harness bleibt eine eigenständige Installation: Space zeigt Update-Hinweise in der Seitenleiste an und verändert nichts hinter deinem Rücken; die Harness- und Modellauswahl des Composers wechselt pro Konversation den Agent, sodass die Planung mit einem Modell laufen kann und die Implementierung mit einem anderen.

---

### Konversationen

Space speichert jede Session lokal, damit sich Entwicklung über Tage erstrecken kann, statt in einem Fenster to enden: Der Verlauf ist durchsuchbar und fortsetzbar, Konversationen lassen sich umbenennen oder in Side-Chats verzweigen, um eine Alternative zu erkunden, ohne den ursprünglichen Faden zu verlieren, und Entwürfe überstehen Neustarts.

Das Transkript selbst ist virtualisiert und wird inkrementell geparst: Markdown, Syntaxhervorhebung, LaTeX-Mathe, gefaltete Tool-Gruppen und am Fuß klebendes Streaming bleiben bei Tausenden von Nachrichten flüssig. Der Send/Steer/Stop-Flow des Composers lässt dich eine Nachricht hinter einem laufenden Turn einreihen, den Agent unterwegs mit einer Korrektur umlenken oder ihn einfach unterbrechen. Wenn ein Agent dir eine Frage stellt, passiert die Antwort inline; Bilder oder aufgenommene Screenshots lassen sich genau dort anhängen, wo das Gespräch läuft.

---

### Arbeitsbereich

Space behandelt das Projekt als Teil der Konversation, nicht als separates Programm:

* **Dateien-Panel**: Explorer mit Git-Status, Umbenennungen und schnellen Bearbeitungen.
* **Terminal**: eingebettete PTY-Tabs mit begrenztem Replay, nativ gerendert.
* **Änderungen**: Diffs nebeneinander und die Git-Changes-Ansicht direkt neben dem Chat.
* **Pull Requests**: ein eigenes Panel für Review, Kommentare und Todo-Tracking.
* **Browser**: ein echter Webview mit Tabs neben der Arbeit, für Vorschauen und Doku.
* **Appshots**: Erfasse ein beliebiges Fenster auf deinem Desktop und hänge es mit dem Accessibility-Kontext der App an, sodass der Agent sieht, was du siehst.
* **Befehlspalette**: ein Tastendruck über Aktionen, Konversationen, Arbeitsbereichsdateien und Dateiinhalte.

Das rechte Panel wechselt zwischen Dateien, Diffs, Terminals, Side-Chats, Transkripten von Sub-Agents und dem Browser, ohne das Gespräch aus dem Blick zu rücken.

---

### Stimme

Diktat und eine Vollfenster-Sprachansicht laufen auf einem lokalen Speech-to-Text-Modell: Du sprichst, Space tippt, und Untertitel sowie Live-Transkription werden in derselben Oberfläche gerendert. Dein Audio verlässt den Rechner nie.

---

### MCP

`space mcp` führt einen Model Context Protocol-stdio-Server aus, der die lokale API der Engine als Proxy dient, sodass externe Tools und andere Agents deine Space-Sessions über eine standardisierte Schnittstelle steuern können.

---

### Sicherheit

Hast du eine Schwachstelle gefunden, melde sie bitte privat statt über ein öffentliches Issue; der Meldeprozess und die Kontaktdaten stehen in [SECURITY.md](./SECURITY.md).

---

### Beiträge

Space wird offen entwickelt, und Beiträge sind willkommen. Harness-Treiber für weitere Agent-CLIs, Korrekturen an Transkript und Terminal, Paketierung und Dokumentation sind alles nützlich. Kleinere, fokussierte Pull Requests sind am leichtesten zu reviewen.

Bevor du eine größere Änderung beginnst, lies die [Beitragsrichtlinien](./CONTRIBUTING.md) und eröffne ein Issue, um die Richtung zu besprechen, damit die Arbeit am lokalen, nativen Desktop-Fokus des Projekts ausgerichtet bleibt. Alle Teilnehmenden stimmen zu, den [Verhaltenskodex](./CODE_OF_CONDUCT.md) einzuhalten.

---

### Sponsor

Space wird von einem kleinen Team entwickelt und gepflegt, und Sponsoring hält das bestehen: Es finanziert die native Engine und mehr Agent-Harnesses. Wenn Space deinem Team Zeit spart, wende dich an [space@pentomake.com](mailto:space@pentomake.com), um Sponsor zu werden.

---

### Lizenz

Space steht unter der [GPL-3.0-Lizenz](./LICENSE.md).
