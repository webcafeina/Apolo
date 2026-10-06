// La marca de Apolo dentro de la ventana (ADR 0015).
//
// `Marca` es el sol a trazo (empaquetado/marca.svg, copiado aquí por
// `make iconos`), en el color del sitio donde va: se inyecta como SVG para que
// herede currentColor y el grosor se pueda ajustar desde CSS. `Icono` es el
// icono de la aplicación a color, para la bienvenida y «Acerca de».

import marca from "./marca/marca.svg?raw";
import icono from "./marca/icono.svg";

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

export function Icono({ lado = 96, clase }: { lado?: number; clase?: string }) {
  return <img className={clase} src={icono} width={lado} height={lado} alt="" draggable={false} />;
}
