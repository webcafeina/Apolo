//! `apolo lote`: lo mismo que la sección Lotes de la ventana (ADR 0019).
//!
//! apolo lote fotos/ --salida fotos-web --preset "Fotos web" -- -q 80
//! apolo lote fotos/ --formato webp,jpeg --mas-ligero
//!
//! Un ajuste por cada `--preset` y cada formato de `--formato` (WebP si no hay
//! ninguno). Con uno solo, las opciones de su herramienta (cwebp, cjpeg u
//! oxipng) van detrás de `--`, encima del preset. Con `--mas-ligero`, de cada
//! imagen se guarda solo el formato que menos pese (ADR 0020). Con
//! `--lado-mayor` o `--porcentaje`, el tamaño de todas, encima del preset
//! (ADR 0032). Un lote no recorta nunca. El progreso va
//! a la salida de errores; el resumen, a la normal.

use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::time::Instant;

use apolo_nucleo::formatos::png;
use apolo_nucleo::lote::{self, Resumen};
use apolo_nucleo::proceso::{Medida, Redimension};
use apolo_nucleo::salida::{Ajuste, FormatoSalida};
use apolo_nucleo::{cwebp, jpeg, presets, tr, trs};

use crate::comun::decimal;

pub struct Peticion {
    pub entradas: Vec<PathBuf>,
    pub salida: Option<PathBuf>,
    /// Presets guardados: uno por formato que se quiera.
    pub presets: Vec<String>,
    /// Formatos sin preset («webp,jpeg»), con sus opciones por defecto.
    pub formatos: Vec<String>,
    /// De cada imagen, solo el formato que menos pese.
    pub mas_ligero: bool,
    pub hilos: Option<usize>,
    /// Medir la nota de cada fichero (ADR 0022).
    pub medir: bool,
    /// Buscar, imagen a imagen, la calidad que da esta nota.
    pub objetivo: Option<f32>,
    /// Reducir cada imagen a este lado mayor, encima del preset (ADR 0032).
    pub lado_mayor: Option<u32>,
    /// O a este tanto por ciento.
    pub porcentaje: Option<f64>,
    /// Las opciones de la herramienta, detrás de `--`: solo con un formato.
    pub herramienta: Vec<String>,
    pub silencio: bool,
}

fn bytes(n: u64) -> String {
    let t = match n {
        n if n < 1024 => format!("{n} B"),
        n if n < 1024 * 1024 => format!("{:.1} KB", n as f64 / 1024.0),
        n => format!("{:.1} MB", n as f64 / 1024.0 / 1024.0),
    };
    decimal(t)
}

fn ahorro(antes: u64, despues: u64) -> String {
    if antes == 0 {
        return "—".into();
    }
    let p = (1.0 - despues as f64 / antes as f64) * 100.0;
    if p < 0.0 {
        tr!("+{:.0} %", "+{:.0}%", -p)
    } else {
        tr!("−{p:.0} %", "−{p:.0}%")
    }
}

fn formato(s: &str) -> Result<FormatoSalida, String> {
    match s.trim().to_lowercase().as_str() {
        "webp" => Ok(FormatoSalida::Webp),
        "jpeg" | "jpg" | "mozjpeg" => Ok(FormatoSalida::Jpeg),
        "png" | "oxipng" => Ok(FormatoSalida::Png),
        "qoi" => Ok(FormatoSalida::Qoi),
        "avif" | "avifenc" => Ok(FormatoSalida::Avif),
        "jxl" | "jpegxl" | "cjxl" => Ok(FormatoSalida::Jxl),
        otro => Err(tr!(
            "no hay formato «{otro}»: webp, jpeg, png, qoi, avif o jxl",
            "there's no format “{otro}”: webp, jpeg, png, qoi, avif or jxl"
        )),
    }
}

