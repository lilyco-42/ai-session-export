# Lilyco integration verification

- Added Lilyco 0.3.0 local path dependency (default features = CLI + MCP) to ai-session-export-server; crate is publish=false.
- Added lilyco command registry with three read-only commands: ai-session-sources, ai-session-list, ai-session-export. Routes call the same AppState service as Axum; no duplicate adapters.
- Binary entrypoint dispatches to Lilyco registry for --mcp/--schema/--help/-h/ai-session-* args; default no-arg run remains Axum server on 127.0.0.1:8788 for the Vite Web UI.
- cargo fmt --all --check, cargo clippy --offline --all-targets -- -D warnings, cargo test --offline: PASS (6 lib tests + 1 main test). Only warning is from Lilyco upstream lib.rs unused Arc import, not this crate.
- Lilyco CLI smoke: --help lists three commands; --schema shows all schemas with safety read_only.
- Lilyco CLI live data: ai-session-sources --json returned codearts=12, opencode=11, agy=26, codex=5, dsh=91. ai-session-list --source agy --json returned 26 sessions, including VRChat session de7af58e-7a08-4162-802d-c9aada369af8.
- Lilyco CLI export: ai-session-export --ids de7af58e-7a08-4162-802d-c9aada369af8 --format markdown --json returned {downloadUrl:/exports/de7af58e-7a08-4162-802d-c9aada369af8.md}; output file exists with 37112 bytes and begins "# VRC面部捕捉代码编写".
- MCP smoke: initialize + tools/list returned serverInfo Lilyco 0.3.0 and the three tools. A tools/call ai-session-sources with EOF wait returned isError=false and the same five source counts. The earlier timeout was caused by closing stdin before the worker thread wrote its response.
- Axum API regression: old server still serving 127.0.0.1:8788; GET /api/sources 200 with same counts; GET /exports/... 200 with 37112 bytes.
- Caveat: local cargo config rustc-wrapper points to broken sccache; verification used RUSTC=/usr/bin/rustc RUSTC_WRAPPER= CARGO_TARGET_DIR=/tmp/ai-export-target2. Final full rebuild passed with PATH=/usr/bin:/usr/local/bin:/bin HOME=/home/lilyco RUSTC=/usr/bin/rustc RUSTC_WRAPPER= CARGO_TARGET_DIR=/tmp/ai-export-target2 cargo build --offline; rebuilt binary CLI and MCP tool call smoke tests passed.