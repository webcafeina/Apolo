// security.txt (RFC 9116, ADR 0034): a quién avisar de un fallo de seguridad.
// Caduca al año de compilarse; la web se compila con cada versión.
import type { APIRoute } from "astro";

export const GET: APIRoute = ({ site }) => {
  const caduca = new Date(Date.now() + 365 * 864e5).toISOString().replace(/\.\d+Z$/, "Z");
  return new Response(
    `Contact: https://github.com/webcafeina/Apolo/security/advisories/new
Contact: mailto:info@webcafeina.com
Expires: ${caduca}
Preferred-Languages: es, en
Canonical: ${new URL("/.well-known/security.txt", site).href}
Policy: https://github.com/webcafeina/Apolo/blob/main/SECURITY.md
`,
    { headers: { "Content-Type": "text/plain; charset=utf-8" } },
  );
};
