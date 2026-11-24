import path from "node:path";
import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

const apiTarget = process.env.VITE_API_TARGET || "http://127.0.0.1:8788";
const apiProxy = {
  "/api": apiTarget,
  "/exports": apiTarget,
};

export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: { alias: { "@": path.resolve(__dirname, "./src") } },
  server: { proxy: apiProxy },
  preview: { proxy: apiProxy },
  build: { outDir: "dist", emptyOutDir: true, chunkSizeWarningLimit: 2048 },
});
