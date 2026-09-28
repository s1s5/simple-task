# simple-task

タスクを登録し、そのまま `worker` エージェントへ渡す Rust 製 MCP サーバー。
planner が MCP ツール `create_task` でタスクを積むと、`worker` が `get_task` で content を取得して実行し、
その結果が同期レスポンスとして返る。

- タスクは **`slug` + `content` のみ**（状態管理はしない）
- 永続化: **`~/.config/simple-task/<slug>.md`（1 タスク = 1 ファイル）**
- バイナリ: `~/.local/bin/simple-task`
- MCP は **stdio**

worker にはファイルパスを一切渡さない。worker は MCP の `get_task(slug)` で content を取得する。

## ビルドと配置

```bash
cargo build --release
install -m 755 target/release/simple-task ~/.local/bin/simple-task
```

`~/.local/bin` が `PATH` にあることを確認する。

## CLI

| コマンド | 内容 |
|---|---|
| `simple-task mcp` | MCP サーバーを stdio で起動（既定。OpenClaw から呼ばれる） |
| `simple-task list` | 登録済みタスクの slug 一覧（人間用） |
| `simple-task show <slug>` | 1 件の content を表示（人間用） |
| `simple-task path` | 保存ディレクトリの実パスを表示（デバッグ用） |

サブコマンド無しで起動した場合は `mcp` として動く。

## MCP ツール

| ツール | 入力 | 出力 | 利用者 |
|---|---|---|---|
| `create_task` | `slug: String`, `content: String` | `{ slug, result }` | planner |
| `get_task` | `slug: String` | `{ slug, content }` | worker |
| `list_tasks` | なし | `{ slugs: [String] }` | worker |

`create_task` は**同期**。worker の最終出力を `result` として返す。タスクファイルは書き換えない。
**どのツールの出力にも絶対パスを含めない。**

## slug の検証

slug はそのままファイル名になるため、パストラバーサルを防ぐ。

- 許可: `^[a-z0-9][a-z0-9._-]{0,63}$`
- 拒否: `/`・`..`・先頭 `.`・空白・空文字・64 文字超・非 ASCII
- 違反時は `invalid slug: "..."（許可: ^[a-z0-9][a-z0-9._-]{0,63}$）` を返す
- 重複 slug はエラー（上書きしない）

## OpenClaw への登録

```bash
openclaw mcp add simple-task --command ~/.local/bin/simple-task --args mcp
openclaw mcp status
openclaw mcp probe simple-task
```

per-agent の権限配分（planner に `create_task` のみ / worker に `get_task`・`list_tasks` のみ）は
`openclaw mcp tools` で調整する。詳細はプラン §10 を参照。

## 環境変数

| 変数 | 既定 | 内容 |
|---|---|---|
| `SIMPLE_TASK_OPENCLAW_BIN` | `openclaw` | worker 起動に使う openclaw バイナリ |
| `SIMPLE_TASK_TIMEOUT_SECS` | `3600` | worker 完了待ちのタイムアウト（秒） |
| `SIMPLE_TASK_DIR` | `~/.config/simple-task` | タスク保存ディレクトリ（テスト用の上書き） |
| `XDG_CONFIG_HOME` | `~/.config` | 保存先の基点 |

## ハンドオフの流れ

1. `create_task` が `~/.config/simple-task/<slug>.md` に content を書く
2. 子プロセスで worker セッションへ通知（**slug のみ**渡す。パスは渡さない）:

   ```
   openclaw agent --agent worker \
     --session-key "agent:worker:simple-task:<slug>" \
     --message "<slug> のタスクを get_task で取得し、その content に従って実行せよ。" \
     --json
   ```

3. worker は `get_task(slug)` で content を取得して実行
4. 子プロセスの `--json` 封筒から `result`（最終発話）を取り出して返す

タイムアウト超過時は子プロセスを kill してエラーを返す（タスクファイルは触らない）。

## 開発

```bash
cargo build
cargo clippy --all-targets -- -D warnings
cargo test
```

テストは store の slug 検証（traversal 拒否・長さ境界） / content roundtrip / 重複 / 一覧ソート、
MCP ツール定義（3 ツール・パス非開示）、handoff の封筒抽出を含む。

## ドキュメント

- `thoughts/simple-task-plan.md` — 実装プラン（正本・v0.4）
- `thoughts/v04-impl-task.md` — v0.4 実装タスク指示
- `thoughts/v04-impl-report.md` — v0.4 実装レポート
