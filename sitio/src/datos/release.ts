// Los ficheros de la última versión, leídos al compilar (ADR 0033): web.yml
// escribe release.json con `gh release view` y sumas.txt con SHA256SUMS.txt de
// la Release. El navegador no pregunta nada a GitHub. Si falta un fichero que
// la web enlaza, la compilación falla: mejor que un enlace roto.

export type Sistema = "macos" | "windows" | "linux";
export type Arquitectura = "x64" | "arm64";
export type Tipo = "app" | "cli" | "mcpb";

export interface Fichero {
  clave: string;
  sistema: Sistema;
  arquitectura: Arquitectura | "universal";
  tipo: Tipo;
  /** «dmg», «exe», «deb», «rpm», «AppImage», «tar.gz», «zip», «mcpb». */
  formato: string;
  nombre: string;
  url: string;
  bytes: number;
  sha256: string;
}

export interface Release {
  version: string;
  fecha: string;
  ficheros: Record<string, Fichero>;
}

interface Asset {
  name: string;
  size: number;
  url: string;
}

// Cada fichero que enlaza la web, por cómo acaba su nombre (los de la app los
// nombra Tauri; los de la CLI y el .mcpb, publicar.yml).
const ESPERADOS: [string, Sistema, Arquitectura | "universal", Tipo, string, string][] = [
  ["dmg", "macos", "universal", "app", "dmg", "_universal.dmg"],
  ["exe-x64", "windows", "x64", "app", "exe", "_x64-setup.exe"],
  ["exe-arm64", "windows", "arm64", "app", "exe", "_arm64-setup.exe"],
  ["deb-x64", "linux", "x64", "app", "deb", "_amd64.deb"],
  ["deb-arm64", "linux", "arm64", "app", "deb", "_arm64.deb"],
  ["rpm-x64", "linux", "x64", "app", "rpm", ".x86_64.rpm"],
  ["rpm-arm64", "linux", "arm64", "app", "rpm", ".aarch64.rpm"],
  ["appimage-x64", "linux", "x64", "app", "AppImage", "_amd64.AppImage"],
  ["appimage-arm64", "linux", "arm64", "app", "AppImage", "_aarch64.AppImage"],
  ["cli-macos", "macos", "universal", "cli", "tar.gz", "-macos-universal.tar.gz"],
  ["cli-windows-x64", "windows", "x64", "cli", "zip", "-windows-x64.zip"],
  ["cli-windows-arm64", "windows", "arm64", "cli", "zip", "-windows-arm64.zip"],
  ["cli-linux-x64", "linux", "x64", "cli", "tar.gz", "-linux-amd64.tar.gz"],
  ["cli-linux-arm64", "linux", "arm64", "cli", "tar.gz", "-linux-arm64.tar.gz"],
  ["mcpb-macos", "macos", "universal", "mcpb", "mcpb", "-macos.mcpb"],
  ["mcpb-windows-x64", "windows", "x64", "mcpb", "mcpb", "-windows-x64.mcpb"],
  ["mcpb-windows-arm64", "windows", "arm64", "mcpb", "mcpb", "-windows-arm64.mcpb"],
  ["mcpb-linux-x64", "linux", "x64", "mcpb", "mcpb", "-linux-amd64.mcpb"],
  ["mcpb-linux-arm64", "linux", "arm64", "mcpb", "mcpb", "-linux-arm64.mcpb"],
];

/** Las sumas de SHA256SUMS.txt: «<suma>  <nombre>» por línea. */
export function leerSumas(texto: string): Map<string, string> {
  const m = new Map<string, string>();
  for (const l of texto.split("\n")) {
    const [suma, nombre] = l.trim().split(/\s+\*?/);
    if (suma && nombre) m.set(nombre, suma);
  }
  return m;
}

export function leerRelease(json: { tagName: string; publishedAt: string; assets: Asset[] }, sumas: string): Release {
  const s = leerSumas(sumas);
  const ficheros: Record<string, Fichero> = {};
  const faltan: string[] = [];
  for (const [clave, sistema, arquitectura, tipo, formato, final] of ESPERADOS) {
    const a = json.assets.find((x) => x.name.endsWith(final));
    const sha256 = a && s.get(a.name);
    if (!a || !sha256) {
      faltan.push(`${clave} (…${final})`);
      continue;
    }
    ficheros[clave] = { clave, sistema, arquitectura, tipo, formato, nombre: a.name, url: a.url, bytes: a.size, sha256 };
  }
  if (faltan.length) throw new Error(`La Release ${json.tagName} no tiene: ${faltan.join(", ")}`);
  return { version: json.tagName.replace(/^v/, ""), fecha: json.publishedAt, ficheros };
}

/** Un tamaño en megas decimales, como los cuentan los sistemas: «43,7 MB». */
export function megas(bytes: number, idioma: "es" | "en"): string {
  const n = new Intl.NumberFormat(idioma === "es" ? "es-ES" : "en-US", {
    minimumFractionDigits: 1,
    maximumFractionDigits: 1,
  }).format(bytes / 1e6);
  return `${n} MB`;
}
