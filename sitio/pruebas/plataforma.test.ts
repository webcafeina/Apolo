import { describe, expect, it } from "vitest";
import { detectar } from "../src/scripts/plataforma";

// Cadenas reales de navegadores (2026), recortadas a lo que cuenta.
const CHROME_WIN = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36";
const FIREFOX_WIN = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:144.0) Gecko/20100101 Firefox/144.0";
const SAFARI_MAC = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/27.0 Safari/605.1.15";
const FIREFOX_LINUX_ARM = "Mozilla/5.0 (X11; Linux aarch64; rv:144.0) Gecko/20100101 Firefox/144.0";
const CHROME_LINUX = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36";
const ANDROID = "Mozilla/5.0 (Linux; Android 16; Pixel 9) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Mobile Safari/537.36";
const IPHONE = "Mozilla/5.0 (iPhone; CPU iPhone OS 27_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/27.0 Mobile/15E148 Safari/604.1";
const CHROMEOS = "Mozilla/5.0 (X11; CrOS x86_64 16181.61.0) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36";

describe("qué ordenador visita la web", () => {
  it("Windows: x64 por suposición, y ARM cuando Chromium lo dice", () => {
    expect(detectar({ ua: CHROME_WIN })).toEqual({ plataforma: "windows", arquitectura: "x64", segura: false });
    expect(detectar({ ua: FIREFOX_WIN })).toEqual({ plataforma: "windows", arquitectura: "x64", segura: false });
    expect(detectar({ ua: CHROME_WIN, plataforma: "Windows", arquitectura: "arm", bitness: "64" })).toEqual({
      plataforma: "windows",
      arquitectura: "arm64",
      segura: true,
    });
    expect(detectar({ ua: CHROME_WIN, plataforma: "Windows", arquitectura: "x86", bitness: "64" }).segura).toBe(true);
  });

  it("macOS, Intel o Apple Silicon (el .dmg vale para los dos)", () => {
    expect(detectar({ ua: SAFARI_MAC }).plataforma).toBe("macos");
    expect(detectar({ ua: SAFARI_MAC, toques: 0 }).plataforma).toBe("macos");
  });

  it("un iPad que se hace pasar por Mac es un móvil", () => {
    expect(detectar({ ua: SAFARI_MAC, toques: 5 }).plataforma).toBe("movil");
  });

  it("Linux: ARM por el UA de Firefox o por Chromium; si no, x64 por suposición", () => {
    expect(detectar({ ua: FIREFOX_LINUX_ARM })).toEqual({ plataforma: "linux", arquitectura: "arm64", segura: true });
    expect(detectar({ ua: CHROME_LINUX })).toEqual({ plataforma: "linux", arquitectura: "x64", segura: false });
    expect(detectar({ ua: CHROME_LINUX, plataforma: "Linux", arquitectura: "arm" }).arquitectura).toBe("arm64");
  });

  it("móviles y ChromeOS, aparte", () => {
    expect(detectar({ ua: ANDROID }).plataforma).toBe("movil");
    expect(detectar({ ua: IPHONE }).plataforma).toBe("movil");
    expect(detectar({ ua: CHROME_WIN, movil: true }).plataforma).toBe("movil");
    expect(detectar({ ua: CHROMEOS }).plataforma).toBe("chromeos");
  });

  it("lo que no se reconoce", () => {
    expect(detectar({ ua: "curl/8.9" }).plataforma).toBe("otro");
  });
});
