// La marca de Apolo y los iconos de la ventana (ADR 0015).
//
// `Marca` es el sol a trazo (empaquetado/marca.svg, copiado aquí por
// `make iconos`), en el color del sitio donde va: se inyecta como SVG para que
// herede currentColor. `IconoApp` es el icono de la aplicación a color.
//
// `Icono` son los iconos de línea de la interfaz, dibujados como los de Esfinge
// (componentes.tsx de Esfinge): caja de 18, trazo de 1,4, puntas redondeadas y
// el color del texto que acompañan.

import type { ReactNode } from "react";
import icono from "./marca/icono.svg";
import marca from "./marca/marca.svg?raw";

export function Marca({ lado = 24, clase }: { lado?: number; clase?: string }) {
  return (
    <span
      className={`marca-svg${clase ? ` ${clase}` : ""}`}
      style={{ width: lado, height: lado }}
      aria-hidden="true"
      dangerouslySetInnerHTML={{ __html: marca }}
    />
  );
}

export function IconoApp({ lado = 96, clase }: { lado?: number; clase?: string }) {
  return <img className={clase} src={icono} width={lado} height={lado} alt="" draggable={false} />;
}

const DIBUJOS: Record<string, ReactNode> = {
  // El comparador: un encuadre partido por el deslizador.
  estudio: (
    <>
      <rect x="2.5" y="3.5" width="13" height="11" rx="2" />
      <path d="M9 2v14" />
      <path d="M5 11.5l2-2.5 1.2 1.4" />
    </>
  ),
  // Varias imágenes apiladas.
  lotes: (
    <>
      <rect x="5.5" y="2.5" width="10" height="8.5" rx="1.6" />
      <path d="M2.5 6v7.5a2 2 0 0 0 2 2h8" />
      <path d="M8 9l2-2.4 1.6 1.8 1-1 1.4 1.6" />
    </>
  ),
  ajustes: (
    <>
      <path d="M3 5.5h12M3 12.5h12" />
      <circle cx="7" cy="5.5" r="1.8" />
      <circle cx="11.5" cy="12.5" r="1.8" />
    </>
  ),
  abrir: <path d="M2.5 5.5A1.5 1.5 0 0 1 4 4h3l1.5 1.8H14a1.5 1.5 0 0 1 1.5 1.5V13a1.5 1.5 0 0 1-1.5 1.5H4A1.5 1.5 0 0 1 2.5 13z" />,
  exportar: <path d="M9 2.5V11M5.5 7.5 9 11l3.5-3.5M3 13v1.5a1 1 0 0 0 1 1h10a1 1 0 0 0 1-1V13" />,
  copiar: (
    <>
      <rect x="6" y="6" width="9.5" height="9.5" rx="1.8" />
      <path d="M3.5 12V4.5a1 1 0 0 1 1-1H12" />
    </>
  ),
  pegar: (
    <>
      <rect x="3.5" y="3.5" width="11" height="12" rx="1.8" />
      <path d="M6.5 3.5V2.5h5v1M6.5 8h5M6.5 11h3.5" />
    </>
  ),
  deslizador: (
    <>
      <rect x="2.5" y="3.5" width="13" height="11" rx="2" />
      <path d="M9 2v14" />
    </>
  ),
  ladoALado: (
    <>
      <rect x="2" y="4" width="6.2" height="10" rx="1.4" />
      <rect x="9.8" y="4" width="6.2" height="10" rx="1.4" />
    </>
  ),
  guardar: <path d="M4 2.5h8l2.5 2.5v9.5a1 1 0 0 1-1 1h-9.5a1 1 0 0 1-1-1v-11a1 1 0 0 1 1-1zM6 2.5V6h5V2.5M5.5 15.5V11h7v4.5" />,
};

export function Icono({ nombre, lado = 16 }: { nombre: keyof typeof DIBUJOS | string; lado?: number }) {
  return (
    <svg
      className="icono"
      width={lado}
      height={lado}
      viewBox="0 0 18 18"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.4"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      {DIBUJOS[nombre]}
    </svg>
  );
}

/** La cabecera de cada sección, como la de Esfinge: 52 px, el título a la
 *  izquierda y los controles a la derecha. Se arrastra la ventana desde ella. */
export function Cabecera({ titulo, antetitulo, children }: { titulo: string; antetitulo?: string; children?: ReactNode }) {
  return (
    <header className="herramientas" data-tauri-drag-region>
      <div className="titulos" data-tauri-drag-region>
        {antetitulo && <span className="antetitulo">{antetitulo}</span>}
        <h1 title={titulo}>{titulo}</h1>
      </div>
      {children && <div className="aparte">{children}</div>}
    </header>
  );
}

