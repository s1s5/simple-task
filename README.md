# simple-task

**タスク内容をファイルに保存・取得するだけ**の Rust 製 MCP サーバー。
worker の起動は OpenClaw の正規ルート（planner の `sessions_spawn`）で行う。MCP はプロセスを一切起動しない。

- タスクは **`slug` + `content` のみ**（状態管理はしない）
- 永続化: **`~/.config/simple-task/<slug>.md`（1 タスク = 1 ファイル）**
- バイナリ: `~/.local/bin/simple-task`
- MCP は **stdio**

worker にはファイルパスを一切渡さない。worker は MCP の `get_task(slug)` で content を取得する。

## 正しい流れ

```
planner
  ① MCP create_task(slug, content)      → ファイル作成のみ。即返る（実行も起動もしない）
  ② sessions_spawn(agentId="worker",
       task="<slug> のタスクを get_task で取得し、その content に従って実行せよ")
                                        ← OpenClaw の正規ルート
worker
  ③ MCP get_task(slug)                  → content を取得
  ④ 実行
```

- MCP は ①②③ の**内容の受け渡しだけ**。`openclaw agent` の子プロセス起動は行わない
- per-agent の権限分離は `tools.deny` では不可能（planner の deny が spawn 子へ継承されるため）。
  利用者の分担は **AGENTS.md の規律（ソフト制約）** で行う

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
| `create_task` | `slug: String`, `content: String` | `{ slug }` | planner |
| `get_task` | `slug: String` | `{ slug, content }` | worker |
| `list_tasks` | なし | `{ slugs: [String] }` | worker |

`create_task` は**ファイルを書いて即返る**（worker の完了を待たない。プロセスも起動しない）。
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

利用者の分担（planner = `create_task` / worker = `get_task`・`list_tasks`）は各エージェントの
`AGENTS.md` の規律で運用する。詳細はプラン §7 を参照。

## 環境変数

| 変数 | 既定 | 内容 |
|---|---|---|
| `SIMPLE_TASK_DIR` | `~/.config/simple-task` | タスク保存ディレクトリ（テスト用の上書き） |
| `XDG_CONFIG_HOME` | `~/.config` | 保存先の基点 |

## 開発

```bash
cargo build
cargo clippy --all-targets -- -D warnings
cargo test
```

テストは store の slug 検証（traversal 拒否・長さ境界） / content roundtrip / 重複 / 一覧ソート、
MCP ツール定義（3 ツール・パス非開示・`create_task` の outputSchema が `slug` のみ）を含む。

## ドキュメント

- `thoughts/simple-task-plan.md` — 実装プラン（正本・v0.6）
- `thoughts/v06-impl-task.md` — v0.6 実装タスク指示
- `thoughts/v06-impl-report.md` — v0.6 実装レポート
