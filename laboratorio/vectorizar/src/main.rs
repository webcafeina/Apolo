//! Banco de pruebas para vectorizar en Apolo (fase 1, ADR 0038).
//!
//! `cargo run --release [-- filtro]`: prepara el corpus, pasa cada motor por cada imagen, mide la nota
//! SSIMULACRA 2 del SVG rasterizado frente al original y escribe `salida/resultados.csv` y la galería
//! `salida/index.html`. Los SVG de Vectorizer.ai, si están en `capturas/vectorizar/vectorizer-ai/<caso>.svg`,
//! entran como un motor más.

mod ajuste;
mod corpus;
mod imagen;
mod informe;
mod motores;
mod paleta;
mod poli;
mod potrace;
mod subpixel;

use corpus::{Caso, Origen};
use imagen::Rgba;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Instant;

#[derive(Clone, Debug)]
pub struct Fila {
    pub caso: String,
    pub grupo: corpus::Grupo,
    pub motor: String,
    pub nota: Option<f64>,
    pub bytes: usize,
    pub trazados: usize,
    pub figuras: usize,
    pub nodos: usize,
    pub colores: usize,
    pub ms: u128,
    pub error: Option<String>,
}

/// Cuenta lo que hay dentro de un SVG: trazados, figuras básicas, nodos y colores de relleno.
fn contar(svg: &str) -> (usize, usize, usize, usize) {
    let trazados = svg.matches("<path").count();
    let figuras = ["<circle", "<ellipse", "<rect", "<polygon", "<line", "<polyline"].iter().map(|f| svg.matches(f).count()).sum();
    let mut nodos = 0;
    let mut resto = svg;
    while let Some(p) = resto.find(" d=\"") {
        let d = &resto[p + 4..];
        let fin = d.find('"').unwrap_or(d.len());
        nodos += d[..fin].chars().filter(|c| "MLCQAHVSTmlcqahvst".contains(*c)).count();
        resto = &d[fin..];
    }
    let mut colores = std::collections::HashSet::new();
    for clave in ["fill=\"#", "fill:#"] {
        let mut r = svg;
        while let Some(p) = r.find(clave) {
            let c: String = r[p + clave.len()..].chars().take_while(|c| c.is_ascii_hexdigit()).collect();
            colores.insert(c.to_lowercase());
            r = &r[p + clave.len()..];
        }
    }
    (trazados, figuras, nodos, colores.len())
}

fn preparar(c: &Caso) -> Result<(Rgba, bool), String> {
    match &c.origen {
        Origen::Svg { ruta, lado, suavizado } => {
            let datos = std::fs::read(ruta).map_err(|e| format!("{}: {e}", ruta.display()))?;
            let (w, h) = imagen::tamano_svg(&datos, *lado)?;
            Ok((imagen::rasterizar(&datos, w, h, *suavizado)?, *suavizado))
        }
        Origen::Pixel { ruta, celda, escala } => {
            let datos = std::fs::read(ruta).map_err(|e| format!("{}: {e}", ruta.display()))?;
            let base = imagen::rasterizar(&datos, *celda, *celda, false)?;
            Ok((imagen::ampliar_vecino(&base, *escala), false))
        }
        Origen::Foto { ruta, lado_max } => Ok((imagen::foto(ruta, *lado_max)?, true)),
    }
}

fn medir_svg(apolo: &Path, dir: &Path, nombre: &str, svg: &str, original: &Rgba, ms: u128, c: &Caso) -> Fila {
    let (trazados, figuras, nodos, colores) = contar(svg);
    let mut f = Fila { caso: c.id.clone(), grupo: c.grupo, motor: nombre.into(), nota: None, bytes: svg.len(), trazados, figuras, nodos, colores, ms, error: None };
    let r = (|| -> Result<f64, String> {
        std::fs::write(dir.join(format!("{nombre}.svg")), svg).map_err(|e| e.to_string())?;
        let raster = imagen::rasterizar(svg.as_bytes(), original.ancho, original.alto, true)?;
        let png = dir.join(format!("{nombre}.png"));
        raster.guardar_png(&png)?;
        imagen::nota(apolo, &dir.join("original.png"), &png)
    })();
    match r {
        Ok(n) => f.nota = Some(n),
        Err(e) => f.error = Some(e),
    }
    f
}

