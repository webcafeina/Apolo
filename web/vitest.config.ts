import { defineConfig } from "vitest/config";

// Las de Playwright (pruebas/*.spec.ts) no son de Vitest.
export default defineConfig({ test: { include: ["pruebas/**/*.test.ts"] } });
