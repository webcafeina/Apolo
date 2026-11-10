//! Banco de pruebas para vectorizar en Apolo (fase 1, ADR 0038).
//!
//! `cargo run --release [-- filtro]`: prepara el corpus, pasa cada motor por cada imagen, mide la nota
//! SSIMULACRA 2 del SVG rasterizado frente al original y escribe `salida/resultados.csv` y la galería
//! `salida/index.html`. Los SVG de Vectorizer.ai, si están en `capturas/vectorizar/vectorizer-ai/<caso>.svg`,
//! entran como un motor más.

mod ajuste;
mod corpus;
mod degradado;
mod imagen;
mod informe;
mod motores;
mod paleta;
mod poli;
mod sintesis;
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
    // APOLO_MOTORES=a,b: solo esos motores (además de la referencia y Vectorizer.ai).
    let solo: Option<Vec<String>> = std::env::var("APOLO_MOTORES").ok().map(|v| v.split(',').map(str::to_string).collect());
    for m in motores::MOTORES.iter().filter(|m| m.aplica(c.grupo)).filter(|m| solo.as_ref().map(|s| s.iter().any(|x| x == m.id)).unwrap_or(true)) {
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
    if std::env::var("APOLO_TIPOS").is_ok() {
        for c in corpus::casos(&raiz).into_iter().filter(|c| c.id.contains(&filtro)) {
            if let Ok((o, _)) = preparar(&c) {
                let im = apolo_vector::Rgba::nueva(o.ancho, o.alto, o.px.clone());
                println!("{:24} {:14} → {:?}", c.id, format!("{:?}", c.grupo), apolo_vector::tipo_efectivo(&im, apolo_vector::TipoImagen::Auto));
            }
        }
        return;
    }
    if std::env::var("APOLO_FORMATOS").is_ok() {
        formatos(&raiz, &salida, &filtro);
        return;
    }
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

/// APOLO_FORMATOS=1: cada imagen en los cuatro formatos (`salida/formatos/`), con su peso y su tiempo, y el PDF
/// rasterizado con pdftoppm comparado con el SVG (diferencia media por canal, de 0 a 255).
fn formatos(raiz: &Path, salida: &Path, filtro: &str) {
    use apolo_vector::{Estilo, FormatoVector, NivelDxf, OpcionesExportar, exportar};
    let dir = salida.join("formatos");
    std::fs::create_dir_all(&dir).expect("formatos");
    println!("{:24} {:>9} {:>9} {:>9} {:>9} {:>9} {:>9} {:>6} {:>8}", "caso", "svg", "pdf", "eps", "dxf", "dxf-arcos", "dxf-rect", "ms", "pdf≠svg");
    for c in corpus::casos(raiz).into_iter().filter(|c| c.id.contains(filtro)) {
        let Ok((o, _)) = preparar(&c) else { continue };
        let im = apolo_vector::Rgba::nueva(o.ancho, o.alto, o.px.clone());
        let Ok(doc) = apolo_vector::vectorizar(&im, &Default::default()) else { continue };
        let t = Instant::now();
        let mut pesos = Vec::new();
        for (nombre, f, op) in [
            ("svg", FormatoVector::Svg, OpcionesExportar::default()),
            ("pdf", FormatoVector::Pdf, OpcionesExportar::default()),
            ("eps", FormatoVector::Eps, OpcionesExportar::default()),
            ("dxf", FormatoVector::Dxf, OpcionesExportar::default()),
            ("arcos.dxf", FormatoVector::Dxf, OpcionesExportar { nivel_dxf: NivelDxf::Arcos, ..Default::default() }),
            ("rectas.dxf", FormatoVector::Dxf, OpcionesExportar { nivel_dxf: NivelDxf::Rectas, ..Default::default() }),
            ("bordes.svg", FormatoVector::Svg, OpcionesExportar { estilo: Estilo::TrazarBordes, ..Default::default() }),
            ("bordes.dxf", FormatoVector::Dxf, OpcionesExportar { estilo: Estilo::TrazarBordes, ..Default::default() }),
            ("bordes.rectas.dxf", FormatoVector::Dxf, OpcionesExportar { estilo: Estilo::TrazarBordes, nivel_dxf: NivelDxf::Rectas, ..Default::default() }),
        ] {
            let b = exportar(&doc, f, &op);
            pesos.push(b.len());
            std::fs::write(dir.join(format!("{}.{nombre}", c.id)), &b).expect("escribir");
        }
        let ms = t.elapsed().as_millis();
        // Los bordes, a doble tamaño, para mirarlos.
        if let Ok(b) = std::fs::read(dir.join(format!("{}.bordes.svg", c.id))) {
            if let Ok(r) = imagen::rasterizar(&b, o.ancho * 2, o.alto * 2, true) {
                let _ = r.guardar_png(&dir.join(format!("{}.bordes.png", c.id)));
            }
        }
        let pdf = dir.join(format!("{}.pdf", c.id));
        let png = dir.join(format!("{}-pdf", c.id));
        let dif = std::process::Command::new("pdftoppm").args(["-r", "72", "-singlefile", "-png"]).arg(&pdf).arg(&png).status().ok().filter(|s| s.success()).and_then(|_| {
            let a = imagen::rasterizar(&std::fs::read(dir.join(format!("{}.svg", c.id))).ok()?, o.ancho, o.alto, true).ok()?;
            let b = imagen::foto(&dir.join(format!("{}-pdf.png", c.id)), u32::MAX).ok()?;
            let n = a.px.len().min(b.px.len());
            Some(a.px[..n].iter().zip(&b.px[..n]).enumerate().filter(|(i, _)| i % 4 != 3).map(|(_, (x, y))| (*x as i32 - *y as i32).unsigned_abs() as f64).sum::<f64>() / (n as f64 * 0.75))
        });
        let kb = |b: usize| format!("{:.1}K", b as f64 / 1024.0);
        println!("{:24} {:>9} {:>9} {:>9} {:>9} {:>9} {:>9} {:>6} {:>8}", c.id, kb(pesos[0]), kb(pesos[1]), kb(pesos[2]), kb(pesos[3]), kb(pesos[4]), kb(pesos[5]), ms, dif.map(|d| format!("{d:.2}")).unwrap_or("-".into()));
    }
}
