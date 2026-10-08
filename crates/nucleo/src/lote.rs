//! Lotes: muchas imágenes con las mismas opciones (ADR 0019).
//!
//! Cada imagen se codifica exactamente como `apolo webp` (`cwebp::ejecutar`),
//! así que la equivalencia byte a byte con cwebp se conserva imagen a imagen.
//! Lo propio del lote es lo que lo rodea:
//!
//! - **recoger** los ficheros de las carpetas soltadas, con sus subcarpetas;
//! - **no pisar nunca** un fichero: si el nombre existe, `foto-2.webp`, y el
//!   nombre se reserva al crear el fichero (`create_new`), para que dos hilos
//!   que llegan al mismo nombre a la vez no se lo quiten;
//! - **en paralelo**, con un hilo por núcleo, y **cancelable**: la imagen a
//!   medias se aborta (libwebp pregunta en cada paso) y las pendientes ya no
//!   empiezan;
//! - el **resumen**: cuánto se ahorró y cuáles salieron peor, también las que
//!   pesan más que el original, que se guardan igual (las pidió así quien las
//!   convierte) y se señalan.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Instant;

use serde::Serialize;

use crate::entrada::{self, Imagen, Lectura};
use crate::salida::{self, Ajuste, FormatoSalida};
use crate::{Error, Resultado};

/// Las extensiones que se recogen de una carpeta. Un fichero suelto se
/// intenta igual aunque no tenga una de estas: si no es una imagen, lo dirá.
pub const EXTENSIONES: &[&str] = &[
    "png", "jpg", "jpeg", "heic", "heif", "avif", "jxl", "webp", "tif", "tiff", "gif", "bmp",
    "qoi", "ppm", "pgm", "pam", "pnm",
];

/// Una imagen del lote.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Elemento {
    pub origen: PathBuf,
    /// Dónde va dentro de la carpeta de salida, con su extensión original.
    pub relativa: PathBuf,
    pub bytes: u64,
}

fn oculto(p: &Path) -> bool {
    // Los «._foto.jpg» que deja macOS en discos externos, `.DS_Store`, etc.
    p.file_name()
        .is_some_and(|n| n.to_string_lossy().starts_with('.'))
}

fn es_imagen(p: &Path) -> bool {
    p.extension().is_some_and(|e| {
        let e = e.to_string_lossy().to_lowercase();
        EXTENSIONES.contains(&e.as_str())
    })
}

fn bajar(carpeta: &Path, base: &Path, excluir: Option<&Path>, v: &mut Vec<Elemento>) {
    let Ok(lista) = std::fs::read_dir(carpeta) else {
        return;
    };
    for e in lista.flatten() {
        let ruta = e.path();
        if oculto(&ruta) || excluir.is_some_and(|x| ruta == x) {
            continue;
        }
        // file_type no sigue enlaces: una carpeta enlazada no entra, y así un
        // enlace que apunta hacia arriba no da vueltas para siempre.
        let Ok(tipo) = e.file_type() else { continue };
        if tipo.is_dir() {
            bajar(&ruta, base, excluir, v);
        } else if tipo.is_file() && es_imagen(&ruta) {
            let relativa = ruta.strip_prefix(base).unwrap_or(&ruta).to_path_buf();
            let bytes = e.metadata().map(|m| m.len()).unwrap_or(0);
            v.push(Elemento {
                origen: ruta,
                relativa,
                bytes,
            });
        }
    }
}

/// Las imágenes de lo que se ha soltado o elegido, ordenadas.
///
/// Con **una sola carpeta**, la estructura de salida es la de dentro de ella
/// (`vacaciones/playa/1.jpg` → `salida/playa/1.webp`). Con **varias cosas**,
/// cada carpeta conserva su nombre (`salida/vacaciones/playa/1.webp`) y los
/// ficheros sueltos van arriba: así dos carpetas con un `1.jpg` no chocan.
///
/// `excluir` es la carpeta de salida, por si está dentro de una de entrada:
/// sin eso, repetir el lote convertiría también lo ya convertido.
pub fn recoger(entradas: &[PathBuf], excluir: Option<&Path>) -> Vec<Elemento> {
    let excluir = excluir.map(|x| std::path::absolute(x).unwrap_or_else(|_| x.to_path_buf()));
    let excluir = excluir.as_deref();
    let mut v = Vec::new();
    let una_carpeta = entradas.len() == 1 && entradas[0].is_dir();
    for entrada in entradas {
        let entrada = std::path::absolute(entrada).unwrap_or_else(|_| entrada.clone());
        if entrada.is_dir() {
            let base = if una_carpeta {
                entrada.as_path()
            } else {
                entrada.parent().unwrap_or(&entrada)
            };
            bajar(&entrada, base, excluir, &mut v);
        } else if entrada.is_file() {
            let bytes = entrada.metadata().map(|m| m.len()).unwrap_or(0);
            v.push(Elemento {
                relativa: entrada.file_name().map(PathBuf::from).unwrap_or_default(),
                origen: entrada,
                bytes,
            });
        }
    }
    v.sort_by(|a, b| a.relativa.cmp(&b.relativa));
    v.dedup_by(|a, b| a.origen == b.origen);
    v
}

