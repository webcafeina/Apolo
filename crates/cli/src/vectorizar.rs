//! `apolo vectorizar`: de un mapa de bits a SVG, PDF, EPS o DXF (ADR 0039). Cada opción está en
//! docs/cobertura-vectorizar.md.
//!
//! - Una imagen: `apolo vectorizar logo.png` deja `logo.svg` al lado (sin pisar: `logo-2.svg` si existe);
//!   con `-o logo.pdf`, el formato sale de la extensión y se escribe ahí.
//! - Varias o carpetas: a una carpeta (`-o carpeta`, o junto a la entrada con «-svg», «-pdf»…, como los
//!   lotes), con las subcarpetas repetidas y sin pisar nada.

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use apolo_nucleo::lote::{escribir_sin_pisar, recoger, salida_sugerida};
use apolo_nucleo::vector::{self, AjusteVector, Estilo, FormatoVector, NivelDxf, TipoImagen};
use apolo_nucleo::{tr, trs};

use crate::comun;

pub struct Peticion {
    pub entradas: Vec<PathBuf>,
    pub salida: Option<PathBuf>,
    pub formato: Option<String>,
    pub tipo: String,
    pub colores: Option<u32>,
    pub paleta: Option<String>,
    pub detalle: u8,
    pub objetivo: Option<f32>,
    pub sin_degradados: bool,
    pub area_minima: Option<f64>,
    pub estilo: String,
    pub grosor: f64,
    pub color_trazo: Option<String>,
    pub dxf: String,
    pub silencio: bool,
}

pub const TIPOS: [&str; 6] = [
    "auto",
    "logotipo",
    "ilustracion",
    "byn",
    "pixel-art",
    "foto",
];
pub const ESTILOS: [&str; 3] = ["rellenar", "trazar-formas", "trazar-bordes"];
pub const NIVELES_DXF: [&str; 3] = ["rectas", "arcos", "splines"];
pub const FORMATOS: [&str; 4] = ["svg", "pdf", "eps", "dxf"];

fn tipo(s: &str) -> TipoImagen {
    match s {
        "logotipo" => TipoImagen::Logotipo,
        "ilustracion" => TipoImagen::Ilustracion,
        "byn" => TipoImagen::ByN,
        "pixel-art" => TipoImagen::PixelArt,
        "foto" => TipoImagen::Foto,
        _ => TipoImagen::Auto,
    }
}

fn nombre_tipo(t: TipoImagen) -> &'static str {
    match t {
        TipoImagen::Auto => trs!("automático", "automatic"),
        TipoImagen::Logotipo => trs!("logotipo", "logo"),
        TipoImagen::Ilustracion => trs!("ilustración", "illustration"),
        TipoImagen::ByN => trs!("blanco y negro", "black and white"),
        TipoImagen::PixelArt => "pixel art",
        TipoImagen::Foto => trs!("foto", "photo"),
    }
}

fn ajuste(p: &Peticion) -> Result<AjusteVector, String> {
    let mut a = AjusteVector {
        tipo: tipo(&p.tipo),
        colores: p.colores,
        paleta: p
            .paleta
            .as_deref()
            .map(|t| {
                t.split(',')
                    .map(|c| c.trim().to_string())
                    .filter(|c| !c.is_empty())
                    .collect()
            })
            .unwrap_or_default(),
        detalle: p.detalle.min(100),
        objetivo: p.objetivo,
        degradados: !p.sin_degradados,
        estilo: match p.estilo.as_str() {
            "trazar-formas" => Estilo::TrazarFormas,
            "trazar-bordes" => Estilo::TrazarBordes,
            _ => Estilo::Rellenar,
        },
        grosor: p.grosor,
        color_trazo: p.color_trazo.clone(),
        nivel_dxf: match p.dxf.as_str() {
            "rectas" => NivelDxf::Rectas,
            "arcos" => NivelDxf::Arcos,
            _ => NivelDxf::Splines,
        },
        ..Default::default()
    };
    if let Some(m) = p.area_minima {
        a.area_minima = m;
    }
    if let Some(c) = p.colores
        && !(2..=256).contains(&c)
    {
        return Err(trs!("--colores va de 2 a 256", "--colores goes from 2 to 256").into());
    }
    if p.grosor.is_nan() || p.grosor <= 0.0 {
        return Err(trs!(
            "--grosor tiene que ser mayor que 0",
            "--grosor must be greater than 0"
        )
        .into());
    }
    if let Some(o) = p.objetivo
        && !(0.0..=100.0).contains(&o)
    {
        return Err(trs!("--objetivo va de 0 a 100", "--objetivo goes from 0 to 100").into());
    }
    // Los colores, comprobados ya: mejor fallar antes de empezar que en cada imagen.
    for c in a.paleta.iter().chain(a.color_trazo.iter()) {
        vector::color(c).map_err(|e| e.to_string())?;
    }
    Ok(a)
}