/// Los ajustes del lote: uno por preset y por formato pedido. Sin nada, WebP.
fn ajustes(p: &Peticion) -> Result<Vec<Ajuste>, String> {
    let mut v = Vec::new();
    if !p.presets.is_empty() {
        let carpeta = presets::carpeta().ok_or(trs!(
            "No se encuentra la carpeta de configuración",
            "Can't find the settings folder"
        ))?;
        for nombre in &p.presets {
            v.push(
                presets::cargar(&carpeta, nombre)
                    .ok_or_else(|| {
                        tr!(
                            "No hay ningún preset «{nombre}» (apolo presets los lista)",
                            "There's no preset “{nombre}” (apolo presets lists them)"
                        )
                    })?
                    .ajuste,
            );
        }
    }
    for lista in &p.formatos {
        for f in lista.split(',') {
            v.push(Ajuste {
                formato: formato(f)?,
                ..Default::default()
            });
        }
    }
    if v.is_empty() {
        v.push(Ajuste::default());
    }
    if !p.herramienta.is_empty() {
        let [a] = v.as_mut_slice() else {
            return Err(trs!(
                "Las opciones detrás de «--» van con un solo formato",
                "The options after “--” go with a single format"
            )
            .into());
        };
        match a.formato {
            FormatoSalida::Webp => {
                let o = cwebp::leer_desde(&a.webp, &p.herramienta).map_err(|e| e.0)?;
                if o.entrada.is_some() || o.salida.is_some() {
                    return Err(solo_opciones().into());
                }
                a.webp = o.opciones;
            }
            FormatoSalida::Jpeg => {
                a.jpeg = jpeg::OpcionesJpeg::leer_desde(&a.jpeg, &p.herramienta)?
            }
            FormatoSalida::Png => {
                let o = png::leer_orden_desde(&a.png, &p.herramienta).map_err(|e| e.to_string())?;
                if !o.entradas.is_empty() || o.salida.is_some() {
                    return Err(solo_opciones().into());
                }
                a.png = o.opciones;
            }
            FormatoSalida::Qoi => {
                return Err(trs!("QOI no tiene opciones", "QOI has no options").into());
            }
            FormatoSalida::Avif => {
                let o = apolo_nucleo::formatos::avif::leer_orden_desde(&a.avif, &p.herramienta)
                    .map_err(|e| e.to_string())?;
                if !o.ficheros.is_empty() {
                    return Err(solo_opciones().into());
                }
                a.avif = o.opciones;
            }
            FormatoSalida::Jxl => {
                let o = apolo_nucleo::formatos::jxl::leer_orden_desde(&a.jxl, &p.herramienta)
                    .map_err(|e| e.to_string())?;
                if !o.ficheros.is_empty() {
                    return Err(solo_opciones().into());
                }
                a.jxl = o.opciones;
            }
        }
    }
    if let Some(nota) = p.objetivo {
        for a in &mut v {
            a.objetivo = Some(nota);
        }
    }
    // El tamaño, encima del preset, como en la ventana: un máximo que nunca
    // agranda.
    let medida = match (p.lado_mayor, p.porcentaje) {
        (Some(0), _) => {
            return Err(trs!(
                "--lado-mayor tiene que ser de 1 px o más",
                "--lado-mayor must be 1 px or more"
            )
            .into());
        }
        (Some(l), _) => Some(Medida::LadoMayor(l)),
        (_, Some(pc)) if !(pc.is_finite() && pc > 0.0 && pc <= 100.0) => {
            return Err(trs!(
                "--porcentaje va de más de 0 a 100: nunca agranda",
                "--porcentaje goes from above 0 to 100: it never enlarges"
            )
            .into());
        }
        (_, Some(pc)) => Some(Medida::Porcentaje(pc)),
        (None, None) => None,
    };
    if let Some(medida) = medida {
        for a in &mut v {
            let antes = a.proceso.redimension.unwrap_or_default();
            a.proceso.redimension = Some(Redimension {
                medida,
                solo_reducir: true,
                ..antes
            });
        }
    }
    if v.iter().any(|a| !a.webp.validar()) {
        return Err(trs!(
            "La configuración no es válida",
            "The configuration isn't valid"
        )
        .into());
    }
    Ok(v)
}

fn solo_opciones() -> &'static str {
    trs!(
        "Detrás de «--» van solo opciones: las entradas van delante y la salida con --salida",
        "Only options go after “--”: the inputs go before and the output with --salida"
    )
}

