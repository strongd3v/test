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
  <a href="README-DE.md">Deutsch</a> | <a href="README.md">English</a> | <a href="README-ES.md">Español</a> | <a href="README-FR.md">Français</a> | <a href="README-PT.md">Português</a> | <a href="README-TR.md">Türkçe</a> | <a href="README-AR.md">العربية</a> | <a href="README-HI.md">हिन्दी</a> | <a href="README-ZH.md">中文</a> | <strong>日本語</strong>
</p>

Space は、コーディングエージェントのためのネイティブなデスクトップコントローラです。日常的に使うエージェント CLI、ツール、プロジェクト、デバイスを 1 つのインターフェースにまとめます。ブラウザのタブで暮らしたり、1 社のプロバイダに固定されたりする代わりに、Space はマシンにすでにインストール済みのコーディングエージェントを、Claude Code から Cursor まで操作し、セッション・成果物・設定はすべてあなた自身のハードウェア上に保たれます。

インターフェースの下では、Space は高速なネイティブアプリと本物のエンジンを組み合わせます。セッション、転記（transcript）、ターミナル、ワークスペースの状態を所有するローカルなサービスで、アプリから、または MCP クライアントからアクセスできます。Linux、macOS、Windows のビルドは [Releases](https://github.com/strongd3v/test/releases) ページで公開されています。

---

### なぜ Space なのか

* **好きなエージェントを持ち込む。** Antigravity、Claude Code、Codex、Cursor、Devin、fx、Grok、Hermes、Muse Code、OpenCode、Pi のすべてが、それぞれのネイティブプロトコルまたは ACP で、単一のインターフェースの背後で操作されます。会話ごとに harness とモデルを選べます。その背後にどのエージェントがいても、Space がワークフローの一貫性を保ちます。

* **ネイティブが基本。** Space は Rust で書かれ、GPUI で描画されます。GPUI は Zed の背後にある UI フレームワークで、このプロジェクトに vendored され拡張されています。Electron もブラウザのラッパーもありません。ストリーミングされる転記、シンタックスハイライト付きの diff、ライブなターミナルペインは、長いセッションでも滑らかさを保ちます。

* **ローカルファースト。** セッション、設定、ワークスペースの状態は、あなた自身のデータディレクトリ配下のディスクに置かれ、エンジンはアカウントなしでインプロセスとして起動します。ネットワークを切り離しても仕事は止まりません。同期は自分で選んで有効にするもので、依存先ではありません。

* **現場の実務に耐えるつくり。** 実行は耐久性のあるコマンドキューとクラッシュリカバリジャーナルの上を走ります。中断された会話は、進捗をリスタートに失う代わりに、中断したところから再開します。

* **チャットボックスではなく、完全なワークスペース。** 会話の隣に、Space はファイルエクスプローラ、組み込みの PTY ターミナル、git の変更と diff ビュー、プルリクエストペイン、組み込みブラウザ、そして操作・会話・ワークスペースのファイル・ファイルの中身を検索するコマンドパレットを備えています。

* **欲しいときに音声で。** ディクテーションと全ウィンドウの音声ステージはローカルな音声認識モデルで動き、あなたの音声がマシンの外に出る必要はありません。

---

### 対応エージェント

Space は各エージェント CLI と、その CLI が実際に使うプロトコルで通信します：

* **Antigravity**（Google）：ACP で操作します。
* **Claude Code**（Anthropic）：ネイティブのストリーミングプロトコルで操作します。
* **Codex**（OpenAI）：ネイティブの app-server プロトコルで操作します。
* **Cursor**：Cursor CLI、Agent Client Protocol（ACP）で操作します。
* **Devin**（Cognition）：ACP で操作します。
* **fx**（Vercel）：ネイティブの ACP サーバーで操作します。
* **Grok**（xAI）：Grok Build エージェント、ACP で操作します。
* **Hermes**（Nous Research）：ACP で操作します。
* **Muse Code**（Meta）：Muse Session Protocol（MSP）でネイティブに操作します。
* **OpenCode**（SST）：HTTP/SSE サーバープロトコルでネイティブに操作します。
* **Pi**：ネイティブの JSONL RPC プロトコルで操作します。

各 harness は独立したインストールのまま保たれます。Space はサイドバーで更新の通知を出すだけで、背後で勝手に何かを変えません。Composer の harness とモデルの選択肢を使えば、会話ごとにエージェントを切り替えられます。計画は 1 つのモデルで、実装は別のモデルで、という進め方ができます。

---

### 会話

Space はすべてのセッションをローカルに保存するため、開発はひとつのウィンドウの中で終わるのではなく、何日にも渡って進められます。履歴は検索・再開でき、会話の名前を変えたり、元のスレッドを失わずに代替案を調べるためにサイドチャットへフォークしたりできます。下書きは再起動しても生き残ります。

転記自体は仮想化され、逐次的にパースされます。markdown、シンタックスハイライト、LaTeX の数式、折りたたまれたツールグループ、下端に貼り付くストリーミングは、数千メッセージでも滑らかです。Composer の Send/Steer/Stop の流れにより、実行中のターンの後ろにメッセージを積んでおいたり、飛行中のエージェントを訂正で操り直したり、そのまま遮断したりできます。エージェントがあなたに質問をしたら、回答はインラインで完結し、画像や撮ったスクリーンショットを会話の場所へそのまま添付できます。

---

### ワークスペース

Space はプロジェクトを、独立した別プログラムではなく会話の一部として扱います：

* **ファイルペイン**: git ステータス、リネーム、クイック編集つきのエクスプローラ。
* **ターミナル**: 有限のリプレイを持つ組み込み PTY タブ、ネイティブ描画。
* **変更**: 並んだ diff と git 変更ビュー、チャットの横に。
* **プルリクエスト**: レビュー・コメント・todo 追跡専用のペイン。
* **ブラウザ**: 作業の隣にある本物のタブ付き webview。プレビューとドキュメント用。
* **Appshots**: デスクトップの任意のウィンドウを撮って、アプリのアクセシビリティ文脈つきで添付します。エージェントがあなたの見ているものを見られます。
* **コマンドパレット**: 1 回のキー操作で操作・会話・ワークスペースのファイル・ファイルの中身を検索。

右側のペインは、会話を見失うことなく、ファイル・diff・ターミナル・サイドチャット・サブエージェントの転記・ブラウザを切り替えます。

---

### 音声

ディクテーションと全ウィンドウの音声ステージは、ローカルの音声認識モデルで動きます。あなたが話すと Space が打ち、字幕とライブ転写も同じインターフェースに描かれます。音声はマシンから決して出て行きません。

---

### MCP

`space mcp` は、エンジンのローカル API をプロキシする Model Context Protocol stdio サーバーを動かし、外部ツールや他のエージェントが標準化されたインターフェース経由であなたの Space セッションを操作できるようにします。

---

### セキュリティ

脆弱性を見つけた場合は、公開 issue ではなく非公開で報告してください。報告の流れと連絡先は [SECURITY.md](./SECURITY.md) にあります。

---

### コントリビュート

Space はオープンに開発されており、コントリビューションを歓迎します。新しいエージェント CLI 向けの harness ドライバ、転記とターミナルの修正、パッケージング、ドキュメント、すべて有効です。小さく焦点を絞ったプルリクエストがいちばんレビューしやすいです。

大きな変更を始める前に、[コントリビューションガイド](./CONTRIBUTING.md)を読み、issue を開いて方向性を議論してください。プロジェクトの軸であるローカルファースト・ネイティブデスクトップと合わせます。参加するすべての人は[行動規範](./CODE_OF_CONDUCT.md)の遵守に同意するものとします。

---

### スポンサー

Space は小さなチームによって開発・維持されています。スポンサーシップはそれを支えます。ネイティブエンジンと、より多くのエージェント harness の資金になるからです。Space がチームの時間を節約しているなら、スポンサーになるために [space@pentomake.com](mailto:space@pentomake.com) までご連絡ください。

---

### ライセンス

Space は [GPL-3.0 ライセンス](./LICENSE.md) のもとで公開されています。
