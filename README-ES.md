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
  <a href="README-DE.md">Deutsch</a> | <a href="README.md">English</a> | <strong>Español</strong> | <a href="README-FR.md">Français</a> | <a href="README-PT.md">Português</a> | <a href="README-TR.md">Türkçe</a> | <a href="README-AR.md">العربية</a> | <a href="README-HI.md">हिन्दी</a> | <a href="README-ZH.md">中文</a> | <a href="README-JA.md">日本語</a>
</p>

Space es un controlador de escritorio nativo para agentes de código: una única interfaz para las CLIs de agentes, herramientas, proyectos y dispositivos con los que trabajas a diario. En vez de vivir en una pestaña del navegador o encerrarte con un solo proveedor, Space maneja los agentes de código ya instalados en tus máquinas, de Claude Code a Cursor, mientras cada sesión, artefacto y ajuste queda en tu propio hardware.

Bajo la interfaz, Space combina una aplicación nativa rápida con un motor de verdad: un servicio local que posee sesiones, transcripciones, terminales y el estado del espacio de trabajo, accesible desde la app o un cliente MCP. Las compilaciones para Linux, macOS y Windows se publican en la página de [Releases](https://github.com/strongd3v/test/releases).

---

### Por qué Space

* **Trae tu propio agente.** Antigravity, Claude Code, Codex, Cursor, Devin, fx, Grok, Hermes, Muse Code, OpenCode y Pi se manejan todos mediante sus protocolos nativos o ACP detrás de una única interfaz. Escoges el harness y el modelo por conversación, y Space mantiene el flujo de trabajo consistente sin importar qué agente esté detrás.

* **Nativo por defecto.** Space está escrito en Rust y renderizado con GPUI, el framework de interfaz detrás de Zed, vendored y extendido aquí. Sin Electron, sin envoltorio de navegador: las transcripciones en streaming, los diff con resaltado de sintaxis y los paneles de terminal en vivo quedan fluidos en sesiones largas.

* **Local primero.** Las sesiones, los ajustes y el estado del espacio de trabajo viven en disco en tu propio directorio de datos, y el motor arranca dentro del proceso sin pedir una cuenta. Desenchufar la red no detiene tu trabajo; la sincronización es algo que eliges activar, no una dependencia.

* **Hecho para aguantar trabajo real.** Las ejecuciones viajan sobre una cola de comandos duradera y un diario de recuperación tras fallo: una conversación interrumpida retoma donde se quedó, sin perder el progreso por un reinicio.

* **Un espacio de trabajo completo, no una caja de chat.** Junto a la conversación, Space incluye un explorador de archivos, un terminal PTY integrado, vistas de cambios de git y diff, un panel de pull requests, un navegador integrado y una paleta de comandos que busca acciones, conversaciones, archivos del espacio de trabajo y contenido de archivos.

* **Voz, cuando quieras.** El dictado y una escena de voz a pantalla completa funcionan con un modelo local de speech-to-text, para que tu audio nunca tenga que salir de la máquina.

---

### Agentes compatibles

Space habla con cada CLI de agente mediante el protocolo que realmente usa:

* **Antigravity** (Google): manejado mediante ACP.
* **Claude Code** (Anthropic): manejado mediante su protocolo de streaming nativo.
* **Codex** (OpenAI): manejado mediante su protocolo app-server nativo.
* **Cursor**: la CLI de Cursor, manejada mediante el Agent Client Protocol (ACP).
* **Devin** (Cognition): manejado mediante ACP.
* **fx** (Vercel): manejado mediante su servidor ACP nativo.
* **Grok** (xAI): el agente Grok Build, manejado mediante ACP.
* **Hermes** (Nous Research): manejado mediante ACP.
* **Muse Code** (Meta): manejado nativamente mediante el Muse Session Protocol (MSP).
* **OpenCode** (SST): manejado nativamente mediante su protocolo de servidor HTTP/SSE.
* **Pi**: manejado mediante su protocolo nativo JSONL RPC.

Cada harness queda como instalación independiente: Space muestra avisos de actualización en la barra lateral en lugar de cambiar nada a tus espaldas, y el selector de harness y modelo del composer cambia de agente por conversación: planificar con un modelo, implementar con otro.

---

### Conversaciones

Space guarda cada sesión en local para que el desarrollo pueda avanzar a lo largo de los días en vez de morir dentro de una ventana: el historial es buscable y reanudable, las conversaciones pueden renombrarse o bifurcarse en chats laterales para explorar una alternativa sin perder el hilo original, y los borradores sobreviven a los reinicios.

La transcripción en sí está virtualizada y se analiza incrementalmente: markdown, resaltado de sintaxis, matemáticas LaTeX, grupos de herramientas plegados y el flujo pegado abajo se quedan fluidos con miles de mensajes. El flujo Send/Steer/Stop del composer te permite poner un mensaje en cola detrás de un turno en marcha, redirigir al agente en pleno vuelo con una corrección o interrumpirlo directamente. Cuando un agente te formula una pregunta, la respuesta ocurre en línea, y las imágenes o capturas pueden adjuntarse precisamente donde está la conversación.

---

### Espacio de trabajo

Space trata el proyecto como parte de la conversación y no como un programa separado:

* **Panel de archivos**: explorador con estado de git, renombrados y ediciones rápidas.
* **Terminal**: pestañas PTY integradas con repetición limitada, renderizadas de forma nativa.
* **Cambios**: diffs en dos columnas y la vista de cambios de git, junto al chat.
* **Pull requests**: un panel dedicado a la revisión, los comentarios y el seguimiento de pendientes.
* **Navegador**: un webview real con pestañas junto al trabajo, para previsualizaciones y documentación.
* **Appshots**: captura cualquier ventana de tu escritorio y la adjunta con el contexto de accesibilidad de la app, para que el agente vea lo que ves.
* **Paleta de comandos**: una pulsación sobre las acciones, las conversaciones, los archivos del espacio de trabajo y el contenido de los archivos.

El panel derecho cambia entre archivos, diffs, terminales, chats laterales, transcripciones de subagentes y el navegador sin sacar la conversación de la vista.

---

### Voz

El dictado y una escena de voz a pantalla completa funcionan con un modelo local de speech-to-text: tú hablas, Space escribe, y los subtítulos y la transcripción en vivo se renderizan en la misma interfaz. Tu audio nunca sale de la máquina.

---

### MCP

`space mcp` ejecuta un servidor stdio del Model Context Protocol que hace de proxy de la API local del motor, permitiendo que herramientas externas y otros agentes manejen tus sesiones de Space mediante una interfaz estandarizada.

---

### Seguridad

¿Has encontrado una vulnerabilidad? Por favor, repórtala en privado en vez de abrir un issue público: el proceso de reporte y los datos de contacto están en [SECURITY.md](./SECURITY.md).

---

### Contribuir

Space se desarrolla abiertamente y las contribuciones son bienvenidas. Los controladores de harness para más CLIs de agentes, las correcciones a la transcripción y al terminal, el empaquetado y la documentación son todos útiles. Los pull requests pequeños y enfocados son los más fáciles de revisar.

Antes de arrancar un cambio grande, lee la [guía de contribución](./CONTRIBUTING.md) y abre un issue para discutir el rumbo, para que el trabajo se mantenga alineado con el foco del proyecto: local primero, escritorio nativo. Quien participa acepta seguir el [Código de conducta](./CODE_OF_CONDUCT.md).

---

### Patrocinio

Space se construye y mantiene con un equipo pequeño, y el patrocinio ayuda a seguir así: financia el motor nativo y más harness de agentes. Si Space ahorra tiempo a tu equipo, escribe a [space@pentomake.com](mailto:space@pentomake.com) para convertirte en patrocinador.

---

### Licencia

Space está licenciado bajo la licencia [GPL-3.0](./LICENSE.md).