fn formato_de(nombre: &str) -> Option<FormatoVector> {
    FormatoVector::desde_extension(&format!("x.{nombre}"))
}

/// Dónde va cada imagen.
enum Destino {
    /// Un fichero, que se escribe aunque exista (lo ha pedido con `-o`).
    Fichero(PathBuf),
    /// Una carpeta: la ruta relativa de cada imagen dentro, sin pisar.
    Carpeta(PathBuf),
    /// Junto a cada imagen, sin pisar.
    Junto,
}

fn kb(n: usize) -> String {
    if n < 1024 * 1024 {
        comun::decimal(format!("{:.1} KB", n as f64 / 1024.0))
    } else {
        comun::decimal(format!("{:.2} MB", n as f64 / 1024.0 / 1024.0))
    }
}

pub fn ejecutar(p: Peticion) -> ExitCode {
    match correr(p) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::from(2)
        }
    }
}

/// `Ok(false)`: alguna imagen falló (las demás se hicieron).
fn correr(p: Peticion) -> Result<bool, String> {
    let a = ajuste(&p)?;
    let pedido = match &p.formato {
        Some(f) => Some(formato_de(f).ok_or_else(|| {
            tr!(
                "formato desconocido «{f}»: svg, pdf, eps o dxf",
                "unknown format “{f}”: svg, pdf, eps or dxf"
            )
        })?),
        None => None,
    };
    let carpetas = p.entradas.iter().any(|e| e.is_dir());
    let varias = p.entradas.len() > 1 || carpetas;
    // El formato del fichero de salida, si -o lo es.
    let de_salida = p
        .salida
        .as_ref()
        .and_then(|s| s.file_name())
        .and_then(|n| FormatoVector::desde_extension(&n.to_string_lossy()));
    let (destino, formato) = match (&p.salida, varias) {
        (Some(s), false) if de_salida.is_some() && !s.is_dir() => {
            (Destino::Fichero(s.clone()), pedido.or(de_salida).unwrap())
        }
        (Some(s), _) => (
            Destino::Carpeta(s.clone()),
            pedido.unwrap_or(FormatoVector::Svg),
        ),
        (None, true) => {
            let f = pedido.unwrap_or(FormatoVector::Svg);
            let s = salida_sugerida(&p.entradas, f.extension()).ok_or_else(|| {
                trs!(
                    "no se sabe dónde dejarlas: usa -o",
                    "can't tell where to put them: use -o"
                )
                .to_string()
            })?;
            (Destino::Carpeta(s), f)
        }
        (None, false) => (Destino::Junto, pedido.unwrap_or(FormatoVector::Svg)),
    };
    let excluir = match &destino {
        Destino::Carpeta(c) => Some(c.as_path()),
        _ => None,
    };
    let elementos = recoger(&p.entradas, excluir);
    if elementos.is_empty() {
        return Err(trs!(
            "no hay imágenes que vectorizar",
            "there are no images to vectorize"
        )
        .into());
    }
    let t0 = Instant::now();
    let mut fallos = 0;
    for e in &elementos {
        let r = (|| -> Result<(PathBuf, usize, vector::Vectorizado, f64), String> {
            let t = Instant::now();
            let datos = std::fs::read(&e.origen)
                .map_err(|err| tr!("no se puede leer: {err}", "can't read: {err}"))?;
            let img = comun::leer(&datos)?;
            let v = vector::vectorizar(&img, &a, None).map_err(|err| err.to_string())?;
            let bytes =
                vector::exportar(&v.documento, formato, &a).map_err(|err| err.to_string())?;
            let destino = match &destino {
                Destino::Fichero(f) => {
                    std::fs::write(f, &bytes).map_err(|err| {
                        tr!(
                            "no se puede escribir «{}»: {err}",
                            "can't write “{}”: {err}",
                            f.display()
                        )
                    })?;
                    f.clone()
                }
                Destino::Carpeta(c) => {
                    let r = c.join(&e.relativa).with_extension(formato.extension());
                    escribir_sin_pisar(&r, &bytes).map_err(|err| {
                        tr!(
                            "no se puede escribir «{}»: {err}",
                            "can't write “{}”: {err}",
                            r.display()
                        )
                    })?
                }
                Destino::Junto => {
                    let r = e.origen.with_extension(formato.extension());
                    escribir_sin_pisar(&r, &bytes).map_err(|err| {
                        tr!(
                            "no se puede escribir «{}»: {err}",
                            "can't write “{}”: {err}",
                            r.display()
                        )
                    })?
                }
            };
            Ok((destino, bytes.len(), v, t.elapsed().as_secs_f64()))
        })();
        match r {
            Ok((ruta, bytes, v, s)) => {
                if !p.silencio {
                    informar(&e.origen, &ruta, bytes, &v, s, a.objetivo);
                }
            }
            Err(err) => {
                fallos += 1;
                eprintln!("{}: {err}", e.origen.display());
            }
        }
    }
    if varias && !p.silencio {
        let hechas = elementos.len() - fallos;
        println!(
            "{}",
            comun::decimal(tr!(
                "{hechas} vectorizadas, {fallos} con error, en {:.1} s",
                "{hechas} vectorized, {fallos} with errors, in {:.1} s",
                t0.elapsed().as_secs_f64()
            ))
        );
    }
    Ok(fallos == 0)
}

