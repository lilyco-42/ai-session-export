# AI Session Export

Browse and export local AI-agent conversations through a Web UI, CLI, or Lilyco MCP. The original stores are read-only. Markdown, JSONL, HTML and plain-text exports are written to `~/.local/share/ai-session-export/out`.

| Source | Store |
| --- | --- |
| Codex CLI | `$CODEX_HOME/sessions` and `$CODEX_HOME/archived_sessions`, defaulting to `~/.codex` |
| Claude Code | `~/.claude/projects` |
| Antigravity CLI (AGY) | `~/.gemini/antigravity-cli`, including indexed user and assistant text |
| Gemini CLI | `~/.gemini/tmp/*/chats` |
| Pi | `~/.pi/agent/sessions` |
| OpenCode | `~/.local/share/opencode/opencode.db` |
| CodeArts Doer / CodeBuddy | `~/.codeartsdoer` and `~/.codebuddy` |
| DeepSeek Harness | Local Billion Context SQLite index |
| Other agents, including Cursor and Copilot | JSONL adapter contract below |

Built-in adapters cover the documented local layouts. Cloud-only history and provider-specific binary attachments are outside the text export. The server snapshots the source stores at startup.

## Run locally

Install Rust, Node.js and pnpm. From the repository root:

```sh
pnpm --dir web install --frozen-lockfile
pnpm --dir web run build
cargo run --locked --manifest-path server/Cargo.toml
```

Open `http://127.0.0.1:8788`. Set `AI_SESSION_WEB_DIR` if the compiled frontend lives elsewhere.

```sh
cargo run --locked --manifest-path server/Cargo.toml -- ai-session-list --source codex --json
cargo run --locked --manifest-path server/Cargo.toml -- ai-session-export --ids SESSION_ID --format markdown --json
cargo run --locked --manifest-path server/Cargo.toml -- --mcp
```

## Agent interchange v1

Write UTF-8 JSONL to `~/.local/share/ai-session-export/imports/<agent>.jsonl`, or set `AI_SESSION_IMPORT_DIR` to a different local directory. Source IDs use lowercase ASCII, digits, dashes or underscores. The built-in IDs and `all` are reserved. Each line contains one session. Write to a temporary file and rename it on completion.

```json
{"id":"session-123","title":"Example","updatedAt":"2026-11-24T12:00:00Z","messages":[{"role":"user","kind":"text","text":"Hello"},{"role":"assistant","kind":"text","text":"Hi"}]}
```

Supply a stable ID within the source, an RFC 3339 timestamp, and messages in conversational order. The `kind` field defaults to `text`. Published session IDs are prefixed with the source, for example `my-agent:session-123`. A JSONL export can also be used as input. This interface works with any language and does not execute adapter code.
