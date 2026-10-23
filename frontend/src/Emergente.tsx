// Una ventanita que sale de un botón (ADR 0032): el popover de macOS, el
// Flyout de WinUI, el GtkPopover de GNOME o el emergente de Breeze, según la
// hoja de cada sistema. Sin el atributo `popover` ni `<dialog>`: macOS 11 lleva
// un WKWebView que no los tiene (trampas.md). Se pinta junto a su botón, con
// position: absolute, y se abre hacia donde quepa.

import { type ReactNode, useCallback, useEffect, useId, useLayoutEffect, useRef, useState } from "react";

export function Emergente({
  boton,
  titulo,
  clase,
  prueba,
  children,
}: {
  /** Lo de dentro del botón. */
  boton: ReactNode;
  /** El nombre de la ventanita, para el lector de pantalla. */
  titulo: string;
  clase?: string;
  prueba?: string;
  /** Lo de dentro, con la función que la cierra (y devuelve el foco al botón). */
  children: (cerrar: () => void) => ReactNode;
}) {
  const [abierta, setAbierta] = useState(false);
  const [hacia, setHacia] = useState({ arriba: false, izquierda: false });
  const id = useId();
  const caja = useRef<HTMLSpanElement>(null);
  const panel = useRef<HTMLDivElement>(null);
  const boton_ = useRef<HTMLButtonElement>(null);
  const cerrar = useCallback((devolverFoco: boolean) => {
    setAbierta(false);
    if (devolverFoco) boton_.current?.focus();
  }, []);

  // Al abrir: hacia dónde cabe, y el foco dentro.
  useLayoutEffect(() => {
    if (!abierta || !panel.current || !caja.current) return;
    const b = caja.current.getBoundingClientRect();
    const p = panel.current.getBoundingClientRect();
    const { clientWidth: ancho, clientHeight: alto } = document.documentElement;
    setHacia({
      arriba: b.bottom + p.height + 12 > alto && b.top - p.height - 12 > 0,
      izquierda: b.left + p.width + 8 > ancho,
    });
    panel.current.querySelector<HTMLElement>("[aria-checked='true'], button, input, select")?.focus();
  }, [abierta]);

  // Un clic o el foco fuera la cierran, sin quitarle el foco a lo que se tocó.
  useEffect(() => {
    if (!abierta) return;
    const fuera = (e: Event) => {
      if (!caja.current?.contains(e.target as Node)) cerrar(false);
    };
    document.addEventListener("pointerdown", fuera, true);
    document.addEventListener("focusin", fuera);
    return () => {
      document.removeEventListener("pointerdown", fuera, true);
      document.removeEventListener("focusin", fuera);
    };
  }, [abierta, cerrar]);

  return (
    <span className={`emergente${clase ? ` ${clase}` : ""}`} ref={caja}>
      <button
        ref={boton_}
        aria-haspopup="dialog"
        aria-expanded={abierta}
        aria-controls={abierta ? id : undefined}
        onClick={() => setAbierta((v) => !v)}
        data-prueba={prueba}
      >
        {boton}
      </button>
      {abierta && (
        <div
          id={id}
          ref={panel}
          role="dialog"
          aria-label={titulo}
          className="emergente-panel"
          data-arriba={hacia.arriba || undefined}
          data-izquierda={hacia.izquierda || undefined}
          onKeyDown={(e) => {
            if (e.key === "Escape") {
              // Que no llegue al Estudio, que con Escape acaba el recorte.
              e.stopPropagation();
              cerrar(true);
            }
          }}
        >
          {children(() => cerrar(true))}
        </div>
      )}
    </span>
  );
}
