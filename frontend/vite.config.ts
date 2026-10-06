import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Tauri espera el servidor de desarrollo en un puerto fijo (tauri.conf.json → devUrl).
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: { port: 5173, strictPort: true },
  build: { target: "es2022" },
});
