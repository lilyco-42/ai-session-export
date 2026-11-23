# UI verification evidence (2026-11-23)

## Build
- TypeScript: `../node_modules/.bin/tsc --noEmit` passed with no output.
- Vite production build: `../node_modules/.bin/vite build` passed; 1646 modules transformed; dist JS 409.90 kB / gzip 126.35 kB; CSS 23.30 kB / gzip 5.48 kB; built in 1.63s.

## Real Chromium CDP verification
- Fixture server: `PORT=4199 node verify-server.mjs` from `web/`.
- CDP endpoint: Chrome 156, existing `http://[::1]:9333`, page target navigated to `http://127.0.0.1:4199/`.
- The export tab was clicked via DOM, table rendered 6 fixture sessions; first row: `AGY / Antigravity | VRC面部捕捉代码编写 | 2026-11-20T13:49:03+08:00`.
- Emaulated viewport metrics from CDP (scrollWidth <= innerWidth required):
  - 320: scroll=277, inner=291, rows=6, export controls visible.
  - 390: scroll=341, inner=355, rows=6, export controls visible.
  - 768: scroll=698, inner=698, rows=6, export controls visible.
  - 1440: scroll=1309, inner=1309, rows=6, export controls visible.
- Screenshots: `evidence/cdp-export-320.png`, `evidence/cdp-export-390.png`, `evidence/cdp-export-768.png`, `evidence/cdp-export-1440.png`.

## Notes
- Earlier plain `chromium --headless --screenshot` attempts failed with `net::ERR_CONNECTION_REFUSED` because the preview server did not persist between tool calls; CDP verification uses the stable existing Chrome endpoint and the fixture server started in the same shell.
- The fixture API is only for UI verification; no original AI conversation store was modified.
