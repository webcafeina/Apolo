import { defineConfig, devices } from "@playwright/test";

/**
 * La interfaz de verdad contra el núcleo de verdad: `apolo-dev` levanta el
 * mismo servicio que lleva la ventana (crates/servicio) y solo cambia el
 * transporte, HTTP en vez de Tauri (ADR 0013). Es lo que permite probar el
 * Estudio entero en una máquina sin entorno gráfico.
 *
 * Los presets y los ajustes van a una carpeta temporal, nunca a la del usuario.
 *
 * El idioma, el del sistema, que se fija en español con LC_ALL: la batería
 * comprueba los textos en español en cualquier máquina (en CI el LANG es
 * C.UTF-8, que daría inglés). La prueba en inglés lo cambia en Ajustes y lo
 * devuelve al acabar (ADR 0025).
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
      // Las capturas para la web en inglés (`CAPTURAS_IDIOMA=en`, `make capturas-en`) arrancan
      // apolo-dev en inglés; todo lo demás, en español.
      env: {
        LC_ALL: process.env.CAPTURAS_IDIOMA === "en" ? "en_US.UTF-8" : "es_ES.UTF-8",
        // La ruta de apolo-mcp que enseña la sección MCP en las capturas de la web.
        ...(process.env.APOLO_DEV_RUTA_MCP ? { APOLO_DEV_RUTA_MCP: process.env.APOLO_DEV_RUTA_MCP } : {}),
      },
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