pub fn ejecutar(p: Peticion) -> ExitCode {
    let ajustes = match ajustes(&p) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("Error: {e}");
            return ExitCode::FAILURE;
        }
    };
    let Some(salida) = p
        .salida
        .clone()
        .or_else(|| lote::salida_sugerida(&p.entradas, &lote::sufijo(&ajustes)))
    else {
        eprintln!("Error: {}", trs!("falta --salida", "--salida is missing"));
        return ExitCode::FAILURE;
    };
    let elementos = lote::recoger(&p.entradas, Some(&salida));
    if elementos.is_empty() {
        eprintln!(
            "Error: {}",
            trs!(
                "no hay imágenes que convertir",
                "there are no images to convert"
            )
        );
        return ExitCode::FAILURE;
    }
    let total = elementos.len();
    if !p.silencio {
        eprintln!(
            "{total} {} → {}",
            if total == 1 {
                trs!("imagen", "image")
            } else {
                trs!("imágenes", "images")
            },
            salida.display()
        );
        for a in &ajustes {
            let args = apolo_nucleo::salida::argumentos(a);
            let herramienta = a.formato.herramienta();
            if args.is_empty() {
                eprintln!(
                    "{}",
                    tr!(
                        "{}: como {herramienta} (sin opciones)",
                        "{}: as {herramienta} (no options)",
                        a.formato.nombre()
                    )
                );
            } else {
                eprintln!(
                    "{}",
                    tr!(
                        "{}: como {herramienta} {}",
                        "{}: as {herramienta} {}",
                        a.formato.nombre(),
                        args.join(" ")
                    )
                );
            }
        }
        if p.mas_ligero && ajustes.len() > 1 {
            eprintln!(
                "{}",
                trs!(
                    "De cada imagen se guarda solo el más ligero.",
                    "Of each image, only the lightest is saved."
                )
            );
        }
    }

    let reloj = Instant::now();
    let hechos = Mutex::new(Vec::new());
    lote::ejecutar(
        &elementos,
        &salida,
        &ajustes,
        p.mas_ligero,
        p.medir,
        p.hilos.unwrap_or_else(lote::hilos_por_defecto),
        &AtomicBool::new(false),
        &|h| {
            let mut v = hechos.lock().unwrap();
            if !p.silencio {
                let n = v.len() + 1;
                match &h.resultado {
                    Ok(salidas) => {
                        let partes: Vec<String> = salidas
                            .iter()
                            .map(|s| {
                                let mut t = format!(
                                    "{} {} {}",
                                    s.formato.nombre(),
                                    bytes(s.bytes),
                                    ahorro(h.bytes_entrada, s.bytes)
                                );
                                if let Some(q) = s.calidad {
                                    t.push_str(&tr!(" calidad {q}", " quality {q}"));
                                }
                                if let Some(n) = s.nota {
                                    t.push_str(&tr!(" nota {}", " score {}", nota(n)));
                                }
                                t
                            })
                            .collect();
                        eprintln!(
                            "[{n}/{total}] {}  {} → {}",
                            h.relativa.display(),
                            bytes(h.bytes_entrada),
                            partes.join(", ")
                        )
                    }
                    Err(e) => eprintln!("[{n}/{total}] {}  ERROR: {e}", h.relativa.display()),
                }
            }
            v.push(h);
        },
    );
    let hechos = hechos.into_inner().unwrap();
    let r = Resumen::de(&hechos);
    imprimir(&r, reloj.elapsed().as_secs_f64());
    if r.fallidas > 0 {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn imprimir(r: &Resumen, segundos: f64) {
    if r.por_formato.len() > 1 {
        for f in &r.por_formato {
            println!(
                "{}: {} {}, {} → {} ({})",
                f.formato.nombre(),
                f.ficheros,
                if f.ficheros == 1 {
                    trs!("fichero", "file")
                } else {
                    trs!("ficheros", "files")
                },
                bytes(f.bytes_entrada),
                bytes(f.bytes_salida),
                ahorro(f.bytes_entrada, f.bytes_salida)
            );
        }
    }
    println!(
        "{}",
        tr!(
            "{} convertidas en {} s: {} → {} ({})",
            "{} converted in {} s: {} → {} ({})",
            r.convertidas,
            decimal(format!("{segundos:.1}")),
            bytes(r.bytes_entrada),
            bytes(r.bytes_salida),
            ahorro(r.bytes_entrada, r.bytes_salida)
        )
    );
    match r.mayores {
        0 => {}
        1 => println!(
            "{}",
            trs!(
                "1 pesa más que su original (se ha guardado igual)",
                "1 weighs more than its original (saved anyway)"
            )
        ),
        n => println!(
            "{}",
            tr!(
                "{n} pesan más que su original (se han guardado igual)",
                "{n} weigh more than their originals (saved anyway)"
            )
        ),
    }
    match r.fallidas {
        0 => {}
        1 => println!(
            "{}",
            trs!("1 no se pudo convertir", "1 couldn't be converted")
        ),
        n => println!(
            "{}",
            tr!("{n} no se pudieron convertir", "{n} couldn't be converted")
        ),
    }
    if let Some(m) = r.nota_media {
        println!(
            "{}",
            tr!(
                "Nota media (SSIMULACRA 2): {}",
                "Average score (SSIMULACRA 2): {}",
                nota(m)
            )
        );
        println!("{}", trs!("Las de peor nota:", "Lowest scores:"));
        for d in &r.peores_notas {
            println!(
                "  {} ({})  {}",
                d.relativa.display(),
                d.formato.nombre(),
                nota(d.nota)
            );
        }
    }
    if r.convertidas > r.peores.len() {
        println!("{}", trs!("Las que menos ahorran:", "Least savings:"));
        for d in &r.peores {
            println!(
                "  {} ({})  {} → {}  {}",
                d.relativa.display(),
                d.formato.nombre(),
                bytes(d.bytes_entrada),
                bytes(d.bytes_salida),
                ahorro(d.bytes_entrada, d.bytes_salida)
            );
        }
    }
}

fn nota(n: f64) -> String {
    decimal(format!("{n:.1}"))
}
