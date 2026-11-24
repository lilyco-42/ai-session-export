# AI Session Export

Browse and export local AI-agent conversations through a Web UI, CLI, or Lilyco MCP. The original stores are read-only. Markdown, JSONL, HTML and plain-text exports are written to `~/.local/share/ai-session-export/out`.

| Source | Store |
| --- | --- |
| ChatGPT (manual export) | User-provided conversations.json; see below |
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

## One-command installation

Linux / macOS (no sudo required):

```sh
curl -fsSL https://github.com/lilyco-42/ai-session-export/releases/latest/download/install.sh | sh
```

Windows PowerShell (no administrator rights required):

```powershell
$installer = Join-Path $env:TEMP 'ai-session-export-install.ps1'
Invoke-WebRequest -UseBasicParsing https://github.com/lilyco-42/ai-session-export/releases/latest/download/install.ps1 -OutFile $installer
& powershell.exe -NoProfile -ExecutionPolicy Bypass -File $installer
```

You can download and inspect the scripts before executing them. Both detect the native CPU architecture, check SHA256SUMS before extraction, and replace only the installed application, keeping existing exports and source stores untouched. Rerun the same installer to upgrade. Linux/macOS installs to `~/.local/share/ai-session-export/app` with `~/.local/bin/ai-session-export` (add that bin directory to PATH if necessary). Windows installs to `%LOCALAPPDATA%\ai-session-export\app` and adds the directory to the user PATH; open a new terminal. Run `ai-session-export` on Linux/macOS or `ai-session-export-server` on Windows, then open http://127.0.0.1:8788.

To install a specific version, download the installer from that release and set `AI_SESSION_VERSION` to its tag (or use `-Version v0.3.0` in PowerShell). The install scripts are distributed with releases starting at v0.3.0.

## Download a release

Download the archive matching your OS and CPU from [GitHub Releases](https://github.com/lilyco-42/ai-session-export/releases). Linux and macOS archives use `.tar.gz`; Windows uses `.zip`. Each archive includes the server, Web UI, README and license. Verify the archive against `SHA256SUMS` when provided.

Extract the entire archive and run `./ai-session-export-server` (Linux/macOS) or `.\ai-session-export-server.exe` (Windows PowerShell). Open `http://127.0.0.1:8788`. Keep the bundled `web/dist` beside the executable; the server locates it regardless of the working directory. CLI commands and `--mcp` work with the same executable. Linux builds target glibc-based systems (Ubuntu 22.04 or newer recommended); the archives are not signed/notarized.

### Maintainer release workflow

Pull requests, pushes to main and manual workflow runs build and test six native targets (Linux, macOS and Windows, each x64 and ARM64). To publish a new version, push a new `v*` tag after CI passes:

```sh
git tag v0.3.0
git push origin v0.3.0
```

The tag workflow publishes all six packages and SHA256SUMS only after every build and smoke test passes. Tags containing a hyphen are published as prereleases. An existing published release is never overwritten.

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


## Inspect conversations and task observations

Download your ChatGPT data using the official export feature, extract `conversations.json` and copy it to `~/.local/share/ai-session-export/imports/conversations.json` (or set `AI_SESSION_CHATGPT_EXPORT` to its absolute path). Restart the server to refresh the snapshot. The adapter reads visible user/assistant text on the active conversation branch only; it does not access browser cookies, live ChatGPT, cloud task schedules, or binary attachments.

Open the Web UI and search conversation text; click a title to read paginated messages, the latest task observation, and a native resume plan where supported.

CLI/MCP commands (replace SESSION_ID with the exact ID from ai-session-list):

```sh
ai-session-export ai-session-search --query "release" --source all --limit 50 --json
ai-session-export ai-session-get --id SESSION_ID --offset 0 --limit 100 --json
ai-session-export ai-task-list --source codex --limit 50 --json
ai-session-export ai-task-status --id SESSION_ID --json
ai-session-export ai-session-resume --id SESSION_ID --json
```

On Windows use ai-session-export-server instead. All five commands are read-only. Task observations describe the last recognized Codex turn event in its log (started, completed, aborted, or unknown). A started event does not establish that the agent is still running. Other sources report unknown rather than guessing. ChatGPT scheduled tasks are not included in this adapter. Native resume returns argv for UUID-backed Codex and Claude sessions; it never executes the command, and actual resumption requires the original agent and available session. Other sources can export a transcript for manual continuation.

HTTP endpoints (loopback): GET /api/session?id=...&offset=0&limit=100, GET /api/search?query=...&source=all&limit=50, GET /api/tasks?source=codex&limit=50, GET /api/task?id=..., GET /api/resume?id=... . Responses are local data; do not expose this server to untrusted networks.
