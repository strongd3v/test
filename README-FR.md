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
  <a href="README-DE.md">Deutsch</a> | <a href="README.md">English</a> | <a href="README-ES.md">Español</a> | <strong>Français</strong> | <a href="README-PT.md">Português</a> | <a href="README-TR.md">Türkçe</a> | <a href="README-AR.md">العربية</a> | <a href="README-HI.md">हिन्दी</a> | <a href="README-ZH.md">中文</a> | <a href="README-JA.md">日本語</a>
</p>

Space est un contrôleur de bureau natif pour agents de codage : une seule interface pour les CLIs d'agents, les outils, les projets et les appareils que vous utilisez chaque jour. Plutôt que de vivre dans un onglet de navigateur ou de vous enfermer chez un seul fournisseur, Space pilote les agents de codage déjà installés sur vos machines, de Claude Code à Cursor ; chaque session, artefact et réglage reste sur votre propre matériel.

Derrière l'interface, Space associe une application native rapide à un vrai moteur : un service local qui possède les sessions, les transcripts, les terminaux et l'état de l'espace de travail, accessible depuis l'app ou un client MCP. Les builds Linux, macOS et Windows sont publiés sur la page [Releases](https://github.com/strongd3v/test/releases).

---

### Pourquoi Space

* **Apportez votre propre agent.** Antigravity, Claude Code, Codex, Cursor, Devin, fx, Grok, Hermes, Muse Code, OpenCode et Pi sont tous pilotés via leurs protocoles natifs ou ACP derrière une même interface. Vous choisissez le harness et le modèle par conversation, tandis que Space garde le flux de travail cohérent, quel que soit l'agent derrière.

* **Natif par défaut.** Space est écrit en Rust et rendu avec GPUI, le framework d'interface derrière Zed, vendored et étendu ici. Pas d'Electron, pas de wrapper navigateur : les transcripts en streaming, les diffs avec coloration syntaxique et les panneaux de terminal en direct restent fluides même sur de longues sessions.

* **Local d'abord.** Sessions, réglages et état de l'espace de travail vivent sur disque dans votre répertoire de données, et le moteur démarre dans le processus sans exiger de compte. Débrancher le réseau n'arrête pas votre travail ; la synchronisation est quelque chose à quoi vous adhérez volontairement, pas une dépendance.

* **Conçu pour survivre au vrai travail.** Les exécutions s'appuient sur une file de commandes durable et un journal de reprise après incident : une conversation interrompue reprend là où elle s'était arrêtée au lieu de perdre sa progression à un redémarrage.

* **Un espace de travail complet, pas une boîte de discussion.** Autour de la conversation, Space livre un explorateur de fichiers, un terminal PTY intégré, des vues de changements git et de diff, un panneau de pull requests, un navigateur intégré et une palette de commandes qui recherche les actions, les conversations, les fichiers de l'espace de travail et le contenu des fichiers.

* **La voix, quand vous le souhaitez.** La dictée et une scène vocale plein écran fonctionnent sur un modèle local de speech-to-text, afin que votre audio n'ait jamais à quitter la machine.

---

### Agents pris en charge

Space parle à chaque CLI d'agent via le protocole qu'elle utilise réellement :

* **Antigravity** (Google) : piloté via ACP.
* **Claude Code** (Anthropic) : piloté via son protocole de flux natif.
* **Codex** (OpenAI) : piloté via son protocole app-server natif.
* **Cursor** : la CLI Cursor, pilotée via l'Agent Client Protocol (ACP).
* **Devin** (Cognition) : piloté via ACP.
* **fx** (Vercel) : piloté via son serveur ACP natif.
* **Grok** (xAI) : l'agent Grok Build, piloté via ACP.
* **Hermes** (Nous Research) : piloté via ACP.
* **Muse Code** (Meta) : piloté nativement via le Muse Session Protocol (MSP).
* **OpenCode** (SST) : piloté nativement via son protocole serveur HTTP/SSE.
* **Pi** : piloté via son protocole JSONL RPC natif.

Chaque harness reste une installation indépendante : Space affiche les avis de mise à jour dans la barre latérale au lieu de modifier quoi que ce soit derrière votre dos, et le sélecteur de harness et de modèle du composer change d'agent par conversation : planifier avec un modèle, implémenter avec un autre.

---

### Conversations

Space enregistre chaque session localement pour que le développement puisse s'étendre sur plusieurs jours au lieu de mourir dans une fenêtre : l'historique est recherchable et relançable, les conversations peuvent être renommées ou bifurquées en discussions latérales pour explorer une alternative sans perdre le fil d'origine, et les brouillons survivent aux redémarrages.

Le transcript en lui-même est virtualisé et analysé de façon incrémentale : markdown, coloration syntaxique, mathématiques LaTeX, groupes d'outils pliés et streaming collé en bas restent fluides à des milliers de messages. Le flux Send/Steer/Stop du composer vous permet de mettre un message en file derrière un tour en cours, de rediriger l'agent en plein vol avec une correction ou de l'interrompre franchement. Quand un agent vous pose une question, la réponse se fait en ligne, et les images ou captures d'écran peuvent être jointes exactement là où se déroule la conversation.

---

### Espace de travail

Space considère le projet comme une partie de la conversation plutôt que comme un programme séparé :

* **Panneau fichiers** : explorateur avec statut git, renommages et modifications rapides.
* **Terminal** : onglets PTY intégrés avec relecture limitée, rendus nativement.
* **Changements** : diffs côte à côte et vue des changements git, à côté du chat.
* **Pull requests** : un panneau dédié à la revue, aux commentaires et au suivi des todo.
* **Navigateur** : un vrai webview à onglets près du travail, pour les aperçus et la documentation.
* **Appshots** : capturez n'importe quelle fenêtre de votre bureau et attachez-la avec le contexte d'accessibilité de l'application, pour que l'agent voie ce que vous voyez.
* **Palette de commandes** : une frappe pour chercher parmi les actions, les conversations, les fichiers de l'espace de travail et le contenu des fichiers.

Le panneau droit bascule entre fichiers, diffs, terminaux, discussions latérales, transcripts de sous-agents et navigateur sans faire sortir la conversation du champ de vision.

---

### Voix

La dictée et une scène vocale plein écran fonctionnent sur un modèle local de speech-to-text : vous parlez, Space écrit, et les sous-titres et la transcription en direct s'affichent dans la même interface. Votre audio ne quitte jamais la machine.

---

### MCP

`space mcp` exécute un serveur stdio du Model Context Protocol qui sert de proxy à l'API locale du moteur, laissant les outils externes et d'autres agents piloter vos sessions Space via une interface standardisée.

---

### Sécurité

Vous avez trouvé une vulnérabilité ? Merci de la signaler en privé plutôt que d'ouvrir une issue publique : le processus de signalement et les coordonnées se trouvent dans [SECURITY.md](./SECURITY.md).

---

### Contribuer

Space est développé en plein jour et les contributions sont bienvenues. Les drivers de harness pour d'autres CLIs d'agents, les correctifs du transcript et du terminal, le packaging et la documentation sont tous utiles. Les pull requests plus petites et ciblées sont les plus faciles à relire.

Avant de commencer un changement important, lisez les [consignes de contribution](./CONTRIBUTING.md) et ouvrez une issue pour discuter de l'orientation, afin que le travail reste dans l'axe du projet : local d'abord, bureau natif. Toute personne participant accepte de suivre le [Code de conduite](./CODE_OF_CONDUCT.md).

---

### Sponsor

Space est construit et maintenu par une petite équipe, et le parrainage aide à ce qu'il le reste : financer le moteur natif et davantage de harness d'agents. Si Space fait gagner du temps à votre équipe, contactez [space@pentomake.com](mailto:space@pentomake.com) pour devenir sponsor.

---

### Licence

Space est licencié sous la licence [GPL-3.0](./LICENSE.md).
