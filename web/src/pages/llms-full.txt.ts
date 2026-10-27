// llms-full.txt (ADR 0034): la web en Markdown para las IAs (src/datos/llms.ts).
import type { APIRoute } from "astro";
import { completo } from "../datos/llms";

export const GET: APIRoute = ({ site }) =>
  new Response(completo(site!), { headers: { "Content-Type": "text/markdown; charset=utf-8" } });
