// Las imágenes del README (ADR 0033): la ventana de Apolo sobre su escritorio,
// tal como sale en «En tu sistema» de la web compilada (`pnpm build` antes).
// `node herramientas/readme.mjs`: macOS en claro y oscuro, y Windows, GNOME y
// KDE en claro, en los dos idiomas, a .github/readme/apolo-<vista>-<tema>[-en].webp.
import { spawn, spawnSync } from "node:child_process";
import { chromium } from "@playwright/test";
import { rmSync } from "node:fs";
import { join, resolve } from "node:path";

const SITIO = resolve(import.meta.dirname, "..");
const DESTINO = resolve(SITIO, "../.github/readme");
const APOLO = resolve(SITIO, "../target/release/apolo");
const PUERTO = 4393;

const servidor = spawn("pnpm", ["exec", "astro", "preview", "--port", String(PUERTO), "--host", "127.0.0.1", "--ignore-lock"], { cwd: SITIO, stdio: "ignore" });
await new Promise((r) => setTimeout(r, 3500));
const navegador = await chromium.launch();
const CASOS = [
  ["macos", "claro"],
  ["macos", "oscuro"],
  ["windows", "claro"],
  ["gnome", "claro"],
  ["kde", "claro"],
];
try {
  for (const idioma of ["es", "en"]) {
    for (const [vista, tema] of CASOS) {
      const ctx = await navegador.newContext({
        viewport: { width: 1280, height: 900 },
        deviceScaleFactor: 2,
        colorScheme: tema === "claro" ? "light" : "dark",
        reducedMotion: "reduce",
      });
      await ctx.addInitScript(
        ([v]) => {
          localStorage.setItem("apolo-vista", v);
          localStorage.setItem("apolo-consentimiento", JSON.stringify({ v: 1, analitica: false, fecha: Date.now() }));
        },
        [vista],
      );
      const p = await ctx.newPage();
      await p.goto(`http://127.0.0.1:${PUERTO}${idioma === "es" ? "/" : "/en/"}`);
      const escritorio = p.locator(`#${idioma === "es" ? "sistema" : "system"} .vista[data-vista="${vista}"] .escritorio`);
      await escritorio.scrollIntoViewIfNeeded();
      await p.waitForLoadState("networkidle");
      await p.waitForTimeout(500);
      const png = join(DESTINO, `tmp-${vista}-${tema}.png`);
      await escritorio.screenshot({ path: png });
      const nombre = `apolo-${vista}-${tema}${idioma === "es" ? "" : "-en"}.webp`;
      // A 1800 px de ancho (900 al doble de densidad), con el propio Apolo.
      spawnSync(APOLO, ["webp", "-q", "82", "-m", "6", "-resize", "1800", "0", "-quiet", png, "-o", join(DESTINO, nombre)]);
      rmSync(png);
      console.log(nombre);
      await ctx.close();
    }
  }
} finally {
  await navegador.close();
  servidor.kill();
}
