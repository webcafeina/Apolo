// Todos los textos visibles salen de aquí (ADR 0007). Un componente nunca
// lleva una frase escrita: pide una clave.
import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import es from "./es.json";

void i18n.use(initReactI18next).init({
  resources: { es: { translation: es } },
  lng: "es",
  fallbackLng: "es",
  interpolation: { escapeValue: false },
});

export default i18n;
