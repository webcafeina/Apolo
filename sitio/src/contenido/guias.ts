// Las guías (ADR 0034): respuestas a lo que se busca («convertir a WebP», «AVIF
// o WebP», «comprimir sin perder calidad»…), en los dos idiomas, con cifras
// medidas por Apolo (articulos.ts). Las órdenes son las de la página de la CLI,
// probadas con la CLI de verdad.
import type { Idioma } from "../i18n/rutas";
import { type Articulo, M, fotos, menos, peso, tablaFotos } from "./articulos";

const f80 = fotos(80);
const f90 = fotos(90);
const graficoPng = menos(M.graficos.png, M.graficos.original);
const graficoWebp = menos(M.graficos.webpSinPerdida, M.graficos.original);
const heic = (f: "jpeg" | "webp" | "avif") => menos(M.heic[f], M.heic.original);
const FECHA = "2026-10-27";

export const GUIAS: Record<Idioma, Articulo[]> = {
  es: [
    {
      pagina: "guiaWebp",
      publicado: FECHA,
      revisado: FECHA,
      titulo: "Cómo convertir imágenes a WebP, en Mac, Windows y Linux · Apolo",
      descripcion:
        "Convierte JPG y PNG a WebP gratis y sin subir tus fotos: con la aplicación Apolo, de una en una o por carpetas, o desde la terminal con las opciones de cwebp.",
      etiqueta: "Guía",
      h1: "Convertir imágenes a WebP.",
      entrada: "WebP pesa menos que JPEG y PNG, lo abren todos los navegadores de hoy y admite transparencia. Así se convierte, viendo lo que se pierde.",
      resumen: "Una foto o una carpeta entera, con la aplicación o con la terminal, y cuánto se ahorra de verdad.",
      relacionadas: ["guiaFormatos", "guiaSinPerder", "guiaWeb"],
      secciones: [
        {
          titulo: "Por qué WebP",
          parrafos: [
            `Con la misma calidad percibida, WebP pesa menos que JPEG: en nuestras medidas con 24 fotos, un ${f80.webp.frenteJpeg} % menos cuando no se distingue puesta al lado, y un ${f90.webp.frenteJpeg} % menos cuando no se distingue ni de cerca. Frente al PNG original, ahorra un ${f80.webp.frentePng} %.`,
            "Además guarda transparencia (como PNG) y tiene un modo sin pérdida que va muy bien con capturas de pantalla y logos. Lo abren Chrome, Safari, Firefox y Edge, y casi cualquier programa de edición actual.",
          ],
          enlaces: [{ texto: "WebP frente a AVIF y JPEG XL", pagina: "guiaFormatos" }],
        },
        {
          titulo: "Con la aplicación",
          pasos: [
            "Abre Apolo y arrastra la imagen a la ventana (JPG, PNG, HEIC, TIFF, GIF, BMP…).",
            "Elige WebP en el panel de formato.",
            "Mueve la calidad, o pide una nota con «Calidad por nota» y Apolo busca la calidad más baja que llega a ella.",
            "Mira el resultado con el deslizador o lado a lado, con el peso y la nota SSIMULACRA 2 de cada lado.",
            "Pulsa Exportar: el original no se toca.",
          ],
        },
        {
          titulo: "Muchas a la vez",
          parrafos: [
            "En Lotes, elige una carpeta, WebP y el preset: Apolo convierte todas en paralelo, repite las subcarpetas y deja el resultado al lado, sin sobrescribir nada.",
          ],
        },
        {
          titulo: "Desde la terminal",
          parrafos: ["La línea de comandos apolo acepta las opciones de cwebp y da el mismo fichero, byte a byte:"],
          comandos: [
            { titulo: "Una foto, con calidad 80", nombre: "apolo-webp", lineas: ["apolo webp -q 80 foto.png -o foto.webp"] },
            { titulo: "Una carpeta, con la calidad más baja que no se nota", nombre: "apolo-lote", lineas: ["apolo lote fotos/ --formato webp --objetivo 80"] },
            { titulo: "Una captura o un logo, sin pérdida", nombre: "apolo-webp", lineas: ["apolo webp -lossless -z 9 captura.png -o captura.webp"] },
          ],
          enlaces: [{ texto: "Todo lo que hace la línea de comandos", pagina: "cli" }],
        },
        {
          titulo: "Capturas y logos: sin pérdida",
          parrafos: [
            `En imágenes con texto, colores planos o transparencia, WebP sin pérdida guarda exactamente los mismos píxeles y pesa mucho menos: en nuestras capturas y logos de prueba, un ${graficoWebp} % menos que el PNG original, frente al ${graficoPng} % que se consigue optimizando el PNG con OxiPNG.`,
          ],
        },
      ],
    },
    {
      pagina: "guiaFormatos",
      publicado: FECHA,
      revisado: FECHA,
      titulo: "AVIF, WebP o JPEG XL: cuál usar, con medidas · Apolo",
      descripcion:
        "AVIF, WebP, JPEG XL y JPEG comparados a la misma calidad percibida con SSIMULACRA 2 en 24 fotos: cuánto pesa cada uno, dónde se abre y cuál conviene para la web o para guardar.",
      etiqueta: "Guía",
      h1: "AVIF, WebP o JPEG XL.",
      entrada: "Los tres pesan menos que JPEG, pero no lo mismo ni en todas partes. Los hemos medido a la misma calidad percibida.",
      resumen: "Cuánto pesa cada formato a la misma calidad, medido en 24 fotos, y cuál elegir según dónde vaya la imagen.",
      relacionadas: ["guiaSinPerder", "guiaWeb", "guiaWebp"],
      secciones: [
        {
          titulo: "Cómo lo hemos medido",
          parrafos: [
            "Comparar formatos «a calidad 80» no sirve: el 80 de AVIF no es el 80 de JPEG. Lo justo es igualar lo que se ve. Para eso usamos SSIMULACRA 2, una métrica de calidad percibida: con 80 no se distingue la imagen puesta al lado del original, y con 90 no se distingue ni de cerca.",
            "Apolo busca, para cada foto y cada formato, la calidad más baja que llega a esa nota, y compara los pesos.",
          ],
        },
        {
          titulo: "Si no se distingue al lado (nota 80)",
          tabla: tablaFotos(80, "es"),
        },
        {
          titulo: "Si no se distingue ni de cerca (nota 90)",
          tabla: tablaFotos(90, "es"),
          parrafos: [
            `Con calidad alta, JPEG XL se adelanta (${f90.jxl.frenteJpeg} % menos que JPEG) y AVIF le sigue de cerca (${f90.avif.frenteJpeg} %). Con calidad más justa, AVIF es el más ligero (${f80.avif.frenteJpeg} %). WebP queda en medio, y su ventaja crece con la calidad.`,
          ],
        },
        {
          titulo: "Dónde se abre cada uno",
          lista: [
            "WebP: en todos los navegadores actuales y en casi cualquier programa. El más seguro.",
            "AVIF: en los navegadores actuales (Chrome, Edge, Firefox y Safari desde hace años) y en cada vez más programas.",
            "JPEG XL: Safari lo abre; en otros navegadores llega poco a poco. Antes de servirlo solo, compruébalo en caniuse.com.",
            "JPEG: en todas partes, también en correos, impresoras y programas antiguos.",
          ],
        },
        {
          titulo: "Cuál elegir",
          lista: [
            "Para una web: AVIF, con WebP o JPEG de respaldo en un <picture> para lo que no lo abra.",
            "Para compartir sin pensar: WebP, o JPEG si va a un sitio antiguo.",
            "Para guardar con calidad alta: JPEG XL. Además recomprime los JPEG que ya tienes sin perder nada.",
            "Capturas, logos y gráficos: WebP sin pérdida o PNG optimizado.",
          ],
          enlaces: [
            { texto: "Optimizar imágenes para la web", pagina: "guiaWeb" },
            { texto: "Comprimir sin perder calidad", pagina: "guiaSinPerder" },
          ],
        },
        {
          titulo: "Hazlo con tus fotos",
          parrafos: ["Lo mismo, con tus imágenes: Apolo convierte a los cuatro formatos y se queda con el más ligero de cada foto."],
          comandos: [{ nombre: "apolo-lote", lineas: ["apolo lote fotos/ --formato avif,jxl,webp,jpeg --objetivo 80 --mas-ligero"] }],
        },
      ],
    },
    {
      pagina: "guiaSinPerder",
      publicado: FECHA,
      revisado: FECHA,
      titulo: "Comprimir imágenes sin perder calidad (y sin que se note) · Apolo",
      descripcion:
        "Cómo reducir el peso de una imagen sin perder calidad: compresión sin pérdida, o con pérdida que no se ve, medida con SSIMULACRA 2. Gratis, en tu ordenador y sin subir nada.",
      etiqueta: "Guía",
      h1: "Comprimir sin perder calidad.",
      entrada: "Hay dos maneras: sin perder ni un píxel, o perdiendo lo que el ojo no ve. La segunda ahorra mucho más, y se puede medir.",
      resumen: "Sin pérdida o con una pérdida que no se ve: qué ahorra cada camino y cómo saber que no se nota.",
      relacionadas: ["guiaFormatos", "guiaWebp", "guiaWeb"],
      secciones: [
        {
          titulo: "Sin pérdida: los mismos píxeles",
          parrafos: [
            `La imagen resultante es idéntica, píxel a píxel. Es lo que conviene en capturas de pantalla, logos y gráficos. En nuestras pruebas con ese tipo de imágenes, optimizar el PNG con OxiPNG ahorra un ${graficoPng} %, y pasarlo a WebP sin pérdida, un ${graficoWebp} %.`,
            "Con fotos, el ahorro sin pérdida es pequeño. La excepción: un JPEG que ya tienes se puede pasar a JPEG XL sin perder nada, y pesa bastante menos.",
          ],
          comandos: [
            { titulo: "PNG optimizado, como oxipng", nombre: "apolo-png", lineas: ["apolo png -o 4 captura.png"] },
            { titulo: "WebP sin pérdida", nombre: "apolo-webp", lineas: ["apolo webp -lossless -z 9 captura.png -o captura.webp"] },
          ],
        },
        {
          titulo: "Con una pérdida que no se ve",
          parrafos: [
            `Las fotos se comprimen mucho más si se acepta perder detalle que el ojo no aprecia. A una nota de 90 de SSIMULACRA 2, que no se distingue ni de cerca, nuestras 24 fotos pesan un ${f90.jxl.frentePng} % menos en JPEG XL y un ${f90.avif.frentePng} % menos en AVIF que el PNG original.`,
          ],
          tabla: tablaFotos(90, "es"),
        },
        {
          titulo: "Cómo saber que no se nota",
          parrafos: [
            "Apolo mide cada resultado con SSIMULACRA 2, la métrica de calidad percibida del proyecto JPEG XL: con 90 o más no se distingue; con 70, es alta calidad; con 50, se nota. Y enseña el original y el resultado con un deslizador, lado a lado o con un mapa de diferencias.",
            "En vez de probar calidades a ciegas, se puede pedir la nota: «Calidad por nota» busca, foto a foto, la calidad más baja que llega a ella.",
          ],
          comandos: [
            { titulo: "Una carpeta, sin que se note ni de cerca", nombre: "apolo-lote", lineas: ["apolo lote fotos/ --formato avif --objetivo 90"] },
            { titulo: "Medir un resultado frente a su original", nombre: "apolo-medir", lineas: ["apolo medir foto.png foto.avif"] },
          ],
        },
        {
          titulo: "Y el tamaño",
          parrafos: [
            "Lo que más pesa de una foto suele ser que es más grande de lo que se va a ver. Una foto de móvil de 4000 píxeles mostrada a 1200 lleva once veces más píxeles de los que hacen falta. Apolo la reduce al lado mayor que digas antes de comprimir.",
          ],
          comandos: [{ nombre: "apolo-lote", lineas: ["apolo lote fotos/ --formato avif --objetivo 90 --lado-mayor 2400"] }],
        },
      ],
    },
    {
      pagina: "guiaWeb",
      publicado: FECHA,
      revisado: FECHA,
      titulo: "Optimizar imágenes para la web: tamaño, formato y código · Apolo",
      descripcion:
        "Las imágenes son lo que más pesa en casi cualquier web. Cómo dejarlas en su tamaño, elegir AVIF o WebP con respaldo y servirlas con <picture>, lazy loading y medidas.",
      etiqueta: "Guía",
      h1: "Optimizar imágenes para la web.",
      entrada: "Una web carga rápido si sus imágenes pesan lo justo. Son cuatro decisiones: tamaño, formato, calidad y cómo se sirven.",
      resumen: "Tamaño, formato, calidad y el HTML para servirlas: lo que de verdad hace que una web cargue rápido.",
      relacionadas: ["guiaFormatos", "guiaSinPerder", "guiaWebp"],
      secciones: [
        {
          titulo: "1. El tamaño",
          parrafos: [
            "Cada imagen, al tamaño al que se ve, multiplicado por dos para las pantallas de alta densidad. Una foto que ocupa 800 píxeles de ancho necesita 1600, no los 4000 de la cámara. Apolo reduce al lado mayor que digas, sin agrandar nunca.",
          ],
          comandos: [{ nombre: "apolo-lote", lineas: ["apolo lote fotos/ --formato avif,webp --objetivo 80 --lado-mayor 1600"] }],
        },
        {
          titulo: "2. El formato",
          parrafos: [
            `AVIF es hoy el más ligero que abren todos los navegadores actuales: en nuestras medidas, un ${f80.avif.frenteJpeg} % menos que JPEG a la misma calidad percibida. WebP, como respaldo, y PNG o WebP sin pérdida para logos y capturas.`,
          ],
          enlaces: [{ texto: "AVIF, WebP o JPEG XL: las medidas", pagina: "guiaFormatos" }],
        },
        {
          titulo: "3. La calidad",
          parrafos: [
            "Para fotos de una web, una nota de 80 de SSIMULACRA 2 (no se distingue al lado del original) es un buen punto de partida; 90 si la foto es el producto. Con «Calidad por nota», Apolo encuentra la calidad que llega a esa nota en cada foto, en vez de usar el mismo número para todas.",
          ],
        },
        {
          titulo: "4. Cómo se sirven",
          parrafos: [
            "Con <picture>, el navegador coge el primer formato que sabe abrir. Las medidas (width y height) evitan que la página salte al cargar, y loading=\"lazy\" deja para después lo que está fuera de la pantalla (pero nunca la imagen principal de arriba).",
          ],
          codigo: [
            "<picture>",
            '  <source srcset="foto.avif" type="image/avif">',
            '  <source srcset="foto.webp" type="image/webp">',
            '  <img src="foto.jpg" alt="Lo que se ve en la foto" width="1600" height="1067" loading="lazy" decoding="async">',
            "</picture>",
          ],
        },
        {
          titulo: "Esta web, por ejemplo",
          parrafos: [
            "Todas las imágenes de esta web las ha optimizado Apolo: capturas en AVIF con WebP de respaldo, buscando la nota 90 porque llevan texto. En Webcafeína hacemos webs así, que cargan rápido.",
          ],
        },
      ],
    },
    {
      pagina: "guiaAgente",
      publicado: FECHA,
      revisado: FECHA,
      titulo: "Optimizar imágenes con un agente de IA (Claude, Cursor, VS Code) · Apolo",
      descripcion:
        "Pide a Claude, Cursor o VS Code que optimice, convierta o compare tus imágenes: con el servidor MCP de Apolo, en tu ordenador, sin subir las fotos y sin sobrescribir nada.",
      etiqueta: "Guía",
      h1: "Optimizar imágenes con un agente de IA.",
      entrada: "Con el servidor MCP de Apolo, tu agente convierte, compara y mide imágenes en tu ordenador. Tú se lo pides con palabras.",
      resumen: "Qué es el servidor MCP de Apolo, cómo se conecta a Claude, Cursor o VS Code y qué pedirle.",
      relacionadas: ["agentes", "guiaFormatos", "guiaWeb"],
      secciones: [
        {
          titulo: "Qué es",
          parrafos: [
            "MCP es la forma en que los agentes de IA usan herramientas de tu ordenador. Apolo trae un servidor MCP con once herramientas: analizar, optimizar, comparar formatos, medir la calidad, mirar una imagen, convertir carpetas en lotes y guardar presets. Las imágenes se procesan en tu ordenador con los mismos motores que la aplicación.",
          ],
        },
        {
          titulo: "Conectarlo",
          lista: [
            "Claude Desktop: en Apolo, la fila MCP de la barra lateral crea la extensión, que se instala una vez y se actualiza con la aplicación.",
            "Claude Code: una orden, la de tu sistema, en la página de agentes.",
            "Cursor, VS Code y los demás: la fila MCP de Apolo te da lo que hay que copiar.",
          ],
          enlaces: [{ texto: "Cómo conectarlo en cada agente", pagina: "agentes" }],
        },
        {
          titulo: "Qué pedirle",
          lista: [
            "«Convierte a AVIF las fotos de ~/Escritorio/viaje a 1920 píxeles y dime cuánto he ahorrado.»",
            "«Compara WebP, AVIF y JPEG XL para esta foto a la misma calidad y quédate con el más ligero.»",
            "«Mide cuánto se ha perdido entre foto.png y foto.webp.»",
            "«Quédate con la mitad izquierda de esta captura y pásala a WebP sin pérdida.»",
            "«Guarda un preset para las fotos del blog: AVIF, nota 80, 1600 píxeles.»",
          ],
        },
        {
          titulo: "Lo que no hace",
          parrafos: [
            "Nunca sobrescribe ni borra: siempre escribe ficheros nuevos al lado. Y no ve las imágenes que adjuntas en la conversación: se le dice dónde está el fichero en tu ordenador.",
            "Las fotos no salen de tu ordenador. Lo que el agente pide (rutas, nombres, miniaturas) sí entra en su conversación, y lo ve el proveedor del modelo.",
          ],
        },
      ],
    },
    {
      pagina: "guiaHeic",
      publicado: FECHA,
      revisado: FECHA,
      titulo: "Convertir HEIC a JPG (o WebP) gratis, sin subir tus fotos · Apolo",
      descripcion:
        "Las fotos del iPhone se guardan en HEIC y muchas webs y programas no las abren. Cómo pasarlas a JPG, WebP o AVIF en Mac, Windows o Linux, de una en una o por carpetas.",
      etiqueta: "Guía",
      h1: "Convertir HEIC a JPG.",
      entrada: "El iPhone guarda las fotos en HEIC, que pesa poco pero no se abre en todas partes. Apolo las abre y las pasa a JPG, WebP o AVIF.",
      resumen: "Pasar las fotos del iPhone a un formato que se abra en todas partes, sin subirlas a ningún sitio.",
      relacionadas: ["guiaWebp", "guiaFormatos", "guiaSinPerder"],
      secciones: [
        {
          titulo: "Por qué cuesta abrir un HEIC",
          parrafos: [
            "HEIC es el formato del iPhone desde hace años: pesa poco, pero muchas webs, programas y ordenadores con Windows o Linux no lo abren sin añadir nada. Pasarlo a JPG lo arregla; a WebP o AVIF, además, pesa menos.",
          ],
        },
        {
          titulo: "Con la aplicación",
          pasos: [
            "Arrastra el HEIC a Apolo, en Mac, Windows o Linux: lo abre sin instalar nada más.",
            "Elige JPEG para que se abra en todas partes, o WebP o AVIF si va a una web.",
            "Exporta. Para una carpeta entera de fotos del móvil, usa Lotes.",
          ],
        },
        {
          titulo: "Cuánto pesa cada opción",
          parrafos: [
            `Con la foto de ejemplo de libheif (${peso(M.heic.original, "es")} en HEIC), a una nota de 80 que no se distingue al lado: en JPEG, ${peso(M.heic.jpeg, "es")} (${heic("jpeg")} % menos); en WebP, ${peso(M.heic.webp, "es")} (${heic("webp")} % menos), y en AVIF, ${peso(M.heic.avif, "es")} (${heic("avif")} % menos). La nota se mide frente al HEIC abierto, que ya es una foto comprimida.`,
          ],
        },
        {
          titulo: "Desde la terminal",
          comandos: [
            { titulo: "Una carpeta de fotos del iPhone a JPG, sin que se note", nombre: "apolo-lote", lineas: ["apolo lote fotos-iphone/ --formato jpeg --objetivo 90"] },
            { titulo: "A WebP, para una web", nombre: "apolo-lote", lineas: ["apolo lote fotos-iphone/ --formato webp --objetivo 80 --lado-mayor 2048"] },
          ],
        },
        {
          titulo: "O que el iPhone no las guarde en HEIC",
          parrafos: [
            "En el iPhone, Ajustes › Cámara › Formatos › «Más compatible» guarda las fotos nuevas en JPG. Pesan más, y las que ya tienes siguen en HEIC.",
          ],
        },
      ],
    },
  ],
  en: [
    {
      pagina: "guiaWebp",
      publicado: FECHA,
      revisado: FECHA,
      titulo: "How to convert images to WebP on Mac, Windows and Linux · Apolo",
      descripcion:
        "Convert JPG and PNG to WebP for free without uploading your photos: with the Apolo app, one by one or whole folders, or from the terminal with cwebp's options.",
      etiqueta: "Guide",
      h1: "Convert images to WebP.",
      entrada: "WebP weighs less than JPEG and PNG, opens in every current browser and supports transparency. Here's how to convert, seeing what you lose.",
      resumen: "One photo or a whole folder, with the app or the terminal, and how much you really save.",
      relacionadas: ["guiaFormatos", "guiaSinPerder", "guiaWeb"],
      secciones: [
        {
          titulo: "Why WebP",
          parrafos: [
            `At the same perceived quality, WebP weighs less than JPEG: in our measurements with 24 photos, ${f80.webp.frenteJpeg}% less when you can't tell it apart side by side, and ${f90.webp.frenteJpeg}% less when you can't tell it apart even up close. Against the original PNG, it saves ${f80.webp.frentePng}%.`,
            "It also keeps transparency (like PNG) and has a lossless mode that works very well for screenshots and logos. Chrome, Safari, Firefox and Edge open it, and so does almost any current editing app.",
          ],
          enlaces: [{ texto: "WebP against AVIF and JPEG XL", pagina: "guiaFormatos" }],
        },
        {
          titulo: "With the app",
          pasos: [
            "Open Apolo and drag the image onto the window (JPG, PNG, HEIC, TIFF, GIF, BMP…).",
            "Choose WebP in the format panel.",
            "Move the quality, or ask for a score with “Quality by score” and Apolo finds the lowest quality that reaches it.",
            "Look at the result with the slider or side by side, with each side's size and SSIMULACRA 2 score.",
            "Press Export: the original is left untouched.",
          ],
        },
        {
          titulo: "Many at once",
          parrafos: [
            "In Batches, choose a folder, WebP and the preset: Apolo converts them all in parallel, repeats the subfolders and leaves the result next to it, without overwriting anything.",
          ],
        },
        {
          titulo: "From the terminal",
          parrafos: ["The apolo command line takes cwebp's options and gives the same file, byte for byte:"],
          comandos: [
            { titulo: "One photo, at quality 80", nombre: "apolo-webp", lineas: ["apolo webp -q 80 photo.png -o photo.webp"] },
            { titulo: "A folder, at the lowest quality that doesn't show", nombre: "apolo-lote", lineas: ["apolo lote photos/ --formato webp --objetivo 80"] },
            { titulo: "A screenshot or a logo, lossless", nombre: "apolo-webp", lineas: ["apolo webp -lossless -z 9 screenshot.png -o screenshot.webp"] },
          ],
          enlaces: [{ texto: "Everything the command line does", pagina: "cli" }],
        },
        {
          titulo: "Screenshots and logos: lossless",
          parrafos: [
            `In images with text, flat colors or transparency, lossless WebP keeps exactly the same pixels and weighs much less: in our test screenshots and logos, ${graficoWebp}% less than the original PNG, against the ${graficoPng}% you get optimizing the PNG with OxiPNG.`,
          ],
        },
      ],
    },
    {
      pagina: "guiaFormatos",
      publicado: FECHA,
      revisado: FECHA,
      titulo: "AVIF, WebP or JPEG XL: which to use, with measurements · Apolo",
      descripcion:
        "AVIF, WebP, JPEG XL and JPEG compared at the same perceived quality with SSIMULACRA 2 on 24 photos: how much each weighs, where it opens and which suits the web or archiving.",
      etiqueta: "Guide",
      h1: "AVIF, WebP or JPEG XL.",
      entrada: "All three weigh less than JPEG, but not by the same amount, nor everywhere. We measured them at the same perceived quality.",
      resumen: "How much each format weighs at the same quality, measured on 24 photos, and which to choose depending on where the image goes.",
      relacionadas: ["guiaSinPerder", "guiaWeb", "guiaWebp"],
      secciones: [
        {
          titulo: "How we measured",
          parrafos: [
            "Comparing formats “at quality 80” is meaningless: AVIF's 80 isn't JPEG's 80. The fair way is to match what you see. For that we use SSIMULACRA 2, a perceived-quality metric: at 80 you can't tell the image apart from the original side by side, and at 90 not even up close.",
            "For each photo and each format, Apolo finds the lowest quality that reaches that score, and we compare the sizes.",
          ],
        },
        {
          titulo: "If you can't tell side by side (score 80)",
          tabla: tablaFotos(80, "en"),
        },
        {
          titulo: "If you can't tell even up close (score 90)",
          tabla: tablaFotos(90, "en"),
          parrafos: [
            `At high quality, JPEG XL pulls ahead (${f90.jxl.frenteJpeg}% less than JPEG) with AVIF close behind (${f90.avif.frenteJpeg}%). At tighter quality, AVIF is the lightest (${f80.avif.frenteJpeg}%). WebP sits in between, and its advantage grows with quality.`,
          ],
        },
        {
          titulo: "Where each one opens",
          lista: [
            "WebP: in every current browser and almost any app. The safest.",
            "AVIF: in current browsers (Chrome, Edge, Firefox, and Safari for years now) and in more and more apps.",
            "JPEG XL: Safari opens it; other browsers are getting there. Before serving it alone, check caniuse.com.",
            "JPEG: everywhere, including email, printers and old software.",
          ],
        },
        {
          titulo: "Which to choose",
          lista: [
            "For a website: AVIF, with WebP or JPEG as a fallback in a <picture> for whatever can't open it.",
            "To share without thinking: WebP, or JPEG if it goes somewhere old.",
            "To archive at high quality: JPEG XL. It also recompresses the JPEGs you already have without losing anything.",
            "Screenshots, logos and graphics: lossless WebP or optimized PNG.",
          ],
          enlaces: [
            { texto: "Optimize images for the web", pagina: "guiaWeb" },
            { texto: "Compress without losing quality", pagina: "guiaSinPerder" },
          ],
        },
        {
          titulo: "Try it with your photos",
          parrafos: ["The same, with your images: Apolo converts to all four formats and keeps the lightest for each photo."],
          comandos: [{ nombre: "apolo-lote", lineas: ["apolo lote photos/ --formato avif,jxl,webp,jpeg --objetivo 80 --mas-ligero"] }],
        },
      ],
    },
    {
      pagina: "guiaSinPerder",
      publicado: FECHA,
      revisado: FECHA,
      titulo: "Compress images without losing quality (or without it showing) · Apolo",
      descripcion:
        "How to reduce an image's size without losing quality: lossless compression, or lossy compression you can't see, measured with SSIMULACRA 2. Free, on your computer, nothing uploaded.",
      etiqueta: "Guide",
      h1: "Compress without losing quality.",
      entrada: "There are two ways: without losing a single pixel, or losing what the eye can't see. The second saves far more, and it can be measured.",
      resumen: "Lossless or with a loss you can't see: what each path saves and how to know it doesn't show.",
      relacionadas: ["guiaFormatos", "guiaWebp", "guiaWeb"],
      secciones: [
        {
          titulo: "Lossless: the same pixels",
          parrafos: [
            `The resulting image is identical, pixel for pixel. It's what you want for screenshots, logos and graphics. In our tests with that kind of image, optimizing the PNG with OxiPNG saves ${graficoPng}%, and turning it into lossless WebP, ${graficoWebp}%.`,
            "With photos, lossless savings are small. The exception: a JPEG you already have can be turned into JPEG XL without losing anything, and it weighs quite a bit less.",
          ],
          comandos: [
            { titulo: "Optimized PNG, like oxipng", nombre: "apolo-png", lineas: ["apolo png -o 4 screenshot.png"] },
            { titulo: "Lossless WebP", nombre: "apolo-webp", lineas: ["apolo webp -lossless -z 9 screenshot.png -o screenshot.webp"] },
          ],
        },
        {
          titulo: "With a loss you can't see",
          parrafos: [
            `Photos compress much further if you accept losing detail the eye doesn't notice. At a SSIMULACRA 2 score of 90, which you can't tell apart even up close, our 24 photos weigh ${f90.jxl.frentePng}% less in JPEG XL and ${f90.avif.frentePng}% less in AVIF than the original PNG.`,
          ],
          tabla: tablaFotos(90, "en"),
        },
        {
          titulo: "How to know it doesn't show",
          parrafos: [
            "Apolo measures each result with SSIMULACRA 2, the perceived-quality metric from the JPEG XL project: at 90 or more you can't tell them apart; at 70, it's high quality; at 50, it shows. And it shows the original and the result with a slider, side by side or with a difference map.",
            "Instead of trying qualities blindly, you can ask for the score: “Quality by score” finds, photo by photo, the lowest quality that reaches it.",
          ],
          comandos: [
            { titulo: "A folder, without it showing even up close", nombre: "apolo-lote", lineas: ["apolo lote photos/ --formato avif --objetivo 90"] },
            { titulo: "Measure a result against its original", nombre: "apolo-medir", lineas: ["apolo medir photo.png photo.avif"] },
          ],
        },
        {
          titulo: "And the size",
          parrafos: [
            "What usually weighs most in a photo is that it's bigger than it will be seen. A 4000-pixel phone photo shown at 1200 carries eleven times more pixels than needed. Apolo shrinks it to the longest side you ask for before compressing.",
          ],
          comandos: [{ nombre: "apolo-lote", lineas: ["apolo lote photos/ --formato avif --objetivo 90 --lado-mayor 2400"] }],
        },
      ],
    },
    {
      pagina: "guiaWeb",
      publicado: FECHA,
      revisado: FECHA,
      titulo: "Optimize images for the web: size, format and code · Apolo",
      descripcion:
        "Images are the heaviest part of almost any website. How to bring them to size, choose AVIF or WebP with a fallback and serve them with <picture>, lazy loading and dimensions.",
      etiqueta: "Guide",
      h1: "Optimize images for the web.",
      entrada: "A website loads fast if its images weigh just what they need. It comes down to four decisions: size, format, quality and how they're served.",
      resumen: "Size, format, quality and the HTML to serve them: what really makes a website load fast.",
      relacionadas: ["guiaFormatos", "guiaSinPerder", "guiaWebp"],
      secciones: [
        {
          titulo: "1. Size",
          parrafos: [
            "Each image at the size it's shown, times two for high-density screens. A photo that takes 800 pixels of width needs 1600, not the camera's 4000. Apolo shrinks to the longest side you ask for, and never enlarges.",
          ],
          comandos: [{ nombre: "apolo-lote", lineas: ["apolo lote photos/ --formato avif,webp --objetivo 80 --lado-mayor 1600"] }],
        },
        {
          titulo: "2. Format",
          parrafos: [
            `AVIF is today the lightest format every current browser opens: in our measurements, ${f80.avif.frenteJpeg}% less than JPEG at the same perceived quality. WebP as a fallback, and PNG or lossless WebP for logos and screenshots.`,
          ],
          enlaces: [{ texto: "AVIF, WebP or JPEG XL: the measurements", pagina: "guiaFormatos" }],
        },
        {
          titulo: "3. Quality",
          parrafos: [
            "For website photos, a SSIMULACRA 2 score of 80 (you can't tell it apart from the original side by side) is a good starting point; 90 if the photo is the product. With “Quality by score”, Apolo finds the quality that reaches that score for each photo, instead of using the same number for all of them.",
          ],
        },
        {
          titulo: "4. How they're served",
          parrafos: [
            "With <picture>, the browser takes the first format it can open. Dimensions (width and height) stop the page from jumping while it loads, and loading=\"lazy\" defers whatever is off screen (but never the main image at the top).",
          ],
          codigo: [
            "<picture>",
            '  <source srcset="photo.avif" type="image/avif">',
            '  <source srcset="photo.webp" type="image/webp">',
            '  <img src="photo.jpg" alt="What the photo shows" width="1600" height="1067" loading="lazy" decoding="async">',
            "</picture>",
          ],
        },
        {
          titulo: "This website, for example",
          parrafos: [
            "Every image on this website was optimized by Apolo: screenshots in AVIF with a WebP fallback, at a score of 90 because they contain text. At Webcafeína we build websites like this, that load fast.",
          ],
        },
      ],
    },
    {
      pagina: "guiaAgente",
      publicado: FECHA,
      revisado: FECHA,
      titulo: "Optimize images with an AI agent (Claude, Cursor, VS Code) · Apolo",
      descripcion:
        "Ask Claude, Cursor or VS Code to optimize, convert or compare your images: with Apolo's MCP server, on your computer, without uploading your photos or overwriting anything.",
      etiqueta: "Guide",
      h1: "Optimize images with an AI agent.",
      entrada: "With Apolo's MCP server, your agent converts, compares and measures images on your computer. You just ask in plain words.",
      resumen: "What Apolo's MCP server is, how to connect it to Claude, Cursor or VS Code and what to ask it.",
      relacionadas: ["agentes", "guiaFormatos", "guiaWeb"],
      secciones: [
        {
          titulo: "What it is",
          parrafos: [
            "MCP is how AI agents use tools on your computer. Apolo ships an MCP server with eleven tools: analyze, optimize, compare formats, measure quality, look at an image, convert folders in batches and save presets. Images are processed on your computer with the same engines as the app.",
          ],
        },
        {
          titulo: "Connect it",
          lista: [
            "Claude Desktop: in Apolo, the MCP row in the sidebar creates the extension, which you install once and updates with the app.",
            "Claude Code: one command, your system's, on the agents page.",
            "Cursor, VS Code and the rest: Apolo's MCP row gives you what to copy.",
          ],
          enlaces: [{ texto: "How to connect it to each agent", pagina: "agentes" }],
        },
        {
          titulo: "What to ask",
          lista: [
            "“Convert the photos in ~/Desktop/trip to AVIF at 1920 pixels and tell me how much I saved.”",
            "“Compare WebP, AVIF and JPEG XL for this photo at the same quality and keep the lightest.”",
            "“Measure how much was lost between photo.png and photo.webp.”",
            "“Keep the left half of this screenshot and make it a lossless WebP.”",
            "“Save a preset for blog photos: AVIF, score 80, 1600 pixels.”",
          ],
        },
        {
          titulo: "What it doesn't do",
          parrafos: [
            "It never overwrites or deletes: it always writes new files alongside. And it doesn't see images you attach to the conversation: you tell it where the file is on your computer.",
            "Your photos never leave your computer. What the agent asks for (paths, names, thumbnails) does go into its conversation, and the model provider sees it.",
          ],
        },
      ],
    },
    {
      pagina: "guiaHeic",
      publicado: FECHA,
      revisado: FECHA,
      titulo: "Convert HEIC to JPG (or WebP) for free, without uploading · Apolo",
      descripcion:
        "iPhone photos are saved as HEIC and many websites and apps can't open them. How to turn them into JPG, WebP or AVIF on Mac, Windows or Linux, one by one or whole folders.",
      etiqueta: "Guide",
      h1: "Convert HEIC to JPG.",
      entrada: "The iPhone saves photos as HEIC, which is light but doesn't open everywhere. Apolo opens them and turns them into JPG, WebP or AVIF.",
      resumen: "Turn iPhone photos into a format that opens everywhere, without uploading them anywhere.",
      relacionadas: ["guiaWebp", "guiaFormatos", "guiaSinPerder"],
      secciones: [
        {
          titulo: "Why HEIC is hard to open",
          parrafos: [
            "HEIC has been the iPhone's format for years: it's light, but many websites, apps and Windows or Linux computers can't open it without adding something. Turning it into JPG fixes that; into WebP or AVIF, it also weighs less.",
          ],
        },
        {
          titulo: "With the app",
          pasos: [
            "Drag the HEIC onto Apolo, on Mac, Windows or Linux: it opens without installing anything else.",
            "Choose JPEG so it opens everywhere, or WebP or AVIF if it's going to a website.",
            "Export. For a whole folder of phone photos, use Batches.",
          ],
        },
        {
          titulo: "How much each option weighs",
          parrafos: [
            `With libheif's sample photo (${peso(M.heic.original, "en")} as HEIC), at a score of 80 you can't tell apart side by side: as JPEG, ${peso(M.heic.jpeg, "en")} (${heic("jpeg")}% less); as WebP, ${peso(M.heic.webp, "en")} (${heic("webp")}% less), and as AVIF, ${peso(M.heic.avif, "en")} (${heic("avif")}% less). The score is measured against the opened HEIC, which is already a compressed photo.`,
          ],
        },
        {
          titulo: "From the terminal",
          comandos: [
            { titulo: "A folder of iPhone photos to JPG, without it showing", nombre: "apolo-lote", lineas: ["apolo lote iphone-photos/ --formato jpeg --objetivo 90"] },
            { titulo: "To WebP, for a website", nombre: "apolo-lote", lineas: ["apolo lote iphone-photos/ --formato webp --objetivo 80 --lado-mayor 2048"] },
          ],
        },
        {
          titulo: "Or stop the iPhone saving HEIC",
          parrafos: [
            "On the iPhone, Settings › Camera › Formats › “Most Compatible” saves new photos as JPG. They weigh more, and the ones you already have stay HEIC.",
          ],
        },
      ],
    },
  ],
};
