import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { detectar } from "../src/scripts/plataforma";

const CODIGO = readFileSync(join(import.meta.dirname, "../src/scripts/temprano.js"), "utf8");

// temprano.js con un navegador de mentira: lo que deja en <html>.
function temprano(ua: string, extra: { toques?: number; plataforma?: string; guardado?: Record<string, string> } = {}) {
  const atributos: Record<string, string> = {};
  const document = { documentElement: { setAttribute: (k: string, v: string) => (atributos[k] = v) } };
  const navigator = {
    userAgent: ua,
    maxTouchPoints: extra.toques ?? 0,
    userAgentData: extra.plataforma ? { platform: extra.plataforma } : undefined,
  };
  const localStorage = { getItem: (k: string) => extra.guardado?.[k] ?? null };
  new Function("document", "navigator", "localStorage", CODIGO)(document, navigator, localStorage);
  return atributos;
}

const UA = {
  chromeWin: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36",
  safariMac: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/27.0 Safari/605.1.15",
  firefoxLinuxArm: "Mozilla/5.0 (X11; Linux aarch64; rv:144.0) Gecko/20100101 Firefox/144.0",
  chromeLinux: "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36",
  android: "Mozilla/5.0 (Linux; Android 16; Pixel 9) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Mobile Safari/537.36",
  chromeos: "Mozilla/5.0 (X11; CrOS x86_64 16181.61.0) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36",
  curl: "curl/8.9",
};

describe("temprano.js dice lo mismo que detectar()", () => {
  it.each(Object.entries(UA))("%s", (_, ua) => {
    const a = temprano(ua);
    const d = detectar({ ua });
    expect(a["data-plataforma"]).toBe(d.plataforma);
    expect(a["data-arquitectura"]).toBe(d.arquitectura);
  });

  it("el iPad que se hace pasar por Mac", () => {
    expect(temprano(UA.safariMac, { toques: 5 })["data-plataforma"]).toBe("movil");
  });

  it("la vista sale del sistema, y lo guardado manda", () => {
    expect(temprano(UA.chromeWin)["data-vista"]).toBe("windows");
    expect(temprano(UA.chromeLinux)["data-vista"]).toBe("gnome");
    expect(temprano(UA.android)["data-vista"]).toBe("macos");
    const g = temprano(UA.chromeLinux, { guardado: { "apolo-vista": "kde", "apolo-distro": "arch" } });
    expect(g["data-vista"]).toBe("kde");
    expect(g["data-distro"]).toBe("arch");
  });
});
