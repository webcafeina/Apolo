// La navegación de la web, por grupos: la misma en la cabecera, el menú del móvil y el pie.
import { textos } from ".";
import { type Idioma, ruta } from "./rutas";

export interface Enlace {
  href: string;
  texto: string;
}

export function navegacion(idioma: Idioma) {
  const t = textos(idioma);
  return {
    descargas: {
      titulo: t.nav.descargar,
      enlaces: [
        { href: ruta("descargarMac", idioma), texto: "Mac" },
        { href: ruta("descargarWindows", idioma), texto: "Windows" },
        { href: ruta("descargarLinux", idioma), texto: "Linux" },
      ] as Enlace[],
    },
    usar: {
      titulo: t.nav.usar,
      enlaces: [
        { href: ruta("agentes", idioma), texto: t.pie.agentes },
        { href: ruta("cli", idioma), texto: t.pie.cli },
      ] as Enlace[],
    },
    aprender: {
      titulo: t.nav.aprender,
      enlaces: [
        { href: ruta("guias", idioma), texto: t.pie.guias },
        { href: ruta("comparativas", idioma), texto: t.pie.comparativas },
        { href: ruta("preguntas", idioma), texto: t.pie.preguntas },
      ] as Enlace[],
    },
    novedades: { href: ruta("novedades", idioma), texto: t.pie.novedades } as Enlace,
  };
}
