// Los textos de la web en español (ADR 0033). en.ts tiene que llevar las
// mismas claves: TypeScript lo exige con el tipo `Textos`.
export const es = {
  idioma: { etiqueta: "Idioma", es: "Español", en: "English" },
  sitio: {
    titulo: "Apolo · Imágenes más ligeras, igual de buenas",
    descripcion:
      "Optimizador de imágenes gratuito para macOS, Windows y Linux: WebP, AVIF, JPEG XL, JPEG y PNG en tu ordenador, viendo lo que pierdes antes de guardar.",
  },
  heroe: { titulo: "Imágenes más ligeras, igual de buenas." },
};

export type Textos = typeof es;
