// La marca de Apolo y los iconos de la ventana (ADR 0015).
//
// `Marca` es el sol a trazo (empaquetado/marca.svg, copiado aquí por
// `make iconos`), en el color del sitio donde va: se inyecta como SVG para que
// herede currentColor. `IconoApp` es el icono de la aplicación a color.
//
// `Icono` son los iconos de línea de la interfaz, dibujados como los de Esfinge
// (componentes.tsx de Esfinge): caja de 18, trazo de 1,4, puntas redondeadas y
// el color del texto que acompañan.

import type { CSSProperties, InputHTMLAttributes, ReactNode } from "react";
import { ICONOS_FLUENT } from "./iconos-fluent";
import { ICONOS_BREEZE } from "./iconos-breeze";
import { ICONOS_YARU } from "./iconos-yaru";
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
  // Una cinta de marcapáginas: lo guardado para volver a usarlo.
  presets: <path d="M5 2.5h8a1 1 0 0 1 1 1v12l-5-3.2-5 3.2v-12a1 1 0 0 1 1-1zM7 6.5h4" />,
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
  // El mapa de diferencias: un marco con una mancha de calor.
  diferencias: (
    <>
      <rect x="2.5" y="3.5" width="13" height="11" rx="2" />
      <circle cx="10.5" cy="8.5" r="2.6" />
      <circle cx="10.5" cy="8.5" r="0.8" />
    </>
  ),
  guardar: <path d="M4 2.5h8l2.5 2.5v9.5a1 1 0 0 1-1 1h-9.5a1 1 0 0 1-1-1v-11a1 1 0 0 1 1-1zM6 2.5V6h5V2.5M5.5 15.5V11h7v4.5" />,
  // El botón de la barra lateral de macOS (ADR 0028): una ventana con su
  // panel a la izquierda.
  lateral: (
    <>
      <rect x="2" y="3.5" width="14" height="11" rx="2.2" />
      <path d="M7 3.5v11M3.9 6.3h1.4M3.9 8.3h1.4" />
    </>
  ),
  // La barra de encima de la imagen (ADR 0032): las escuadras de recortar, y
  // un cuadro que crece en diagonal.
  recortar: <path d="M5 1.5v11a.5.5 0 0 0 .5.5h11M1.5 5h11a.5.5 0 0 1 .5.5v11" />,
  redimensionar: (
    <>
      <rect x="2.5" y="9" width="6.5" height="6.5" rx="1" />
      <path d="M10.5 2.5h5v5M15.5 2.5 10 8" />
    </>
  ),
  candado: (
    <>
      <rect x="4" y="8" width="10" height="7.5" rx="1.5" />
      <path d="M6 8V5.5a3 3 0 0 1 6 0V8" />
    </>
  ),
  candadoAbierto: (
    <>
      <rect x="4" y="8" width="10" height="7.5" rx="1.5" />
      <path d="M6 8V5.5a3 3 0 0 1 5.8-1.1" />
    </>
  ),
  desplegar: <path d="m5 7 4 4 4-4" />,
  quitar: <path d="m5 5 8 8M13 5l-8 8" />,
};

