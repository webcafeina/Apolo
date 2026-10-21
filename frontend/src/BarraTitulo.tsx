// La barra de título de Windows (ADR 0027): la de una aplicación de WinUI con
// la barra alta, de 48 px, que Apolo dibuja porque la ventana no lleva marco.
// A la izquierda, el botón ☰ del panel de navegación, el icono y el nombre; a
// la derecha, los tres botones de la ventana, de 46 × 48 con los dibujos de
// Segoe Fluent Icons. El resto arrastra la ventana, y el doble clic la
// maximiza (lo hace Tauri con `data-tauri-drag-region`).
//
// El menú de Snap al dejar el puntero sobre «Maximizar» no lo da el HTML: lo
// da una ventana nativa transparente encima del botón (src-tauri, solo en
// Windows), que avisa con `data-encima` para que el botón se ilumine igual.

import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Icono, IconoApp } from "./componentes";
import * as puente from "./puente";

export function BarraTitulo({ panelAbierto, alMenu }: { panelAbierto: boolean; alMenu: () => void }) {
  const { t } = useTranslation();
  const [estado, setEstado] = useState<puente.EstadoVentana>({ maximizada: false, activa: true });
  const [snap, setSnap] = useState<"encima" | "pulsado" | "fuera">("fuera");

  useEffect(() => {
    const quitar: (() => void)[] = [];
    let vivo = true;
    const guardar = (q: () => void) => (vivo ? quitar.push(q) : q());
    puente.alCambiarVentana(setEstado).then(guardar);
    puente.alBotonMaximizar(setSnap).then(guardar);
    return () => {
      vivo = false;
      quitar.forEach((q) => q());
    };
  }, []);

  return (
    <header className="barra-titulo" data-tauri-drag-region data-activa={estado.activa ? "si" : "no"}>
      <button
        className="boton-menu"
        aria-label={t(panelAbierto ? "ventana.cerrarNavegacion" : "ventana.abrirNavegacion")}
        title={t(panelAbierto ? "ventana.cerrarNavegacion" : "ventana.abrirNavegacion")}
        aria-expanded={panelAbierto}
        onClick={alMenu}
        data-prueba="boton-menu"
      >
        <Icono nombre="menu" lado={16} />
      </button>
      <span className="icono-ventana" data-tauri-drag-region>
        <IconoApp lado={16} />
      </span>
      <span className="titulo-ventana" data-tauri-drag-region>
        {t("app.nombre")}
      </span>
      {/* Fuera del orden del tabulador, como los de Windows: con el teclado,
          Alt+F4 y Win+flechas. */}
      <div className="botones-ventana">
        <button tabIndex={-1} aria-label={t("ventana.minimizar")} title={t("ventana.minimizar")} onClick={() => void puente.botonVentana("minimizar")}>
          <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
            <path d="M0 5.5h10" />
          </svg>
        </button>
        <button
          tabIndex={-1}
          aria-label={t(estado.maximizada ? "ventana.restaurar" : "ventana.maximizar")}
          title={t(estado.maximizada ? "ventana.restaurar" : "ventana.maximizar")}
          // En la ventana, el clic lo recoge la de Snap, que maximiza ella.
          onClick={() => void puente.botonVentana("maximizar")}
          data-encima={snap !== "fuera" ? "si" : undefined}
          data-pulsado={snap === "pulsado" ? "si" : undefined}
          data-prueba="boton-maximizar"
        >
          <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
            {estado.maximizada ? (
              <path d="M2.5 2.5V1.5a1 1 0 0 1 1-1h5a1 1 0 0 1 1 1v5a1 1 0 0 1-1 1h-1M1.5 2.5h5a1 1 0 0 1 1 1v5a1 1 0 0 1-1 1h-5a1 1 0 0 1-1-1v-5a1 1 0 0 1 1-1z" />
            ) : (
              <rect x="0.5" y="0.5" width="9" height="9" rx="1" />
            )}
          </svg>
        </button>
        <button className="cerrar" tabIndex={-1} aria-label={t("ventana.cerrar")} title={t("ventana.cerrar")} onClick={() => void puente.botonVentana("cerrar")}>
          <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
            <path d="M0.5 0.5l9 9M9.5 0.5l-9 9" />
          </svg>
        </button>
      </div>
    </header>
  );
}
