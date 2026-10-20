//! `apolo webp`: acepta las opciones de `cwebp` tal cual y da el mismo fichero.
//!
//! Lo que cwebp escribe por pantalla se cuenta aquí en español, con los mismos
//! datos y en el mismo orden, pero no con el mismo texto: la promesa es el
//! fichero, no la consola. En inglés se usan las palabras de cwebp.

use std::io::Write;
use std::process::ExitCode;
use std::time::Instant;

use apolo_nucleo::cwebp::{self, Ayuda, OrdenCwebp};
use apolo_nucleo::idioma::{self, Idioma};
use apolo_nucleo::webp::{self, Codificado, Medida};
use apolo_nucleo::{presets, tr, trs};

use crate::comun;

pub fn ejecutar(args: &[String]) -> ExitCode {
    if args.is_empty() {
        ayuda_corta();
        return ExitCode::FAILURE;
    }
    let (objetivo, args) = match comun::objetivo(args) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("Error: {e}");
            return ExitCode::FAILURE;
        }
    };
    let args = &args[..];
    // `-apolo_preset <nombre>`: un preset guardado (en el Estudio o a mano) es
    // el punto de partida, y el resto de opciones se leen encima, como si
    // cwebp ya las tuviera puestas.
    let mut base = webp::OpcionesWebp::default();
    let mut resto: Vec<String> = Vec::with_capacity(args.len());
    let mut i = 0;
    while i < args.len() {
        if args[i] == "-apolo_preset" {
            let Some(nombre) = args.get(i + 1) else {
                eprintln!(
                    "Error: {}",
                    trs!(
                        "-apolo_preset necesita un nombre",
                        "-apolo_preset needs a name"
                    )
                );
                return ExitCode::FAILURE;
            };
            let Some(carpeta) = presets::carpeta() else {
                eprintln!(
                    "Error: {}",
                    trs!(
                        "no se encuentra la carpeta de configuración",
                        "can't find the settings folder"
                    )
                );
                return ExitCode::FAILURE;
            };
            match presets::cargar(&carpeta, nombre) {
                Some(p) => base = p.ajuste.webp,
                None => {
                    eprintln!(
                        "Error: {}",
                        tr!(
                            "no hay ningún preset «{nombre}» en {}",
                            "there's no preset “{nombre}” in {}",
                            carpeta.display()
                        )
                    );
                    return ExitCode::FAILURE;
                }
            }
            i += 2;
            continue;
        }
        resto.push(args[i].clone());
        i += 1;
    }
    let orden = match cwebp::leer_desde(&base, &resto) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("Error: {e}");
            return ExitCode::FAILURE;
        }
    };
    match orden.ayuda {
        Some(Ayuda::Corta) => {
            ayuda_corta();
            return ExitCode::SUCCESS;
        }
        Some(Ayuda::Larga) => {
            ayuda_larga();
            return ExitCode::SUCCESS;
        }
        None => {}
    }
    if orden.version {
        println!("{}", apolo_nucleo::motores::version_libwebp());
        return ExitCode::SUCCESS;
    }
    let r = match objetivo {
        Some(nota) => con_objetivo(&orden, nota),
        None => codificar(&orden),
    };
    match r {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Con `-apolo_objetivo`: la calidad la busca Apolo (ADR 0022), por el mismo
/// camino que el Estudio. La orden de cwebp con la calidad encontrada da el
/// mismo fichero.
fn con_objetivo(o: &OrdenCwebp, nota: f32) -> Result<(), String> {
    let Some(ruta) = &o.entrada else {
        return Err(trs!("Falta el fichero de entrada", "The input file is missing").into());
    };
    if o.opciones.sin_perdida {
        return Err(trs!(
            "-apolo_objetivo es para la calidad: sin pérdida no hay nada que buscar",
            "-apolo_objetivo is for the quality: lossless there's nothing to search"
        )
        .into());
    }
    let datos = std::fs::read(ruta)
        .map_err(|e| tr!("No se puede leer «{ruta}»: {e}", "Can't read “{ruta}”: {e}"))?;
    let img = comun::leer(&datos)?;
    let ajuste = apolo_nucleo::salida::Ajuste {
        formato: apolo_nucleo::salida::FormatoSalida::Webp,
        webp: o.opciones.clone(),
        objetivo: Some(nota),
        ..Default::default()
    };
    let r = apolo_nucleo::salida::codificar(&datos, &img, &ajuste, None)
        .map_err(|e| tr!("«{ruta}»: {e}", "“{ruta}”: {e}"))?;
    comun::informar_hallada(r.hallada, Some(nota));
    match o.salida.as_deref() {
        Some("-") => std::io::stdout()
            .write_all(&r.datos)
            .map_err(|e| e.to_string())?,
        Some(s) => std::fs::write(s, &r.datos)
            .map_err(|e| tr!("No se puede escribir «{s}»: {e}", "Can't write “{s}”: {e}"))?,
        None => eprintln!("{}", sin_salida()),
    }
    Ok(())
}

fn codificar(o: &OrdenCwebp) -> Result<(), String> {
    let hablar = !o.silencio;
    let Some(ruta) = &o.entrada else {
        ayuda_corta();
        return Err(trs!("Falta el fichero de entrada", "The input file is missing").into());
    };
    if o.sin_asm && hablar {
        eprintln!(
            "{}",
            trs!(
                "Aviso: -noasm se ignora en Apolo.",
                "Warning: -noasm is ignored in Apolo."
            )
        );
    }
    let op = &o.opciones;
    if hablar && op.sin_perdida {
        if op.tamano_objetivo > 0 || op.psnr_objetivo > 0.0 {
            eprintln!(
                "{}",
                trs!(
                    "Sin pérdida no se puede apuntar a un tamaño ni a un PSNR: se ignora.",
                    "Lossless can't target a size or a PSNR: ignored."
                )
            );
        }
        if op.limite_particion > 0 {
            eprintln!(
                "{}",
                trs!(
                    "Sin pérdida no hace falta límite de partición: se ignora.",
                    "Lossless needs no partition limit: ignored."
                )
            );
        }
    }
    if !op.validar() {
        return Err(trs!(
            "La configuración no es válida",
            "The configuration isn't valid"
        )
        .into());
    }

    let reloj = Instant::now();
    let datos = std::fs::read(ruta)
        .map_err(|e| tr!("No se puede leer «{ruta}»: {e}", "Can't read “{ruta}”: {e}"))?;

    if hablar && o.breve == 0 {
        match &o.salida {
            Some(s) => eprintln!("{}", tr!("Guardando «{s}»", "Saving file “{s}”")),
            None => eprintln!("{}\n", sin_salida()),
        }
    }

    let mut pintar = |p: i32| {
        eprint!("[{ruta}]: {p:3} %      \r");
        true
    };
    let progreso: Option<webp::Progreso> = if o.progreso && hablar {
        Some(&mut pintar)
    } else {
        None
    };
    let r =
        cwebp::ejecutar(o, &datos, progreso).map_err(|e| tr!("«{ruta}»: {e}", "“{ruta}”: {e}"))?;
    if o.detalle {
        eprintln!(
            "{}",
            tr!(
                "Lectura y codificación: {:.3} s",
                "Time to read and encode: {:.3}s",
                reloj.elapsed().as_secs_f64()
            )
        );
    }

    if let Some(d) = &o.volcado {
        match &r.volcado {
            Some(pgm) => {
                if let Err(e) = std::fs::write(d, pgm) {
                    eprintln!(
                        "{}",
                        tr!(
                            "Aviso: no se pudo volcar en «{d}»: {e}",
                            "Warning: couldn't dump to “{d}”: {e}"
                        )
                    );
                }
            }
            None => eprintln!(
                "{}",
                trs!(
                    "Aviso: sin pérdida no se puede volcar (-d).",
                    "Warning: lossless can't be dumped (-d)."
                )
            ),
        }
    }

    if let Some(s) = &o.salida {
        if s == "-" {
            std::io::stdout()
                .write_all(&r.datos)
                .map_err(|e| e.to_string())?;
        } else {
            std::fs::write(s, &r.datos)
                .map_err(|e| tr!("No se puede escribir «{s}»: {e}", "Can't write “{s}”: {e}"))?;
        }
    }

    if hablar {
        informe(o, ruta, &r);
    }
    Ok(())
}

fn informe(o: &OrdenCwebp, ruta: &str, r: &Codificado) {
    let s = &r.estadisticas;
    let breve = o.breve > 0;
    if !breve || o.extras.medir.is_none() {
        if breve {
            eprintln!("{:7} {:2.2}", s.bytes, s.psnr[3]);
        } else {
            let bpp = 8.0 * s.bytes as f64 / r.ancho as f64 / r.alto as f64;
            eprintln!("{}", tr!("Fichero:     {ruta}", "File:      {ruta}"));
            let (an, al) = (r.ancho, r.alto);
            if s.bytes_alfa > 0 {
                eprintln!(
                    "{}",
                    tr!(
                        "Dimensiones: {an} × {al} (con transparencia)",
                        "Dimension: {an} x {al} (with alpha)"
                    )
                );
            } else {
                eprintln!(
                    "{}",
                    tr!("Dimensiones: {an} × {al}", "Dimension: {an} x {al}")
                );
            }
            if r.sin_perdida {
                eprintln!(
                    "{}",
                    tr!(
                        "Salida:      {} bytes ({bpp:.2} bpp)",
                        "Output:    {} bytes ({bpp:.2} bpp)",
                        s.bytes
                    )
                );
                sin_perdida(s, "ARGB");
            } else {
                eprintln!(
                    "{}",
                    tr!(
                        "Salida:      {} bytes · PSNR Y-U-V-total {:2.2} {:2.2} {:2.2}   {:2.2} dB ({bpp:.2} bpp)",
                        "Output:    {} bytes Y-U-V-All-PSNR {:2.2} {:2.2} {:2.2}   {:2.2} dB ({bpp:.2} bpp)",
                        s.bytes,
                        s.psnr[0],
                        s.psnr[1],
                        s.psnr[2],
                        s.psnr[3]
                    )
                );
                con_perdida(s, o.opciones.poca_memoria);
            }
        }
    }
    if !breve && let Some(mapa) = &r.mapa {
        let mb_w = r.ancho.div_ceil(16) as usize;
        for fila in mapa.chunks(mb_w) {
            let linea: String = fila
                .iter()
                .map(|&c| match o.extras.mapa {
                    1 => ["+", "."].get(c as usize).unwrap_or(&"?").to_string(),
                    2 => [".", "-", "*", "X"]
                        .get(c as usize)
                        .unwrap_or(&"?")
                        .to_string(),
                    3 => format!("{c:02} "),
                    6 | 7 => format!("{c:3} "),
                    _ => format!("0x{c:02x} "),
                })
                .collect();
            eprintln!("{linea}");
        }
    }
    if let (Some(m), Some(v)) = (o.extras.medir, r.distorsion) {
        let nombre = match m {
            Medida::Psnr => "PSNR",
            Medida::Ssim => "SSIM",
            Medida::Lsim => "LSIM",
        };
        if breve {
            eprintln!("{:7} {:.4}", s.bytes, v[4]);
        } else {
            eprintln!(
                "{nombre}: B:{:.2} G:{:.2} R:{:.2} A:{:.2}  Total:{:.2}",
                v[0], v[1], v[2], v[3], v[4]
            );
        }
    }
    if !breve {
        let m = &r.metadatos;
        if m.icc.is_some() || m.exif.is_some() || m.xmp.is_some() {
            eprintln!("{}", trs!("Metadatos:", "Metadata:"));
            if let Some(n) = m.icc {
                eprintln!(
                    "{}",
                    tr!(
                        "  · Perfil ICC: {n:6} bytes",
                        "  * ICC profile:  {n:6} bytes"
                    )
                );
            }
            if let Some(n) = m.exif {
                eprintln!(
                    "{}",
                    tr!(
                        "  · EXIF:       {n:6} bytes",
                        "  * EXIF data:    {n:6} bytes"
                    )
                );
            }
            if let Some(n) = m.xmp {
                eprintln!(
                    "{}",
                    tr!(
                        "  · XMP:        {n:6} bytes",
                        "  * XMP data:     {n:6} bytes"
                    )
                );
            }
        }
    }
}

fn sin_perdida(s: &webp::Estadisticas, que: &str) {
    eprintln!(
        "{}",
        tr!(
            "Sin pérdida ({que}): {} bytes",
            "Lossless-{que} compressed size: {} bytes",
            s.sin_perdida_bytes
        )
    );
    eprintln!(
        "{}",
        tr!(
            "  · Cabecera: {} bytes; datos: {}",
            "  * Header size: {} bytes, image data size: {}",
            s.sin_perdida_cabecera,
            s.sin_perdida_datos
        )
    );
    let r = s.sin_perdida_rasgos;
    if r != 0 {
        let mut t = vec![];
        for (bit, n) in [
            (1, trs!("PREDICCIÓN", "PREDICTION")),
            (2, trs!("COLOR-CRUZADO", "CROSS-COLOR-TRANSFORM")),
            (4, trs!("RESTAR-VERDE", "SUBTRACT-GREEN")),
            (8, trs!("PALETA", "PALETTE")),
        ] {
            if r & bit != 0 {
                t.push(n);
            }
        }
        eprintln!(
            "{}",
            tr!(
                "  · Herramientas: {}",
                "  * Lossless features used: {}",
                t.join(" ")
            )
        );
    }
    let mut bits = tr!(
        "  · Bits: histograma={}",
        "  * Precision Bits: histogram={}",
        s.bits_histograma
    );
    if r & 1 != 0 {
        bits += &tr!(" predicción={}", " prediction={}", s.bits_transformada);
    }
    if r & 2 != 0 {
        bits += &tr!(" color-cruzado={}", " cross-color={}", s.bits_color_cruzado);
    }
    eprintln!("{bits}{}", tr!(" caché={}", " cache={}", s.bits_cache));
    if s.tamano_paleta > 0 {
        eprintln!(
            "{}",
            tr!(
                "  · Paleta: {} colores",
                "  * Palette size:   {}",
                s.tamano_paleta
            )
        );
    }
}

fn con_perdida(s: &webp::Estadisticas, detalle: bool) {
    let [i4, i16, salto] = s.bloques;
    let total = i4 + i16;
    if total > 0 {
        let pc = |n: i32| 100.0 * n as f64 / total as f64;
        let pcb = |n: i32| 100.0 * n as f64 / s.bytes as f64;
        eprintln!(
            "{}",
            tr!(
                "Bloques:     intra4 {i4:6} ({:.2} %) · intra16 {i16:6} ({:.2} %) · saltados {salto:6} ({:.2} %)",
                "block count: intra4: {i4:6} ({:.2}%) · intra16: {i16:6} ({:.2}%) · skipped: {salto:6} ({:.2}%)",
                pc(i4),
                pc(i16),
                pc(salto)
            )
        );
        eprintln!(
            "{}",
            tr!(
                "Bytes:       cabecera {:6} ({:.1} %) · modos {:6} ({:.1} %)",
                "bytes used:  header: {:6} ({:.1}%) · mode-partition: {:6} ({:.1}%)",
                s.bytes_cabecera[0],
                pcb(s.bytes_cabecera[0]),
                s.bytes_cabecera[1],
                pcb(s.bytes_cabecera[1])
            )
        );
        if s.bytes_alfa > 0 {
            eprintln!(
                "{}",
                tr!(
                    "             transparencia {:6} ({:.1} dB)",
                    "             transparency: {:6} ({:.1} dB)",
                    s.bytes_alfa,
                    s.psnr[4]
                )
            );
        }
        eprintln!(
            "{}",
            trs!(
                " Residuos         |segmento 1|segmento 2|segmento 3|segmento 4|  total",
                " Residuals bytes  |segment 1 |segment 2 |segment 3 |segment 4 |  total"
            )
        );
        let fila = |nombre: &str, v: &[i32; 4]| {
            let t: i32 = v.iter().sum();
            let celdas: String = v.iter().map(|x| format!("| {x:8} ")).collect();
            let pc = 100.0 * t as f64 / s.bytes as f64;
            eprintln!(
                "{}",
                tr!(
                    "{nombre:>17} {celdas}| {t:7}  ({pc:.1} %)",
                    "{nombre:>17} {celdas}| {t:7}  ({pc:.1}%)"
                )
            );
        };
        if detalle {
            fila(
                trs!("coef. intra4:", "intra4-coeffs:"),
                &s.bytes_residuos[0],
            );
            fila(
                trs!("coef. intra16:", "intra16-coeffs:"),
                &s.bytes_residuos[1],
            );
            fila(trs!("coef. croma:", "chroma coeffs:"), &s.bytes_residuos[2]);
        }
        let n: i32 = s.tamano_segmento.iter().sum();
        let celdas: String = s
            .tamano_segmento
            .iter()
            .map(|x| {
                let pc = (100.0 * *x as f64 / n.max(1) as f64 + 0.5) as i32;
                tr!("|     {pc:3} % ", "|     {pc:3}% ")
            })
            .collect();
        eprintln!(
            "{:>17} {celdas}| {n:7}",
            trs!("macrobloques:", "macroblocks:")
        );
        let valores = |nombre: &str, v: &[i32; 4]| {
            let celdas: String = v.iter().map(|x| format!("| {x:8} ")).collect();
            eprintln!("{nombre:>17} {celdas}|");
        };
        valores(trs!("cuantizador:", "quantizer:"), &s.cuantizador_segmento);
        valores(
            trs!("nivel de filtro:", "filter level:"),
            &s.filtro_segmento,
        );
    }
    if s.sin_perdida_bytes > 0 {
        sin_perdida(s, trs!("alfa", "alpha"));
    }
}

/// «Sin -o»: el resultado no se guarda (en inglés, como lo dice cwebp).
fn sin_salida() -> &'static str {
    trs!(
        "Sin -o: se codifica, pero el resultado no se guarda.",
        "No output file specified (no -o flag). Encoding will be performed, but its results discarded."
    )
}