/// El sufijo de la carpeta que se propone: la extensión del formato si es
/// uno solo («-webp», «-jpg»…), y «-apolo» si son varios.
pub fn sufijo(ajustes: &[Ajuste]) -> String {
    let mut formatos: Vec<FormatoSalida> = ajustes.iter().map(|a| a.formato).collect();
    formatos.dedup();
    match formatos.as_slice() {
        [f] => f.extension().to_string(),
        _ => "apolo".to_string(),
    }
}

/// La carpeta de salida que se propone: junto a la carpeta de entrada (o a la
/// de los ficheros), con `-sufijo` detrás.
pub fn salida_sugerida(entradas: &[PathBuf], sufijo: &str) -> Option<PathBuf> {
    let primera = std::path::absolute(entradas.first()?).ok()?;
    let carpeta = if entradas.len() == 1 && primera.is_dir() {
        primera
    } else {
        primera.parent()?.to_path_buf()
    };
    let nombre = carpeta.file_name()?.to_string_lossy().into_owned();
    Some(carpeta.with_file_name(format!("{nombre}-{sufijo}")))
}

/// Escribe `datos` en `ruta` o, si ya existe, en `nombre-2.ext`, `nombre-3.ext`…
/// Devuelve dónde quedó. El nombre se reserva al crear el fichero, sin mirar
/// antes si existe: así no hay hueco entre mirar y escribir.
pub fn escribir_sin_pisar(ruta: &Path, datos: &[u8]) -> std::io::Result<PathBuf> {
    if let Some(d) = ruta.parent() {
        std::fs::create_dir_all(d)?;
    }
    let tronco = ruta
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let ext = ruta
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();
    for n in 1u32.. {
        let candidata = if n == 1 {
            ruta.to_path_buf()
        } else {
            ruta.with_file_name(format!("{tronco}-{n}{ext}"))
        };
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidata)
        {
            Ok(mut f) => {
                if let Err(e) = f.write_all(datos) {
                    // Un fichero a medias no se deja: parecería bueno.
                    drop(f);
                    let _ = std::fs::remove_file(&candidata);
                    return Err(e);
                }
                return Ok(candidata);
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e),
        }
    }
    unreachable!()
}

/// Cómo acabó una imagen.
#[derive(Debug, Clone, Serialize)]
pub struct Hecho {
    /// Su posición en la lista de `recoger`.
    pub indice: usize,
    pub relativa: PathBuf,
    pub bytes_entrada: u64,
    /// Los ficheros que dejó (uno por ajuste, o el más ligero); o por qué
    /// no se pudo.
    pub resultado: Result<Vec<Salida>, String>,
}

