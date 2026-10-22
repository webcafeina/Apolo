// Los botones de la ventana en Linux (ADR 0029), como los de una aplicación de
// libadwaita: redondos y grises, con los iconos simbólicos de Yaru, en el lado y
// el orden que diga el sistema (`gtk-decoration-layout`, por defecto en Ubuntu
// «:minimize,maximize,close», todos a la derecha). Solo salen con la barra
// propia, en GNOME; con la barra del sistema los pone el sistema. Se usan los
// mismos `puente.botonVentana` y `alCambiarVentana` que la barra de Windows.

import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Icono } from "./componentes";
import * as puente from "./puente";

type Boton = "minimize" | "maximize" | "close";
const CONOCIDOS: Boton[] = ["minimize", "maximize", "close"];

/** Los botones de cada lado, en su orden. Lo que no conoce (`appmenu`, `icon`), fuera. */
export function ladosBotones(disposicion: string): { izquierda: Boton[]; derecha: Boton[] } {
  const [izq = "", der = ""] = disposicion.split(":");
  const lista = (s: string) => s.split(",").filter((b): b is Boton => CONOCIDOS.includes(b as Boton));
  return { izquierda: lista(izq), derecha: lista(der) };
}

export function BotonesVentana({ botones }: { botones: Boton[] }) {
  const { t } = useTranslation();
  const [maximizada, setMaximizada] = useState(false);

  useEffect(() => {
    let quitar = () => {};
    let vivo = true;
    puente.alCambiarVentana((e) => setMaximizada(e.maximizada)).then((q) => (vivo ? (quitar = q) : q()));
    return () => {
      vivo = false;
      quitar();
    };
  }, []);

  const datos: Record<Boton, { que: "minimizar" | "maximizar" | "cerrar"; icono: string; texto: string }> = {
    minimize: { que: "minimizar", icono: "minimizar", texto: t("ventana.minimizar") },
    maximize: {
      que: "maximizar",
      icono: maximizada ? "restaurar" : "maximizar",
      texto: t(maximizada ? "ventana.restaurar" : "ventana.maximizar"),
    },
    close: { que: "cerrar", icono: "cerrar", texto: t("ventana.cerrar") },
  };

  return (
    <div className="botones-gnome">
      {botones.map((b) => (
        // Fuera del orden del tabulador, como los de GNOME.
        <button
          key={b}
          tabIndex={-1}
          aria-label={datos[b].texto}
          title={datos[b].texto}
          onClick={() => void puente.botonVentana(datos[b].que)}
          data-prueba={`boton-${datos[b].que}`}
        >
          <Icono nombre={datos[b].icono} lado={16} />
        </button>
      ))}
    </div>
  );
}