fn ayuda_corta() {
    let version = apolo_nucleo::motores::version_libwebp();
    match idioma::actual() {
        Idioma::Es => {
            println!("Uso:\n\n   apolo webp [opciones] -q calidad entrada.png -o salida.webp\n");
            println!("La calidad va de 0 (peor) a 100 (muy buena). Lo típico es en torno a 80.\n");
            println!(
                "Acepta las mismas opciones que cwebp {version}. Lista completa: apolo webp -longhelp"
            );
        }
        Idioma::En => {
            println!("Usage:\n\n   apolo webp [options] -q quality input.png -o output.webp\n");
            println!(
                "where quality is between 0 (poor) to 100 (very good). Typical value is around 80.\n"
            );
            println!("Takes the same options as cwebp {version}. Full list: apolo webp -longhelp");
        }
    }
}

fn ayuda_larga() {
    match idioma::actual() {
        Idioma::Es => print!("{AYUDA_LARGA}"),
        Idioma::En => print!("{AYUDA_LARGA_EN}"),
    }
}

const AYUDA_LARGA: &str = r#"Uso:
 apolo webp [-preset <…>] [opciones] entrada [-o salida]

Mismas opciones que cwebp, con el mismo efecto: con las mismas opciones, el
fichero sale idéntico byte a byte. Entradas: PNG, JPEG, TIFF, WebP y PNM
(como cwebp) y además GIF, BMP y QOI.

  -h / -help ............. ayuda corta
  -H / -longhelp ......... esta ayuda
  -q <decimal> ........... calidad (0: pequeño … 100: grande), 75 por defecto
  -alpha_q <entero> ...... calidad de la transparencia (0…100), 100 por defecto
  -preset <nombre> ....... punto de partida: default, photo, picture,
                           drawing, icon, text. Va primero: reescribe lo demás
  -z <entero> ............ sin pérdida con el nivel dado, 0 (rápido) … 9 (lento)

  -m <entero> ............ método de compresión, 0 (rápido) … 6 (lento), 4
  -segments <entero> ..... segmentos (1…4), 4 por defecto
  -size <entero> ......... tamaño objetivo en bytes
  -psnr <decimal> ........ PSNR objetivo en dB (lo típico: 42)

  -s <ancho> <alto> ...... la entrada es YUV 4:2:0 crudo de ese tamaño
  -sns <entero> .......... modelado espacial del ruido (0 … 100), 50
  -f <entero> ............ fuerza del filtro (0 … 100), 60
  -sharpness <entero> .... nitidez del filtro (0: más … 7: menos), 0
  -strong ................ filtro fuerte (por defecto)
  -nostrong .............. filtro simple
  -sharp_yuv ............. conversión RGB→YUV más nítida (y lenta)
  -partition_limit <ent.>  limitar la calidad para que quepa la partición 0
                           (0 = sin degradar … 100 = completa)
  -pass <entero> ......... pasadas de análisis (1…10)
  -qrange <mín> <máx> .... calidad permitida (0 100 por defecto)
  -crop <x> <y> <an> <al>  recortar
  -resize <an> <al> ...... redimensionar, después de recortar (0 = proporcional)
  -resize_mode <modo> .... up_only, down_only o always (por defecto)
  -mt .................... varios hilos
  -low_memory ............ menos memoria (más lento)
  -map <entero> .......... enseñar un mapa por macrobloque
  -print_psnr ............ PSNR medio
  -print_ssim ............ SSIM medio
  -print_lsim ............ similitud local
  -d <fichero.pgm> ....... volcar el resultado (PGM)
  -alpha_method <entero> . compresión de la transparencia (0…1), 1
  -alpha_filter <modo> ... filtro de la transparencia: none, fast o best
  -exact ................. conservar el RGB bajo los píxeles transparentes
  -blend_alpha <hex> ..... mezclar con un fondo, p. ej. 0xc0e0d0
  -noalpha ............... descartar la transparencia
  -lossless .............. sin pérdida
  -near_lossless <entero>  casi sin pérdida (0…100; 100 = apagado)
  -hint <tipo> ........... photo, picture o graph

  -metadata <lista> ...... qué copiar: all, none (por defecto), exif, icc, xmp

