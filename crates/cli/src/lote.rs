//! `apolo lote`: lo mismo que la sección Lotes de la ventana (ADR 0019).
//!
//! apolo lote fotos/ --salida fotos-web --preset "Fotos web" -- -q 80
//! apolo lote fotos/ --formato webp,jpeg --mas-ligero
//!
//! Un ajuste por cada `--preset` y cada formato de `--formato` (WebP si no hay
//! ninguno). Con uno solo, las opciones de su herramienta (cwebp, cjpeg u
//! oxipng) van detrás de `--`, encima del preset. Con `--mas-ligero`, de cada
//! imagen se guarda solo el formato que menos pese (ADR 0020). El progreso va
//! a la salida de errores; el resumen, a la normal.

use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::time::Instant;

use apolo_nucleo::formatos::png;
use apolo_nucleo::lote::{self, Resumen};
use apolo_nucleo::salida::{Ajuste, FormatoSalida};
use apolo_nucleo::{cwebp, jpeg, presets};

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
    /// Las opciones de la herramienta, detrás de `--`: solo con un formato.
    pub herramienta: Vec<String>,
    pub silencio: bool,
}

fn bytes(n: u64) -> String {
    match n {
        n if n < 1024 => format!("{n} B"),
        n if n < 1024 * 1024 => format!("{:.1} KB", n as f64 / 1024.0),
        n => format!("{:.1} MB", n as f64 / 1024.0 / 1024.0),
    }
    .replace('.', ",")
}

fn ahorro(antes: u64, despues: u64) -> String {
    if antes == 0 {
        return "—".into();
    }
    let p = (1.0 - despues as f64 / antes as f64) * 100.0;
    if p < 0.0 {
        format!("+{:.0} %", -p)
    } else {
        format!("−{p:.0} %")
    }
}

fn formato(s: &str) -> Result<FormatoSalida, String> {
    match s.trim().to_lowercase().as_str() {
        "webp" => Ok(FormatoSalida::Webp),
        "jpeg" | "jpg" | "mozjpeg" => Ok(FormatoSalida::Jpeg),
        "png" | "oxipng" => Ok(FormatoSalida::Png),
        "qoi" => Ok(FormatoSalida::Qoi),
        otro => Err(format!("no hay formato «{otro}»: webp, jpeg, png o qoi")),
    }
}

/// Los ajustes del lote: uno por preset y por formato pedido. Sin nada, WebP.
fn ajustes(p: &Peticion) -> Result<Vec<Ajuste>, String> {
    let mut v = Vec::new();
    if !p.presets.is_empty() {
        let carpeta = presets::carpeta().ok_or("No se encuentra la carpeta de configuración")?;
        for nombre in &p.presets {
            v.push(
                presets::cargar(&carpeta, nombre)
                    .ok_or_else(|| {
                        format!("No hay ningún preset «{nombre}» (apolo presets los lista)")
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
            return Err("Las opciones detrás de «--» van con un solo formato".into());
        };
        match a.formato {
            FormatoSalida::Webp => {
                let o = cwebp::leer_desde(&a.webp, &p.herramienta).map_err(|e| e.0)?;
                if o.entrada.is_some() || o.salida.is_some() {
                    return Err(SOLO_OPCIONES.into());
                }
                a.webp = o.opciones;
            }
            FormatoSalida::Jpeg => {
                a.jpeg = jpeg::OpcionesJpeg::leer_desde(&a.jpeg, &p.herramienta)?
            }
            FormatoSalida::Png => {
                let o = png::leer_orden_desde(&a.png, &p.herramienta).map_err(|e| e.to_string())?;
                if !o.entradas.is_empty() || o.salida.is_some() {
                    return Err(SOLO_OPCIONES.into());
                }
                a.png = o.opciones;
            }
            FormatoSalida::Qoi => return Err("QOI no tiene opciones".into()),
        }
    }
    if v.iter().any(|a| !a.webp.validar()) {
        return Err("La configuración no es válida".into());
    }
    Ok(v)
}

const SOLO_OPCIONES: &str =
    "Detrás de «--» van solo opciones: las entradas van delante y la salida con --salida";

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
        eprintln!("Error: falta --salida");
        return ExitCode::FAILURE;
    };
    let elementos = lote::recoger(&p.entradas, Some(&salida));
    if elementos.is_empty() {
        eprintln!("Error: no hay imágenes que convertir");
        return ExitCode::FAILURE;
    }
    let total = elementos.len();
    if !p.silencio {
        eprintln!(
            "{total} {} → {}",
            if total == 1 { "imagen" } else { "imágenes" },
            salida.display()
        );
        for a in &ajustes {
            let args = apolo_nucleo::salida::argumentos(a);
            let herramienta = a.formato.herramienta();
            if args.is_empty() {
                eprintln!("{}: como {herramienta} (sin opciones)", a.formato.nombre());
            } else {
                eprintln!(
                    "{}: como {herramienta} {}",
                    a.formato.nombre(),
                    args.join(" ")
                );
            }
        }
        if p.mas_ligero && ajustes.len() > 1 {
            eprintln!("De cada imagen se guarda solo el más ligero.");
        }
    }

    let reloj = Instant::now();
    let hechos = Mutex::new(Vec::new());
    lote::ejecutar(
        &elementos,
        &salida,
        &ajustes,
        p.mas_ligero,
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
                                format!(
                                    "{} {} {}",
                                    s.formato.nombre(),
                                    bytes(s.bytes),
                                    ahorro(h.bytes_entrada, s.bytes)
                                )
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
                    "fichero"
                } else {
                    "ficheros"
                },
                bytes(f.bytes_entrada),
                bytes(f.bytes_salida),
                ahorro(f.bytes_entrada, f.bytes_salida)
            );
        }
    }
    println!(
        "{} convertidas en {} s: {} → {} ({})",
        r.convertidas,
        format!("{segundos:.1}").replace('.', ","),
        bytes(r.bytes_entrada),
        bytes(r.bytes_salida),
        ahorro(r.bytes_entrada, r.bytes_salida)
    );
    match r.mayores {
        0 => {}
        1 => println!("1 pesa más que su original (se ha guardado igual)"),
        n => println!("{n} pesan más que su original (se han guardado igual)"),
    }
    match r.fallidas {
        0 => {}
        1 => println!("1 no se pudo convertir"),
        n => println!("{n} no se pudieron convertir"),
    }
    if r.convertidas > r.peores.len() {
        println!("Las que menos ahorran:");
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
