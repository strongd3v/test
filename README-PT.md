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
  <a href="README-DE.md">Deutsch</a> | <a href="README.md">English</a> | <a href="README-ES.md">Español</a> | <a href="README-FR.md">Français</a> | <strong>Português</strong> | <a href="README-TR.md">Türkçe</a> | <a href="README-AR.md">العربية</a> | <a href="README-HI.md">हिन्दी</a> | <a href="README-ZH.md">中文</a> | <a href="README-JA.md">日本語</a>
</p>

Space é um controlador de desktop nativo para agentes de código: uma única interface para as CLIs de agentes, ferramentas, projetos e dispositivos com os quais você trabalha todos os dias. Em vez de viver numa aba do navegador ou ficar preso a um único provedor, Space conduz os agentes de código já instalados nas suas máquinas, de Claude Code a Cursor, enquanto cada sessão, artefato e ajuste fica no seu próprio hardware.

Por baixo da interface, Space combina um app nativo rápido com um motor de verdade: um serviço local que possui sessões, transcrições, terminais e o estado do espaço de trabalho, acessível pelo app ou por um cliente MCP. As compilações para Linux, macOS e Windows são publicadas na página de [Releases](https://github.com/strongd3v/test/releases).

---

### Por que o Space

* **Traga o seu agente.** Antigravity, Claude Code, Codex, Cursor, Devin, fx, Grok, Hermes, Muse Code, OpenCode e Pi são todos acionados via seus protocolos nativos ou ACP atrás de uma única interface. Você escolhe o harness e o modelo por conversa, enquanto o Space mantém o fluxo de trabalho consistente sem se importar qual agente está atrás.

* **Nativo por padrão.** O Space é escrito em Rust e renderizado com GPUI, o framework de UI por trás do Zed, vendored e estendido neste projeto. Sem Electron, sem invólucro de navegador: transcrições em streaming, diff com destaque de sintaxe e painéis de terminal ao vivo continuam fluidos mesmo em sessões longas.

* **Local primeiro.** Sessões, ajustes e estado do espaço de trabalho vivem no disco sob o seu próprio diretório de dados, e o motor inicia no processo sem exigir conta. Desligar a rede não para o seu trabalho; a sincronização é algo com que você participa por vontade, não uma dependência.

* **Construído para sobreviver ao trabalho real.** As execuções correm sobre uma fila de comandos durável e um diário de recuperação de falhas, assim uma conversa interrompida continua de onde parou em vez de perder o progresso para um reinício.

* **Um espaço de trabalho completo, não uma caixa de chat.** Junto à conversa, o Space traz um explorador de arquivos, um terminal PTY embutido, vistas de mudanças do git e de diff, um painel de pull requests, um navegador integrado e uma paleta de comandos que busca ações, conversas, arquivos do espaço de trabalho e conteúdo de arquivos.

* **Voz, quando você quiser.** O ditado e uma cena de voz em janela cheia rodam sobre um modelo local de speech-to-text, para que o seu áudio nunca precise sair da máquina.

---

### Agentes suportados

Space fala com cada CLI de agente pelo protocolo que ela realmente usa:

* **Antigravity** (Google): acionado via ACP.
* **Claude Code** (Anthropic): acionado via seu protocolo de streaming nativo.
* **Codex** (OpenAI): acionado via seu protocolo nativo app-server.
* **Cursor**: a CLI do Cursor, acionada via Agent Client Protocol (ACP).
* **Devin** (Cognition): acionado via ACP.
* **fx** (Vercel): acionado via seu servidor ACP nativo.
* **Grok** (xAI): o agente Grok Build, acionado via ACP.
* **Hermes** (Nous Research): acionado via ACP.
* **Muse Code** (Meta): acionado nativamente via o Muse Session Protocol (MSP).
* **OpenCode** (SST): acionado nativamente via seu protocolo de servidor HTTP/SSE.
* **Pi**: acionado via seu protocolo nativo JSONL RPC.

Cada harness fica como uma instalação independente: o Space mostra avisos de atualização na barra lateral em vez de mudar nada pelas suas costas, e o seletor de harness e modelo do composer troca de agente por conversa: planejar com um modelo, implementar com outro.

---

### Conversas

O Space guarda cada sessão em local, para que o desenvolvimento avance ao longo dos dias em vez de morrer dentro de uma janela: o histórico é pesquisável e retomável, as conversas podem ser renomeadas ou bifurcadas em conversas laterais para explorar uma alternativa sem perder o fio original, e os rascunhos sobrevivem aos reinícios.

A transcrição por si está virtualizada e é analisada de forma incremental: markdown, destaque de sintaxe, matemática LaTeX, grupos de ferramentas dobrados e o fluxo colado embaixo continuam fluidos com milhares de mensagens. O fluxo Send/Steer/Stop do composer permite colocar uma mensagem na fila atrás de um turno em andamento, redirecionar o agente em pleno voo com uma correção ou interrompê-lo diretamente. Quando um agente lhe faz uma pergunta, a resposta acontece na linha, e imagens ou capturas podem ser anexadas exatamente onde está a conversa.

---

### Espaço de trabalho

O Space trata o projeto como parte da conversação, e não como um programa separado:

* **Painel de arquivos**: explorador com estado do git, renomeações e edições rápidas.
* **Terminal**: abas PTY embutidas com replay delimitado, renderizadas nativamente.
* **Mudanças**: diffs lado a lado e a vista das mudanças do git, junto ao chat.
* **Pull requests**: um painel dedicado à revisão, comentários e acompanhamento de pendências.
* **Navegador**: um webview de verdade com abas junto ao trabalho, para pré-visualizações e docs.
* **Appshots**: captura qualquer janela do seu desktop e a anexa com o contexto de acessibilidade do app, para que o agente veja o que você vê.
* **Paleta de comandos**: um toque de tecla sobre ações, conversas, arquivos do espaço de trabalho e conteúdo de arquivos.

O painel direito alterna entre arquivos, diff, terminais, conversas laterais, transcrições de subagentes e o navegador sem mover a conversação para fora da vista.

---

### Voz

O ditado e uma cena de voz em janela cheia rodam sobre um modelo local de speech-to-text: você fala, o Space digita, e as legendas e a transcrição ao vivo são renderizadas na mesma interface. O seu áudio nunca sai da máquina.

---

### MCP

`space mcp` executa um servidor stdio do Model Context Protocol que age como proxy da API local do motor, deixando ferramentas externas e outros agentes conduzir as suas sessões do Space por uma interface padronizada.

---

### Segurança

Encontrou uma vulnerabilidade? Por favor reporte-a em privado em vez de abrir uma issue pública: o processo de reporte e os contatos estão em [SECURITY.md](./SECURITY.md).

---

### Contribuir

O Space é desenvolvido abertamente e as contribuições são bem-vindas. Drivers de harness para mais CLIs de agentes, correções na transcrição e no terminal, empacotamento e documentação são todos úteis. Pull requests menores e focados são os mais fáceis de revisar.

Antes de começar uma mudança maior, leia as [diretrizes de contribuição](./CONTRIBUTING.md) e abra uma issue para discutir a direção, para que o trabalho se mantenha coerente com o foco do projeto: local primeiro, desktop nativo. Toda pessoa que participa concorda em seguir o [Código de Conduta](./CODE_OF_CONDUCT.md).

---

### Patrocínio

O Space é construído e mantido por uma equipe pequena, e o patrocínio ajuda a mantê-lo assim: financia o motor nativo e mais harnesses de agentes. Se o Space economiza o tempo da sua equipe, entre em contato com [space@pentomake.com](mailto:space@pentomake.com) para se tornar patrocinador.

---

### Licença

O Space está licenciado sob a licença [GPL-3.0](./LICENSE.md).