impl Hecho {
    /// El más ligero de los que dejó.
    pub fn principal(&self) -> Option<&Salida> {
        self.resultado.as_ref().ok()?.iter().min_by_key(|s| s.bytes)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Salida {
    pub formato: FormatoSalida,
    pub ruta: PathBuf,
    pub bytes: u64,
    pub milisegundos: u64,
    /// La nota SSIMULACRA 2, si se pidió medir o buscar una nota (ADR 0022).
    pub nota: Option<f64>,
    /// Con una nota objetivo, la calidad que se encontró para esta imagen.
    pub calidad: Option<u8>,
}

/// Lee una imagen para convertirla: con metadatos si se puede, y sin ellos
/// si están rotos (como el Estudio).
fn leer(datos: &[u8]) -> Resultado<Imagen> {
    entrada::leer(datos, Lectura::default()).or_else(|_| {
        entrada::leer(
            datos,
            Lectura {
                conservar_alfa: true,
                metadatos: false,
            },
        )
    })
}

/// Convierte una imagen con cada ajuste y escribe en `destino`, sin pisar
/// nada. Con `solo_mas_ligero`, escribe solo el que menos pese. Con `medir`,
/// mide la nota de lo que escribe (ADR 0022).
pub fn convertir(
    e: &Elemento,
    destino: &Path,
    ajustes: &[Ajuste],
    solo_mas_ligero: bool,
    medir: bool,
    seguir: &mut dyn FnMut(i32) -> bool,
) -> Resultado<Vec<Salida>> {
    let datos = std::fs::read(&e.origen)
        .map_err(|x| Error::Fichero(format!("No se puede leer «{}»: {x}", e.origen.display())))?;
    let img = leer(&datos)?;
    let mut hechos: Vec<(&Ajuste, salida::Codificado, u64)> = Vec::new();
    for a in ajustes {
        let reloj = Instant::now();
        let r = salida::codificar(&datos, &img, a, Some(&mut *seguir))?;
        hechos.push((a, r, reloj.elapsed().as_millis() as u64));
    }
    if solo_mas_ligero && let Some(i) = (0..hechos.len()).min_by_key(|&i| hechos[i].1.datos.len()) {
        let elegido = hechos.swap_remove(i);
        hechos = vec![elegido];
    }
    let mut salidas = Vec::new();
    for (a, r, ms) in hechos {
        // Solo se mide lo que se escribe; con nota objetivo, ya está medida.
        let nota = match r.hallada {
            Some(h) => Some(h.nota),
            None if medir => {
                if !seguir(0) {
                    return Err(Error::Cancelado);
                }
                let (w, h, referencia) = salida::referencia(&img, a)?;
                let (rw, rh, rgba) = crate::vista::decodificar(&r.datos)?;
                if (rw, rh) == (w, h) {
                    crate::medir::nota(&referencia, &rgba, w, h)?
                } else {
                    None
                }
            }
            None => None,
        };
        let (formato, bytes, calidad) = (a.formato, r.datos, r.hallada.map(|h| h.calidad));
        let ruta = destino
            .join(&e.relativa)
            .with_extension(formato.extension());
        let ruta = escribir_sin_pisar(&ruta, &bytes).map_err(|x| {
            Error::Fichero(format!("No se puede escribir «{}»: {x}", ruta.display()))
        })?;
        salidas.push(Salida {
            formato,
            ruta,
            bytes: bytes.len() as u64,
            milisegundos: ms,
            nota,
            calidad,
        });
    }
    Ok(salidas)
}

/// Cuántos hilos usar: uno por núcleo, como mucho ocho. Cada hilo tiene una
/// imagen entera en memoria (96 MB una foto de 24 megapíxeles).
pub fn hilos_por_defecto() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(2)
        .clamp(1, 8)
}

/// Convierte todo el lote en paralelo. `avisar` se llama con cada imagen al
/// terminar, en el orden en que terminan. Con `cancelado` puesto, la imagen a
/// medias se aborta y las que faltan no empiezan: no salen en `avisar`.
#[allow(clippy::too_many_arguments)]
pub fn ejecutar(
    elementos: &[Elemento],
    destino: &Path,
    ajustes: &[Ajuste],
    solo_mas_ligero: bool,
    medir: bool,
    hilos: usize,
    cancelado: &AtomicBool,
    avisar: &(dyn Fn(Hecho) + Sync),
) {
    let siguiente = AtomicUsize::new(0);
    std::thread::scope(|s| {
        for _ in 0..hilos.clamp(1, elementos.len().max(1)) {
            s.spawn(|| {
                loop {
                    if cancelado.load(Ordering::Relaxed) {
                        return;
                    }
                    let i = siguiente.fetch_add(1, Ordering::Relaxed);
                    let Some(e) = elementos.get(i) else { return };
                    let mut seguir = |_| !cancelado.load(Ordering::Relaxed);
                    let r = convertir(e, destino, ajustes, solo_mas_ligero, medir, &mut seguir);
                    if matches!(r, Err(Error::Cancelado)) {
                        return;
                    }
                    avisar(Hecho {
                        indice: i,
                        relativa: e.relativa.clone(),
                        bytes_entrada: e.bytes,
                        resultado: r.map_err(|x| x.to_string()),
                    });
                }
            });
        }
    });
}

/// Una imagen del resumen: de las que peor salieron.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Destacada {
    pub relativa: PathBuf,
    pub formato: FormatoSalida,
    pub bytes_entrada: u64,
    pub bytes_salida: u64,
}