fn informar(
    origen: &Path,
    ruta: &Path,
    bytes: usize,
    v: &vector::Vectorizado,
    segundos: f64,
    objetivo: Option<f32>,
) {
    let nombre = |r: &Path| {
        r.file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default()
    };
    let e = v.estadisticas;
    let nota = v
        .nota
        .map(|n| comun::decimal(tr!(" · nota {:.1}", " · score {:.1}", n)))
        .unwrap_or_default();
    println!(
        "{} → {} · {} · {}{nota} · {} {} · {} {} · {} {} · {}",
        nombre(origen),
        ruta.display(),
        kb(bytes),
        nombre_tipo(v.tipo),
        e.formas,
        trs!("formas", "shapes"),
        e.nodos,
        trs!("nodos", "nodes"),
        e.colores,
        trs!("colores", "colors"),
        comun::decimal(format!("{segundos:.1} s"))
    );
    if let (Some(h), Some(o)) = (v.hallado, objetivo) {
        let n = comun::decimal(format!("{:.1}", h.nota));
        let pruebas = if h.pruebas == 1 {
            trs!("1 prueba", "1 try").to_string()
        } else {
            tr!("{} pruebas", "{} tries", h.pruebas)
        };
        if h.alcanzada {
            println!(
                "  {}",
                tr!(
                    "objetivo {o}: detalle {} (nota {n}, {pruebas})",
                    "target {o}: detail {} (score {n}, {pruebas})",
                    h.detalle
                )
            );
        } else {
            println!(
                "  {}",
                tr!(
                    "objetivo {o}: no se llega; el de mejor nota es el detalle {} (nota {n}, {pruebas})",
                    "target {o}: not reached; the best score is at detail {} (score {n}, {pruebas})",
                    h.detalle
                )
            );
        }
    }
    if let Some((w, h)) = v.reducida_desde {
        println!(
            "  {}",
            tr!(
                "reducida de {w} × {h} a {} × {} antes de vectorizar (más de 2 Mpx)",
                "shrunk from {w} × {h} to {} × {} before vectorizing (over 2 Mpx)",
                v.documento.ancho,
                v.documento.alto
            )
        );
    }
    if v.tipo == TipoImagen::Foto {
        println!(
            "  {}",
            trs!(
                "es una foto: el resultado es un póster de colores planos, y suele pesar más que la foto",
                "it's a photo: the result is a poster of flat colors, and usually weighs more than the photo"
            )
        );
    }
}
