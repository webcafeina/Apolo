// Todos los textos visibles salen de aquí (ADR 0007). Un componente nunca
// lleva una frase escrita: pide una clave. Hay dos catálogos con las mismas
// claves, es.json y en.json (ADR 0025): arranca en español, y main.tsx lo
// cambia al que diga Rust (el de los ajustes, o el del sistema) antes de pintar.
import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import en from "./en.json";
import es from "./es.json";

export type Idioma = "es" | "en";

void i18n.use(initReactI18next).init({
  resources: { es: { translation: es }, en: { translation: en } },
  lng: "es",
  fallbackLng: "es",
  interpolation: { escapeValue: false },
  // En desarrollo (y en las pruebas e2e), una clave que no está en ningún
  // catálogo se avisa en la consola: la batería en inglés lo vigila.
  saveMissing: import.meta.env.DEV,
  missingKeyHandler: (_idiomas, _ns, clave) => console.warn(`i18n: falta la clave «${clave}»`),
});

// El <html lang> sigue al idioma: lo usan los lectores de pantalla.
i18n.on("languageChanged", (l) => {
  document.documentElement.lang = l;
});

/** El idioma de la interfaz, para los números y las fechas (Intl). */
export const lengua = (): string => i18n.language;

export default i18n;
