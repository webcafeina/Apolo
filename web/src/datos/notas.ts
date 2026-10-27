// Las novedades (ADR 0033): las notas de cada versión (.github/notas/vX.Y.Z.md),
// recortadas a lo que cambia, en cada idioma. Las notas se escriben antes de
// etiquetar: las de versiones mayores que la publicada no salen. Las de antes
// de la 0.12.0 solo están en español.
import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { Marked } from "marked";

// Las notas salen tal cual en la web (set:html), así que nada de lo que
// traigan se ejecuta (ADR 0034): el HTML crudo se escribe como texto, y un
// enlace que no sea http(s), mailto, un ancla o una ruta se queda en su texto.
const escapar = (s: string) => s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
export const enlaceSeguro = (href: string) => /^(https?:|mailto:|#|\/|\.)/i.test(href) || !/^[a-z][a-z0-9+.-]*:/i.test(href);
const marked = new Marked({
  renderer: {
    html: ({ text }) => escapar(text),
    link({ href, tokens }) {
      return enlaceSeguro(href) ? false : this.parser.parseInline(tokens);
    },
    image: ({ href, text }) => (enlaceSeguro(href) ? false : escapar(text)),
  },
});

export interface Nota {
  version: string;
  es: string;
  /** null si la nota no tiene parte en inglés. */
  en: string | null;
}

const numero = (v: string) => v.split(".").map(Number);
export function compararVersiones(a: string, b: string): number {
  const [x, y] = [numero(a), numero(b)];
  for (let i = 0; i < 3; i++) if (x[i] !== y[i]) return (x[i] ?? 0) - (y[i] ?? 0);
  return 0;
}

/** La parte de cada idioma, sin las descargas ni lo que es de la Release. */
export function recortar(md: string): { es: string; en: string | null } {
  const [es, en] = md.split(/\n-{3,}\s*\n+## In English\s*\n/);
  const limpiaEs = es
    .replace(/^\*English below\.\*\s*$/m, "")
    .split(/\n## Descargas\b/)[0]
    .replace(/^## Qué cambia\s*$/m, "")
    .trim();
  const limpiaEn = en === undefined ? null : en.split(/\n\*\*Downloads\*\*|\n## Downloads\b/)[0].trim();
  return { es: limpiaEs, en: limpiaEn };
}

export function leerNotas(carpeta: string, hasta: string): Nota[] {
  return readdirSync(carpeta)
    .filter((f) => /^v\d+\.\d+\.\d+\.md$/.test(f))
    .map((f) => {
      const version = f.slice(1, -3);
      const { es, en } = recortar(readFileSync(join(carpeta, f), "utf8"));
      return { version, es: marked.parse(es, { async: false }), en: en === null ? null : marked.parse(en, { async: false }) };
    })
    .filter((n) => compararVersiones(n.version, hasta) <= 0)
    .sort((a, b) => compararVersiones(b.version, a.version));
}
