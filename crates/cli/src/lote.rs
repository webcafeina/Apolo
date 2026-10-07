//! `apolo lote`: lo mismo que la sección Lotes de la ventana (ADR 0019).
//!
//! apolo lote fotos/ --salida fotos-webp --preset "Fotos web" -- -q 80
//!
//! Las opciones de cwebp van detrás de `--`, encima del preset si lo hay. Cada
//! línea de progreso va a la salida de errores; el resumen, a la normal.

use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::time::Instant;

use apolo_nucleo::lote::{self, Resumen};
use apolo_nucleo::webp::OpcionesWebp;
use apolo_nucleo::{cwebp, presets};

pub struct Peticion {
    pub entradas: Vec<PathBuf>,
    pub salida: Option<PathBuf>,
    pub preset: Option<String>,
    pub hilos: Option<usize>,
    pub cwebp: Vec<String>,
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

fn opciones(p: &Peticion) -> Result<OpcionesWebp, String> {
    let base = match &p.preset {
        None => OpcionesWebp::default(),
        Some(nombre) => {
            let carpeta =
                presets::carpeta().ok_or("No se encuentra la carpeta de configuración")?;
            presets::cargar(&carpeta, nombre)
                .ok_or_else(|| {
                    format!("No hay ningún preset «{nombre}» (apolo presets los lista)")
                })?
                .webp
        }
    };
    let o = cwebp::leer_desde(&base, &p.cwebp).map_err(|e| e.0)?;
    if o.entrada.is_some() || o.salida.is_some() {
        return Err(
            "Detrás de «--» van solo opciones de cwebp: las entradas van delante y la salida con --salida"
                .into(),
        );
    }
    if !o.opciones.validar() {
        return Err("La configuración no es válida".into());
    }
    Ok(o.opciones)
}

pub fn ejecutar(p: Peticion) -> ExitCode {
    let op = match opciones(&p) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("Error: {e}");
            return ExitCode::FAILURE;
        }
    };
    let Some(salida) = p
        .salida
        .clone()
        .or_else(|| lote::salida_sugerida(&p.entradas))
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
        let opciones = cwebp::escribir_apolo(&op);
        if opciones.is_empty() {
            eprintln!("Cada una como: cwebp (sin opciones)");
        } else {
            eprintln!("Cada una como: cwebp {}", opciones.join(" "));
        }
    }

    let reloj = Instant::now();
    let hechos = Mutex::new(Vec::new());
    lote::ejecutar(
        &elementos,
        &salida,
        &op,
        p.hilos.unwrap_or_else(lote::hilos_por_defecto),
        &AtomicBool::new(false),
        &|h| {
            let mut v = hechos.lock().unwrap();
            if !p.silencio {
                let n = v.len() + 1;
                match &h.resultado {
                    Ok(s) => eprintln!(
                        "[{n}/{total}] {}  {} → {}  {}",
                        h.relativa.display(),
                        bytes(h.bytes_entrada),
                        bytes(s.bytes),
                        ahorro(h.bytes_entrada, s.bytes)
                    ),
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
                "  {}  {} → {}  {}",
                d.relativa.display(),
                bytes(d.bytes_entrada),
                bytes(d.bytes_salida),
                ahorro(d.bytes_entrada, d.bytes_salida)
            );
        }
    }
}
