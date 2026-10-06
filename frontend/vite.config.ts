/// <reference types="vitest/config" />
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Tauri espera el servidor de desarrollo en un puerto fijo (tauri.conf.json → devUrl).
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  // En desarrollo sin ventana (make dev-web), /api y /pixeles van a apolo-dev.
  server: {
    port: 5173,
    strictPort: true,
    proxy: {
      "/api": `http://127.0.0.1:${process.env.APOLO_DEV_PUERTO ?? 34500}`,
      "/pixeles": `http://127.0.0.1:${process.env.APOLO_DEV_PUERTO ?? 34500}`,
    },
  },
  build: { target: "es2022" },
  // Las de Playwright (e2e/) no son de Vitest.
  test: { include: ["src/**/*.test.ts"] },
});