export function Icono({ nombre, lado = 16 }: { nombre: keyof typeof DIBUJOS | string; lado?: number }) {
  // El logotipo de MCP, relleno y no a trazo: se reconoce por su forma, como
  // en la fila «MCP» de Esfinge, de donde viene el dibujo.
  if (nombre === "mcp") {
    return (
      <svg className="icono" width={lado} height={lado} viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
        <path d="M13.85 0a4.16 4.16 0 0 0-2.95 1.217L1.456 10.66a.835.835 0 0 0 0 1.18.835.835 0 0 0 1.18 0l9.442-9.442a2.49 2.49 0 0 1 3.541 0 2.49 2.49 0 0 1 0 3.541L8.59 12.97l-.1.1a.835.835 0 0 0 0 1.18.835.835 0 0 0 1.18 0l.1-.098 7.03-7.034a2.49 2.49 0 0 1 3.542 0l.049.05a2.49 2.49 0 0 1 0 3.54l-8.54 8.54a1.96 1.96 0 0 0 0 2.755l1.753 1.753a.835.835 0 0 0 1.18 0 .835.835 0 0 0 0-1.18l-1.753-1.753a.266.266 0 0 1 0-.394l8.54-8.54a4.185 4.185 0 0 0 0-5.9l-.05-.05a4.16 4.16 0 0 0-2.95-1.218c-.2 0-.401.02-.6.048a4.17 4.17 0 0 0-1.17-3.552A4.16 4.16 0 0 0 13.85 0m0 3.333a.84.84 0 0 0-.59.245L6.275 10.56a4.186 4.186 0 0 0 0 5.902 4.186 4.186 0 0 0 5.902 0L19.16 9.48a.835.835 0 0 0 0-1.18.835.835 0 0 0-1.18 0l-6.985 6.984a2.49 2.49 0 0 1-3.54 0 2.49 2.49 0 0 1 0-3.54l6.983-6.985a.835.835 0 0 0 0-1.18.84.84 0 0 0-.59-.245" />
      </svg>
    );
  }
  // En Windows, los iconos de Fluent (ADR 0027), con los mismos nombres. Se mira
  // al dibujar, no al cargar el módulo: el sistema llega de Rust después.
  // En Linux, los simbólicos de Yaru (ADR 0029), con los mismos nombres; con
  // KDE, los de Breeze (ADR 0031).
  const breeze = esKde() ? ICONOS_BREEZE[nombre] : undefined;
  if (breeze) {
    return (
      <svg className="icono" width={lado} height={lado} viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
        <g transform={breeze.t}>
          {breeze.o && <path d={breeze.o} opacity={0.5} />}
          <path d={breeze.d} fillRule={breeze.e ? "evenodd" : undefined} />
        </g>
      </svg>
    );
  }
  const yaru = document.documentElement.dataset.sistema === "linux" ? ICONOS_YARU[nombre] : undefined;
  if (yaru) {
    return (
      <svg className="icono" width={lado} height={lado} viewBox={yaru.v ?? "0 0 16 16"} fill="currentColor" aria-hidden="true">
        <g transform={yaru.t}>
          {yaru.o && <path d={yaru.o} opacity={0.5} />}
          <path d={yaru.d} />
        </g>
      </svg>
    );
  }
  const fluent = document.documentElement.dataset.sistema === "windows" ? ICONOS_FLUENT[nombre] : undefined;
  if (fluent) {
    const caja = lado < 18 ? 16 : 20;
    return (
      <svg className="icono" width={lado} height={lado} viewBox={`0 0 ${caja} ${caja}`} fill="currentColor" aria-hidden="true">
        <path d={fluent[caja]} />
      </svg>
    );
  }
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

/** Si la interfaz es la de Windows (ADR 0027). main.tsx pone el sistema antes
 *  de dibujar nada, así que se puede mirar al dibujar. */
export const esWindows = () => document.documentElement.dataset.sistema === "windows";

/** Si la interfaz es la de macOS 27 (ADR 0028). */
export const esMac = () => document.documentElement.dataset.sistema === "macos";

/** Si la interfaz es la de Ubuntu 26.10 (ADR 0029). */
export const esLinux = () => document.documentElement.dataset.sistema === "linux";
/** Linux con KDE Plasma (ADR 0031): Breeze en vez de libadwaita. */
export const esKde = () => esLinux() && document.documentElement.dataset.escritorio === "kde";

/** Si los ajustes van como en el sistema: SettingsCards en Windows (ADR 0027) y
 *  filas agrupadas, como en Ajustes del Sistema, en macOS (ADR 0028), y listas
 *  en tarjeta (*boxed list*) como en Ajustes de Ubuntu, en Linux (ADR 0029).
 *  El mismo marcado; cambia el CSS. En la web, las tarjetas de siempre. */
export const formularioNativo = () => esWindows() || esMac() || esLinux();

/** Una SettingsCard de WinUI en Windows (ADR 0027), una fila de formulario
 *  agrupado en macOS (ADR 0028) y una fila de lista en tarjeta en Linux
 *  (ADR 0029): el icono, el título, la descripción y el
 *  control a la derecha; lo de dentro va debajo, a todo el ancho. El control
 *  lleva su propio `aria-label`. */
export function TarjetaAjuste({
  icono,
  titulo,
  descripcion,
  control,
  prueba,
  children,
}: {
  /** El nombre de un icono, o lo que vaya en su sitio (el de la aplicación en «Acerca de»). */
  icono?: string | ReactNode;
  titulo: ReactNode;
  descripcion?: ReactNode;
  control?: ReactNode;
  prueba?: string;
  children?: ReactNode;
}) {
  return (
    <div className="tarjeta-ajuste" data-prueba={prueba}>
      <div className="tarjeta-fila">
        {typeof icono === "string" ? <Icono nombre={icono} lado={20} /> : icono}
        <div className="tarjeta-texto">
          <span className="tarjeta-titulo">{titulo}</span>
          {descripcion && <span className="tarjeta-descripcion">{descripcion}</span>}
        </div>
        {control && <div className="tarjeta-control">{control}</div>}
      </div>
      {children && <div className="tarjeta-abajo">{children}</div>}
    </div>
  );
}

/** Un interruptor: la casilla con el papel de `switch` (la que se marca, la
 *  que leen el lector de pantalla y las pruebas) y, a su lado, su botón como
 *  elemento propio, decorativo. Hace falta porque WebKit no deja meter nada
 *  dentro de un `<input>`: con el botón aparte, en macOS lleva el vidrio del
 *  sistema al pulsarlo (ADR 0028) y en Linux es el de libadwaita (ADR 0029).
 *  En Windows y en la web se queda oculto y el botón sigue siendo el fondo de
 *  la casilla, como siempre. */
export function Interruptor(props: InputHTMLAttributes<HTMLInputElement>) {
  return (
    <span className="interruptor">
      <input {...props} type="checkbox" role="switch" />
      <span className="pomo" aria-hidden="true" />
    </span>
  );
}

/** Lo recorrido de un deslizador, en `--lleno`: el de Fluent lo pinta en la
 *  pista (sistema-windows.css). En los demás sistemas no se usa. */
export function lleno(valor: number, min: number, max: number): CSSProperties {
  const p = max > min ? ((valor - min) / (max - min)) * 100 : 0;
  return { "--lleno": `${Math.min(100, Math.max(0, p))}%` } as CSSProperties;
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