/// Lo de un formato en el resumen.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PorFormato {
    pub formato: FormatoSalida,
    pub ficheros: usize,
    /// Lo que pesaban los originales de esos ficheros, y lo que pesan ellos.
    pub bytes_entrada: u64,
    pub bytes_salida: u64,
}

/// Lo que se enseña al acabar.
#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct Resumen {
    pub convertidas: usize,
    pub fallidas: usize,
    /// Las imágenes cuyo fichero más ligero pesa más que el original. Se
    /// guardan igual.
    pub mayores: usize,
    /// De las convertidas: lo que pesaban, y lo que pesa el más ligero de
    /// cada una.
    pub bytes_entrada: u64,
    pub bytes_salida: u64,
    /// Por formato, en el orden en que aparecen.
    pub por_formato: Vec<PorFormato>,
    /// Las que menos ahorran (o más crecen), de peor a mejor, por su fichero
    /// más ligero. Como mucho cinco.
    pub peores: Vec<Destacada>,
    /// Con notas (medir o nota objetivo): la media de los ficheros escritos y
    /// los de peor nota, de peor a mejor, como mucho cinco.
    pub nota_media: Option<f64>,
    pub peores_notas: Vec<ConNota>,
}

/// Un fichero del resumen por su nota.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ConNota {
    pub relativa: PathBuf,
    pub formato: FormatoSalida,
    pub nota: f64,
}

