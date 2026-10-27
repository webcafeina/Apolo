// robots.txt (ADR 0033 y 0034): todo se puede indexar salvo los repositorios de
// paquetes, que solo gastarían ancho de banda (cada .deb y .rpm pesa ~25 MB).
// Los rastreadores de IA van nombrados: con permiso explícito, para que
// ChatGPT, Claude, Perplexity o Gemini puedan leer la web y citarla. Van en el
// mismo grupo que «*», o no verían las líneas Disallow.
import type { APIRoute } from "astro";

const IAS = [
  "GPTBot",
  "OAI-SearchBot",
  "ChatGPT-User",
  "ClaudeBot",
  "Claude-SearchBot",
  "Claude-User",
  "PerplexityBot",
  "Perplexity-User",
  "Google-Extended",
  "Applebot-Extended",
  "CCBot",
];

export const GET: APIRoute = ({ site }) =>
  new Response(
    `User-agent: *
${IAS.map((ia) => `User-agent: ${ia}`).join("\n")}
Allow: /
Disallow: /apt/
Disallow: /rpm/
Disallow: /aur/

Sitemap: ${new URL("/sitemap.xml", site).href}
`,
    { headers: { "Content-Type": "text/plain; charset=utf-8" } },
  );
