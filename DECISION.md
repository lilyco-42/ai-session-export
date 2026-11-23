# AI Session Export - research decision (2026-11-23)

## Requirement (one line)
Build a local read-only AI conversation exporter on Lilyco, with a lyco_dev-compliant Web UI, listing and exporting sessions from codearts/opencode/agy/codex/dsh.

## Open questions (still unanswered from the previous round)
1. Export format priority: Markdown / JSONL / raw SQLite+SQL / HTML.
2. Runtime form: Lilyco CLI command, Lilyco Web GUI, or an existing lilyco project/crate.
3. Tool names: is codearts CodeArts Doer or CodeBuddy, is dsh DeepSeek Harness, and are there tools beyond these five?

## Evidence: actual local stores
- AGY: /home/lilyco/.gemini/antigravity-cli/{history.jsonl,cache/conversation_metadata.json,conversations/<uuid>.db}.
- OpenCode: /home/lilyco/.local/share/opencode/opencode.db tables session_v2, message, part, project, workspace.
- Codex: /home/lilyco/.codex/state_5.sqlite, /home/lilyco/.codex/sqlite/codex-dev.db, session_index.jsonl.
- CodeArts/CodeBuddy: /home/lilyco/.codeartsdoer/cli-data/opencode.db and /home/lilyco/.codebuddy/projects/tmp/*.jsonl.
- DSH: /home/lilyco/.local/share/billion-context/sessions/index.db tables sessions, messages, content, blocks, refs; web API requires auth.

## Research evidence (gh/ghs)
- jerrywu001/cc-sessions-viewer: MIT, 421 stars, supports cc/codex/grok/kimicode/pi/antigravity cli/opencode sessions viewer/global search/resume/export.
- SakuraByteCore/codexmate: Apache-2.0, 357 stars, local-first CLI + Web dashboard for Codex/Claude Code and more.
- vibheksoni/session-export: MIT, 24 stars, converts Codex/Claude Code/Pi/OpenCode/Devin/Factory sessions.
- HizTam/codex-history-viewer: MIT, 49 stars, VS Code extension for Codex/Claude Code/Antigravity import/export.
- gautamgpt1/agent-session-browser: MIT, 36 stars, browse/filter/export/resume local Codex/Claude Code/Gemini CLI/Pi/Antigravity sessions.
- No candidate clearly includes CodeArts/CodeBuddy and DSH/billion-context plus Lilyco/lyco_dev UI in one MIT/Apache-compatible local read-only package.

## Decision (interim, recorded before any code)
fork-extend / build a Lilyco-hosted adapter-and-UI layer after verifying cc-sessions-viewer adapters; do NOT wholesale adopt any single candidate because of the two local-only gaps (codearts ambiguous, DSH billion-context not covered) and the hard Lilyco/lyco_dev UI requirement. This is an interim decision: if the user confirms DSH/CodeArts are out of scope, adopting/forking cc-sessions-viewer or codexmate may cover 80 percent or more.

## UI implementation constraints for the next round
- Copy /home/lilyco/code/lilyco-template/web (Tailwind v4 + shadcn oklch + Geist + sonner + base-ui + lobehub icons + tanstack table/query/router) rather than create-vite or handwritten CSS.
- Backend is data/API only and never emits HTML/CSS; typed DTOs, deny_unknown_fields, structured bilingual JSON error_code.
- Read-only sources; do not mutate original stores; real browser screenshot plus 320/390/768/1440px overflow check.
- Rule of 4: type fields <=4, function params <=4, nesting <=4, directory subdirs <=4, depth <=4.
- Every capability needs tests; cargo fmt --check / clippy -D warnings / cargo test before claiming completion.
