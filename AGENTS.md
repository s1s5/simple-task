# AGENTS.md — simple-task

タスク登録 + `worker` ハンドオフを担う MCP サーバー（Rust）。

## このリポジトリの目的

planner が `create_task` でタスクを登録すると、`worker` エージェントが
MCP の `get_task` で content を取得して実行し、その結果を同期レスポンスとして返す。
状態管理は持たず、タスクは `slug` と `content` だけをファイルに保持する。

## 構成

```
src/
├── main.rs     # clap ディスパッチ + tokio runtime
├── lib.rs      # モジュール公開 + Cli/dispatch
├── config.rs   # 保存ディレクトリ解決・env
├── error.rs    # Error 型（thiserror）+ MCP ErrorData 変換
├── store.rs    # ファイル読み書き + slug 検証
├── handoff.rs  # openclaw agent 起動・完了待ち・result 抽出
├── mcp.rs      # rmcp ServerHandler + 3 tools
└── cli.rs      # list / show / path
```

## 設計上の約束

- タスクは `slug` + `content` のみ。**状態（status/時刻/result/error）を保存しない**。
- 永続化は `~/.config/simple-task/<slug>.md`。**1 タスク = 1 ファイル**、本文 = content。
- `create_task` は同期。worker の出力はその場で返し、保存しない。
- `slug` は呼び出し側が強制指定。`^[a-z0-9][a-z0-9._-]{0,63}$` で検証し、違反はエラー。
  重複もエラー（上書きしない）。slug はそのままファイル名になるため**パストラバーサルを拒否する**。
- worker への通知は slug のみ。content は MCP 経由で渡す。**出力にファイルパスを含めない**。
- MCP ハンドルは保持しない。呼び出しごとに `store` 関数へ保存ディレクトリを渡す。
- MCP ツールの権限配分は planner = `create_task` / worker = `get_task`・`list_tasks` を想定。

## 開発ルール

- `unwrap` / `panic!` / `unreachable!` を使わない。エラーは `thiserror` + `?` で伝播する。
- 変更は最小差分に保つ。指示範囲外の改善は混ぜない。
- `.env` を読まない。credential・API キーを出力しない。
- `git commit` / `git push` はしない（差分を残すだけ）。
- ペルソナ・ワークスペース系ファイル（SOUL.md / USER.md / IDENTITY.md / memory/ / .openclaw/ 等）を作らない。

## 検証

```bash
cargo build
cargo clippy --all-targets -- -D warnings   # 警告ゼロ
cargo test                                  # store / mcp / handoff
cargo fmt --check
```

MCP ハンドシェイク確認:

```bash
printf '%s\n%s\n%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"p","version":"0"}}}' \
  '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
  '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' \
  | simple-task mcp
```

## 依存

rmcp 3.4.1（server / transport-io / macros）、tokio、clap、serde、serde_json、schemars 1、thiserror 2、
tracing、tracing-subscriber。**DB 依存（rusqlite）は持たない。**
toolchain は `rust-toolchain.toml`（1.97.1）。