Propias de Apolo (cwebp no las tiene):
  -apolo_preset <nombre> . partir de un preset guardado (apolo presets)
  -apolo_objetivo <nota> . buscar la calidad más baja que da esa nota
                           SSIMULACRA 2 (de 0 a 100)
  -apolo_enderezar ....... girar según la orientación EXIF; con ella, la
                           salida ya no es la de cwebp

  -short ................. salida resumida
  -quiet ................. sin salida
  -version ............... versión de libwebp
  -noasm ................. se acepta y se ignora
  -v ..................... tiempos
  -progress .............. progreso

Experimentales:
  -jpeg_like ............. tamaño parecido al de un JPEG
  -af .................... ajustar el filtro solo
  -pre <entero> .......... preprocesado
"#;

const AYUDA_LARGA_EN: &str = r#"Usage:
 apolo webp [-preset <...>] [options] in_file [-o out_file]

Same options as cwebp, with the same effect: with the same options, the file
comes out identical byte for byte. Inputs: PNG, JPEG, TIFF, WebP and PNM
(like cwebp), plus GIF, BMP and QOI.

  -h / -help ............. short help
  -H / -longhelp ......... this help
  -q <float> ............. quality factor (0:small..100:big), default=75
  -alpha_q <int> ......... transparency-compression quality (0..100), default=100
  -preset <string> ....... preset setting, one of: default, photo, picture,
                           drawing, icon, text. -preset must come first,
                           as it overwrites other parameters
  -z <int> ............... activates lossless preset with given
                           level in [0:fast, ..., 9:slowest]

  -m <int> ............... compression method (0=fast, 6=slowest), default=4
  -segments <int> ........ number of segments to use (1..4), default=4
  -size <int> ............ target size (in bytes)
  -psnr <float> .......... target PSNR (in dB. typically: 42)

  -s <int> <int> ......... input size (width x height) for raw YUV 4:2:0
  -sns <int> ............. spatial noise shaping (0:off, 100:max), default=50
  -f <int> ............... filter strength (0=off..100), default=60
  -sharpness <int> ....... filter sharpness (0:most .. 7:least sharp), default=0
  -strong ................ use strong filter instead of simple (default)
  -nostrong .............. use simple filter instead of strong
  -sharp_yuv ............. use sharper (and slower) RGB->YUV conversion
  -partition_limit <int> . limit quality to fit the 512k limit on
                           the first partition (0=no degradation ... 100=full)
  -pass <int> ............ analysis pass number (1..10)
  -qrange <min> <max> .... specifies the permissible quality range
                           (default: 0 100)
  -crop <x> <y> <w> <h> .. crop picture with the given rectangle
  -resize <w> <h> ........ resize picture (*after* any cropping)
                           (0 = keep the aspect ratio)
  -resize_mode <string> .. one of: up_only, down_only, always (default)
  -mt .................... use multi-threading if available
  -low_memory ............ reduce memory usage (slower encoding)
  -map <int> ............. print map of extra info
  -print_psnr ............ prints averaged PSNR distortion
  -print_ssim ............ prints averaged SSIM distortion
  -print_lsim ............ prints local-similarity distortion
  -d <file.pgm> .......... dump the compressed output (PGM file)
  -alpha_method <int> .... transparency-compression method (0..1), default=1
  -alpha_filter <string> . predictive filtering for alpha plane,
                           one of: none, fast (default) or best
  -exact ................. preserve RGB values in transparent area, default=off
  -blend_alpha <hex> ..... blend colors against background color,
                           e.g. 0xc0e0d0
  -noalpha ............... discard any transparency information
  -lossless .............. encode image losslessly, default=off
  -near_lossless <int> ... use near-lossless image preprocessing
                           (0..100=off), default=100
  -hint <string> ......... specify image characteristics hint,
                           one of: photo, picture or graph

  -metadata <string> ..... comma separated list of metadata to copy from the
                           input to the output if present. Valid values:
                           all, none (default), exif, icc, xmp

Apolo's own (cwebp doesn't have them):
  -apolo_preset <name> ... start from a saved preset (apolo presets)
  -apolo_objetivo <score>  find the lowest quality that gives that
                           SSIMULACRA 2 score (0 to 100)
  -apolo_enderezar ....... auto-rotate according to the EXIF orientation;
                           with it, the output is no longer cwebp's

  -short ................. condense printed message
  -quiet ................. don't print anything
  -version ............... print version number and exit
  -noasm ................. accepted and ignored
  -v ..................... verbose, e.g. print encoding/decoding times
  -progress .............. report encoding progress

Experimental Options:
  -jpeg_like ............. roughly match expected JPEG size
  -af .................... auto-adjust filter strength
  -pre <int> ............. pre-processing filter
"#;
