import { en } from "./en";
import { es } from "./es";
import type { Idioma } from "./rutas";

export const textos = (idioma: Idioma) => (idioma === "es" ? es : en);