impl Resumen {
    pub fn de(hechos: &[Hecho]) -> Resumen {
        let mut r = Resumen::default();
        let mut ok = Vec::new();
        let mut notas: Vec<ConNota> = Vec::new();
        for h in hechos {
            let Ok(salidas) = &h.resultado else {
                r.fallidas += 1;
                continue;
            };
            for s in salidas {
                if let Some(nota) = s.nota {
                    notas.push(ConNota {
                        relativa: h.relativa.clone(),
                        formato: s.formato,
                        nota,
                    });
                }
                match r.por_formato.iter_mut().find(|p| p.formato == s.formato) {
                    Some(p) => {
                        p.ficheros += 1;
                        p.bytes_entrada += h.bytes_entrada;
                        p.bytes_salida += s.bytes;
                    }
                    None => r.por_formato.push(PorFormato {
                        formato: s.formato,
                        ficheros: 1,
                        bytes_entrada: h.bytes_entrada,
                        bytes_salida: s.bytes,
                    }),
                }
            }
            let Some(p) = h.principal() else { continue };
            r.convertidas += 1;
            r.bytes_entrada += h.bytes_entrada;
            r.bytes_salida += p.bytes;
            if p.bytes > h.bytes_entrada {
                r.mayores += 1;
            }
            ok.push(Destacada {
                relativa: h.relativa.clone(),
                formato: p.formato,
                bytes_entrada: h.bytes_entrada,
                bytes_salida: p.bytes,
            });
        }
        let proporcion = |d: &Destacada| d.bytes_salida as f64 / d.bytes_entrada.max(1) as f64;
        ok.sort_by(|a, b| proporcion(b).total_cmp(&proporcion(a)));
        ok.truncate(5);
        r.peores = ok;
        if !notas.is_empty() {
            r.nota_media = Some(notas.iter().map(|n| n.nota).sum::<f64>() / notas.len() as f64);
            notas.sort_by(|a, b| a.nota.total_cmp(&b.nota));
            notas.truncate(5);
            r.peores_notas = notas;
        }
        r
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::cwebp::{self, OrdenCwebp};
    use std::sync::Mutex;

    fn corpus(nombre: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../pruebas/corpus")
            .join(nombre)
    }

    fn carpeta(nombre: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("apolo-lote-{nombre}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    /// vacaciones/{a.webp, playa/b.png, playa/._b.png, notas.txt, .oculta/c.png}
    fn arbol(nombre: &str) -> PathBuf {
        let d = carpeta(nombre).join("vacaciones");
        std::fs::create_dir_all(d.join("playa")).unwrap();
        std::fs::create_dir_all(d.join(".oculta")).unwrap();
        std::fs::copy(corpus("foto.webp"), d.join("a.webp")).unwrap();
        std::fs::copy(corpus("orientacion-6.png"), d.join("playa/b.png")).unwrap();
        std::fs::write(d.join("playa/._b.png"), b"basura").unwrap();
        std::fs::write(d.join("notas.txt"), b"hola").unwrap();
        std::fs::copy(corpus("foto.webp"), d.join(".oculta/c.webp")).unwrap();
        d
    }

    fn relativas(v: &[Elemento]) -> Vec<String> {
        v.iter()
            .map(|e| e.relativa.to_string_lossy().replace('\\', "/"))
            .collect()
    }

    #[test]
    fn una_carpeta_se_recorre_por_dentro_sin_ocultos() {
        let d = arbol("una");
        assert_eq!(
            relativas(&recoger(std::slice::from_ref(&d), None)),
            ["a.webp", "playa/b.png"]
        );
        assert_eq!(
            salida_sugerida(std::slice::from_ref(&d), "webp").unwrap(),
            d.with_file_name("vacaciones-webp")
        );
    }

    #[test]
    fn varias_entradas_conservan_el_nombre_de_su_carpeta() {
        let d = arbol("varias");
        let suelta = d.parent().unwrap().join("suelta.png");
        std::fs::copy(corpus("orientacion-6.png"), &suelta).unwrap();
        assert_eq!(
            relativas(&recoger(&[d, suelta], None)),
            ["suelta.png", "vacaciones/a.webp", "vacaciones/playa/b.png"]
        );
    }

    #[test]
    fn la_salida_dentro_de_la_entrada_no_se_recoge() {
        let d = arbol("dentro");
        let salida = d.join("convertidas");
        std::fs::create_dir_all(&salida).unwrap();
        std::fs::copy(corpus("foto.webp"), salida.join("x.webp")).unwrap();
        assert_eq!(
            relativas(&recoger(&[d], Some(&salida))),
            ["a.webp", "playa/b.png"]
        );
    }

    #[test]
    fn nunca_pisa_un_fichero() {
        let d = carpeta("pisar");
        let r = d.join("sub/foto.webp");
        assert_eq!(escribir_sin_pisar(&r, b"1").unwrap(), r);
        assert_eq!(
            escribir_sin_pisar(&r, b"2").unwrap(),
            d.join("sub/foto-2.webp")
        );
        assert_eq!(
            escribir_sin_pisar(&r, b"3").unwrap(),
            d.join("sub/foto-3.webp")
        );
        assert_eq!(std::fs::read(&r).unwrap(), b"1");
    }

    #[test]
    fn el_lote_da_lo_mismo_que_apolo_webp_y_resume() {
        let d = arbol("lote");
        let salida = d.with_file_name("vacaciones-webp");
        let mut elementos = recoger(std::slice::from_ref(&d), Some(&salida));
        // Uno que no es una imagen aunque lo parezca: falla él solo.
        let rota = d.join("rota.png");
        std::fs::write(&rota, b"no soy un png").unwrap();
        elementos.push(Elemento {
            origen: rota,
            relativa: "rota.png".into(),
            bytes: 13,
        });
        let mut a = Ajuste::default();
        a.webp.calidad = 60.0;
        let op = a.webp.clone();
        let ajustes = [a];
        let hechos = Mutex::new(Vec::new());
        ejecutar(
            &elementos,
            &salida,
            &ajustes,
            false,
            false,
            3,
            &AtomicBool::new(false),
            &|h| hechos.lock().unwrap().push(h),
        );
        let hechos = hechos.into_inner().unwrap();
        assert_eq!(hechos.len(), 3);

        for e in &elementos[..2] {
            let esperado = cwebp::ejecutar(
                &OrdenCwebp {
                    opciones: op.clone(),
                    ..OrdenCwebp::default()
                },
                &std::fs::read(&e.origen).unwrap(),
                None,
            )
            .unwrap()
            .datos;
            let escrito = std::fs::read(salida.join(&e.relativa).with_extension("webp")).unwrap();
            assert_eq!(escrito, esperado, "{}", e.relativa.display());
        }

        let r = Resumen::de(&hechos);
        assert_eq!((r.convertidas, r.fallidas), (2, 1));
        assert_eq!(r.peores.len(), 2);
        let p = |d: &Destacada| d.bytes_salida as f64 / d.bytes_entrada as f64;
        assert!(p(&r.peores[0]) >= p(&r.peores[1]));

        // Repetirlo no pisa: salen con -2.
        let hechos2 = Mutex::new(Vec::new());
        ejecutar(
            &elementos[..1],
            &salida,
            &ajustes,
            false,
            false,
            1,
            &AtomicBool::new(false),
            &|h| hechos2.lock().unwrap().push(h),
        );
        let h = &hechos2.into_inner().unwrap()[0];
        assert_eq!(
            h.resultado.as_ref().unwrap()[0].ruta,
            salida.join("a-2.webp")
        );
    }

    #[test]
    fn varios_formatos_y_el_mas_ligero() {
        let d = arbol("formatos");
        let elementos = recoger(std::slice::from_ref(&d), None);
        let ajustes: Vec<Ajuste> = [FormatoSalida::Webp, FormatoSalida::Png, FormatoSalida::Qoi]
            .into_iter()
            .map(|formato| Ajuste {
                formato,
                ..Default::default()
            })
            .collect();
        // Todos: un fichero por formato y por imagen.
        let todos = d.with_file_name("formatos-todos");
        let hechos = Mutex::new(Vec::new());
        ejecutar(
            &elementos,
            &todos,
            &ajustes,
            false,
            false,
            2,
            &AtomicBool::new(false),
            &|h| hechos.lock().unwrap().push(h),
        );
        let hechos = hechos.into_inner().unwrap();
        assert!(
            hechos
                .iter()
                .all(|h| h.resultado.as_ref().unwrap().len() == 3)
        );
        assert!(
            todos.join("a.png").exists()
                && todos.join("a.qoi").exists()
                && todos.join("a.webp").exists()
        );
        let r = Resumen::de(&hechos);
        assert_eq!(r.por_formato.len(), 3);
        assert!(r.por_formato.iter().all(|p| p.ficheros == 2));
        // El más ligero: uno por imagen, y es el menor de los tres.
        let ligero = d.with_file_name("formatos-ligero");
        let hechos2 = Mutex::new(Vec::new());
        ejecutar(
            &elementos,
            &ligero,
            &ajustes,
            true,
            false,
            2,
            &AtomicBool::new(false),
            &|h| hechos2.lock().unwrap().push(h),
        );
        for h in hechos2.into_inner().unwrap() {
            let s = h.resultado.unwrap();
            assert_eq!(s.len(), 1);
            let otros = hechos
                .iter()
                .find(|x| x.relativa == h.relativa)
                .unwrap()
                .resultado
                .as_ref()
                .unwrap()
                .iter()
                .map(|x| x.bytes)
                .min()
                .unwrap();
            assert_eq!(s[0].bytes, otros);
        }
    }

    #[test]
    fn cancelar_a_medias_deja_de_empezar() {
        let d = carpeta("medias");
        for i in 0..40 {
            std::fs::copy(corpus("foto.webp"), d.join(format!("{i:02}.webp"))).unwrap();
        }
        let elementos = recoger(std::slice::from_ref(&d), None);
        assert_eq!(elementos.len(), 40);
        let cancelado = AtomicBool::new(false);
        let n = AtomicUsize::new(0);
        // Calidad lenta a propósito (-m 6), y se cancela con la primera hecha.
        let mut a = Ajuste::default();
        a.webp.metodo = 6;
        ejecutar(
            &elementos,
            &d.with_file_name("medias-salida"),
            &[a],
            false,
            false,
            2,
            &cancelado,
            &|_| {
                n.fetch_add(1, Ordering::SeqCst);
                cancelado.store(true, Ordering::SeqCst);
            },
        );
        let hechas = n.load(Ordering::SeqCst);
        assert!((1..=2).contains(&hechas), "{hechas}");
    }

    #[test]
    fn cancelado_no_empieza_nada() {
        let d = arbol("cancelar");
        let elementos = recoger(std::slice::from_ref(&d), None);
        let n = AtomicUsize::new(0);
        ejecutar(
            &elementos,
            &d.with_file_name("salida"),
            &[Ajuste::default()],
            false,
            false,
            2,
            &AtomicBool::new(true),
            &|_| {
                n.fetch_add(1, Ordering::Relaxed);
            },
        );
        assert_eq!(n.load(Ordering::Relaxed), 0);
    }
}
