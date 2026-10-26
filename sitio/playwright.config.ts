// Las pruebas de la web en el navegador (ADR 0033): contra una compilación con
// un ID de GTM de prueba (para el aviso de cookies), servida por astro preview.
// Nada sale a la red: las peticiones a Google se interceptan en las pruebas.
import { defineConfig, devices } from "@playwright/test";

const PUERTO = 4391;

export default defineConfig({
  testDir: "pruebas",
  testMatch: "*.spec.ts",
  fullyParallel: true,
  reporter: [["list"]],
  use: { baseURL: `http://127.0.0.1:${PUERTO}`, ...devices["Desktop Chrome"], viewport: { width: 1280, height: 900 } },
  webServer: {
    command: `pnpm exec astro build --outDir dist-e2e && pnpm exec astro preview --outDir dist-e2e --port ${PUERTO} --host 127.0.0.1 --ignore-lock`,
    env: { PUBLIC_GTM_ID: "GTM-PRUEBA" },
    url: `http://127.0.0.1:${PUERTO}/`,
    timeout: 180_000,
    reuseExistingServer: false,
  },
});
