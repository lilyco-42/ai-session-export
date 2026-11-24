import { createServer } from "node:http";
import { readFile } from "node:fs/promises";
import path from "node:path";

const root = path.resolve("dist");
const port = Number(process.env.PORT || 4199);
const types = {
  ".html": "text/html",
  ".js": "text/javascript",
  ".css": "text/css",
  ".json": "application/json",
  ".md": "text/markdown; charset=utf-8",
  ".jsonl": "application/x-ndjson; charset=utf-8",
  ".txt": "text/plain; charset=utf-8",
  ".woff2": "font/woff2",
  ".png": "image/png",
};

function json(res, value) {
  res.writeHead(200, { "content-type": "application/json; charset=utf-8" });
  res.end(JSON.stringify(value));
}

async function sendFile(res, urlPath) {
  const safe = path.normalize(urlPath).replace(/^(\.\.[/\\])+/, "");
  const file = path.join(root, safe === "/" ? "index.html" : safe);
  if (!file.startsWith(root)) throw new Error("path escape");
  const data = await readFile(file);
  res.writeHead(200, { "content-type": types[path.extname(file)] || "application/octet-stream" });
  res.end(data);
}

const server = createServer(async (req, res) => {
  try {
    const url = new URL(req.url || "/", "http://127.0.0.1");
    if (url.pathname === "/api/sources") {
      return json(res, JSON.parse(await readFile("fixtures/sources.json", "utf8")));
    }
    if (url.pathname === "/api/sessions") {
      const source = url.searchParams.get("source");
      const all = JSON.parse(await readFile("fixtures/sessions.json", "utf8"));
      const rows = source && source !== "all" ? all.filter((item) => item.source === source) : all;
      return json(res, rows);
    }
    if (url.pathname === "/api/export" && req.method === "POST") {
      return json(res, { downloadUrl: "/exports/fixture-export.md" });
    }
    if (url.pathname.startsWith("/exports/")) {
      res.writeHead(200, {
        "content-type": "text/markdown; charset=utf-8",
        "content-disposition": "attachment",
      });
      res.end("# Fixture Export\n\nMock export payload for verification.\n");
      return;
    }
    await sendFile(res, url.pathname);
  } catch {
    res.writeHead(404, { "content-type": "application/json; charset=utf-8" });
    res.end(
      JSON.stringify({
        error_code: "E_NOT_FOUND",
        reason_zh: "资源不存在",
        reason_en: "Resource not found",
        repair_zh: "请检查路径。",
      }),
    );
  }
});

server.listen(port, "127.0.0.1", () => console.log("verify server " + port));
