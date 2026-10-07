import { defineConfig, devices } from "@playwright/test";

/**
 * La interfaz de verdad contra el núcleo de verdad: `apolo-dev` levanta el
 * mismo servicio que lleva la ventana (crates/servicio) y solo cambia el
 * transporte, HTTP en vez de Tauri (ADR 0013). Es lo que permite probar el
 * Estudio entero en una máquina sin entorno gráfico.
 *
 * Los presets y los ajustes van a una carpeta temporal, nunca a la del usuario.
 */
const config = `${process.env.TMPDIR ?? "/tmp"}/apolo-e2e-config-${process.pid}`;

export default defineConfig({
  testDir: "./e2e",
  timeout: 30_000,
  fullyParallel: false,
  workers: 1,
  reporter: process.env.CI ? [["list"], ["html", { open: "never" }]] : [["list"]],
  use: {
    baseURL: "http://127.0.0.1:5191",
    viewport: { width: 1280, height: 800 },
    trace: process.env.CI ? "retain-on-failure" : "off",
    screenshot: process.env.CI ? "only-on-failure" : "off",
    permissions: ["clipboard-read", "clipboard-write"],
  },
  projects: [
    { name: "claro", use: { ...devices["Desktop Chrome"], viewport: { width: 1280, height: 800 }, colorScheme: "light" } },
    { name: "oscuro", use: { ...devices["Desktop Chrome"], viewport: { width: 1280, height: 800 }, colorScheme: "dark" } },
  ],
  webServer: [
    {
      command: `cargo run -q -p apolo-dev -- --direccion 127.0.0.1:34591 --config ${config}`,
      cwd: "..",
      url: "http://127.0.0.1:34591/salud",
      reuseExistingServer: false,
      timeout: 600_000,
      stdout: "ignore",
    },
    {
      command: "APOLO_DEV_PUERTO=34591 pnpm exec vite --port 5191 --strictPort --host 127.0.0.1",
      url: "http://127.0.0.1:5191",
      reuseExistingServer: false,
    },
  ],
});
