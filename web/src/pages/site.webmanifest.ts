// El manifiesto (ADR 0033): el nombre y los iconos, para quien la añade a la
// pantalla de inicio.
import type { APIRoute } from "astro";

export const GET: APIRoute = () =>
  new Response(
    JSON.stringify({
      name: "Apolo",
      short_name: "Apolo",
      description: "Imágenes más ligeras, igual de buenas.",
      start_url: "/",
      display: "browser",
      background_color: "#ffffff",
      theme_color: "#ffffff",
      icons: [
        { src: "/icono-192.png", sizes: "192x192", type: "image/png" },
        { src: "/icono-512.png", sizes: "512x512", type: "image/png" },
        { src: "/icono-mascara-512.png", sizes: "512x512", type: "image/png", purpose: "maskable" },
      ],
    }),
    { headers: { "Content-Type": "application/manifest+json" } },
  );
