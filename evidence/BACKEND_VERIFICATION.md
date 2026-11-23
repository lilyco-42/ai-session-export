# AI Session Export backend verification

- Time: 2026-11-23 16:55 +08:00
- Server: `ai-session-export-server` on `http://127.0.0.1:8788` (built offline with Rust 1.100.0).
- Frontend dev proxy: Vite on `http://127.0.0.1:5188` proxying `/api` to backend.
- `cargo fmt --all --check`, `cargo clippy --offline --all-targets -- -D warnings`, `cargo test --offline`: PASS (4 unit tests).
- `web`: TypeScript build and Vite production build PASS (1646 modules).
- Live `GET /api/sources`: HTTP 200, 5 sources: codearts=12, opencode=11, agy=26, codex=5, dsh=91; JSON uses `sessionCount`.
- Live `GET /api/sessions?source=agy`: HTTP 200, 26 sessions; first `Improve Lyco Lilyco Framework Web2CLI`; JSON uses `updatedAt`.
- Live `GET /api/sessions?source=dsh`: HTTP 200, 91 sessions; first `# AGY Command Export, LilyCo AI Conversation Export Tool,…`.
- Live `POST /api/export` with AGY VRChat session `de7af58e-7a08-4162-802d-c9aada369af8` and `format:"markdown"`: HTTP 200; response `{"downloadUrl":"/exports/de7af58e-7a08-4162-802d-c9aada369af8.md"}`; download HTTP 200, 37112 bytes; begins with `# VRC面部捕捉代码编写`.
- Vite proxy verification: `GET http://127.0.0.1:5188/api/sources` HTTP 200 with the same five source counts; `GET /api/sessions?source=agy` HTTP 200 with 26 sessions.
- All stores opened read-only by backend. A full SHA-256 snapshot comparison showed active-session artifacts (DSH `index.db*`, AGY current CLI log) changed due concurrently running desktop CLI processes; no export code writes to source stores. Source DB opens use `SQLITE_OPEN_READ_ONLY`, other adapters use read APIs only.
- Live Chrome CDP verification against Vite + real backend: `evidence/cdp-live.mjs`, `evidence/cdp-live-1440.png`. Browser showed five live source cards with counts 12/11/26/5/91; export queue loaded 145 real sessions including the VRChat session; at 1440 viewport `scrollWidth=1140 <= innerWidth=1152`. Earlier fixture screenshots remain in `evidence/UI_VERIFICATION.md` and `cdp-export-*.png`.
- Known caveats: CodeArts is currently implemented as CodeBuddy-compatible JSONL because `/home/lilyco/.codeartsdoer/cli-data/opencode.db` has no `session_v2` table; m00058 clarifications about exact format/runtime/tool names remain unanswered, so implementation supports all five tools and four formats (markdown/jsonl/raw/html) under those assumptions.