fn correr_caso(c: &Caso, apolo: &Path, salida: &Path, vectorizer: &Path) -> Vec<Fila> {
    let dir = salida.join(&c.id);
    let _ = std::fs::create_dir_all(&dir);
    let mut filas = Vec::new();
    let original = match preparar(c) {
        Ok((o, _)) => o,
        Err(e) => {
            filas.push(Fila { caso: c.id.clone(), grupo: c.grupo, motor: "original".into(), nota: None, bytes: 0, trazados: 0, figuras: 0, nodos: 0, colores: 0, ms: 0, error: Some(e) });
            return filas;
        }
    };
    if let Err(e) = original.guardar_png(&dir.join("original.png")) {
        eprintln!("{}: {e}", c.id);
        return filas;
    }
    if let Some(r) = c.referencia() {
        if let Ok(svg) = std::fs::read_to_string(r) {
            filas.push(medir_svg(apolo, &dir, "referencia", &svg, &original, 0, c));
        }
    }
    for m in motores::MOTORES.iter().filter(|m| m.aplica(c.grupo)) {
        let t = Instant::now();
        let r = (m.correr)(&original, c.grupo);
        let ms = t.elapsed().as_millis();
        match r {
            Ok(svg) => filas.push(medir_svg(apolo, &dir, m.id, &svg, &original, ms, c)),
            Err(e) => filas.push(Fila { caso: c.id.clone(), grupo: c.grupo, motor: m.id.into(), nota: None, bytes: 0, trazados: 0, figuras: 0, nodos: 0, colores: 0, ms, error: Some(e) }),
        }
    }
    let externo = vectorizer.join(format!("{}.svg", c.id));
    if let Ok(svg) = std::fs::read_to_string(&externo) {
        filas.push(medir_svg(apolo, &dir, "vectorizer-ai", &svg, &original, 0, c));
    }
    filas
}

fn main() {
    let raiz = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo = raiz.join("../..");
    let apolo = std::env::var_os("APOLO").map(PathBuf::from).unwrap_or_else(|| repo.join("target/release/apolo"));
    let vectorizer = repo.join("capturas/vectorizar/vectorizer-ai");
    let salida = raiz.join("salida");
    let filtro = std::env::args().nth(1).unwrap_or_default();
    if !apolo.exists() {
        eprintln!("Falta {}: `make cli` en la raíz, o APOLO=ruta.", apolo.display());
        std::process::exit(1);
    }
    std::fs::create_dir_all(&salida).expect("salida");
    let casos: Vec<Caso> = corpus::casos(&raiz).into_iter().filter(|c| c.id.contains(&filtro)).collect();
    let filas = Mutex::new(Vec::new());
    let siguiente = Mutex::new(0usize);
    let hilos = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).min(8);
    let t = Instant::now();
    std::thread::scope(|s| {
        for _ in 0..hilos {
            s.spawn(|| loop {
                let i = {
                    let mut g = siguiente.lock().unwrap();
                    let i = *g;
                    *g += 1;
                    i
                };
                let Some(c) = casos.get(i) else { break };
                let f = correr_caso(c, &apolo, &salida, &vectorizer);
                eprintln!("{:>3}/{} {}", i + 1, casos.len(), c.id);
                filas.lock().unwrap().extend(f);
            });
        }
    });
    let mut filas = filas.into_inner().unwrap();
    let orden = |id: &str| casos.iter().position(|c| c.id == id).unwrap_or(usize::MAX);
    filas.sort_by(|a, b| orden(&a.caso).cmp(&orden(&b.caso)).then(a.motor.cmp(&b.motor)));
    informe::csv(&salida.join("resultados.csv"), &filas).expect("csv");
    informe::galeria(&salida.join("index.html"), &casos, &filas).expect("galería");
    informe::galeria_artefacto(&salida.join("galeria-artefacto.html"), &salida, &casos, &filas).expect("galería del artefacto");
    let errores = filas.iter().filter(|f| f.error.is_some()).count();
    eprintln!("{} casos, {} filas, {} errores, {:.0} s → {}", casos.len(), filas.len(), errores, t.elapsed().as_secs_f64(), salida.display());
    for f in filas.iter().filter(|f| f.error.is_some()) {
        eprintln!("  error {} {}: {}", f.caso, f.motor, f.error.as_deref().unwrap_or(""));
    }
}
