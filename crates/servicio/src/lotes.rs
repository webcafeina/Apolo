//! Lotes desde la interfaz (ADR 0019): empezar, preguntar cómo va y cancelar.
//!
//! El trabajo lo hace `apolo_nucleo::lote` en un hilo aparte. La interfaz
//! pregunta cada poco con `estado_lote(id, desde)` y recibe solo lo nuevo:
//! así funciona igual por Tauri que por HTTP, sin eventos que solo tenga uno
//! de los dos (ADR 0013).

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use apolo_nucleo::lote::{self, Hecho, Resumen};
use apolo_nucleo::salida::{Ajuste, FormatoSalida};
use serde::Serialize;

use crate::{Fallo, R, Servicio};

/// Lo que hay en lo soltado, antes de convertir.
#[derive(Debug, Clone, Serialize)]
pub struct Recogida {
    pub imagenes: usize,
    pub bytes: u64,
    /// La carpeta que se propone, sin el sufijo del formato: la interfaz le
    /// añade «-webp», «-jpg»… o «-apolo» según lo que se pida.
    pub salida_sugerida: Option<String>,
    /// Las primeras, para enseñarlas.
    pub muestra: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LoteEmpezado {
    pub id: u64,
    pub total: usize,
    pub salida: String,
}

/// Una imagen terminada, para la lista de la interfaz.
#[derive(Debug, Clone, Serialize)]
pub struct Fila {
    pub relativa: String,
    pub bytes_entrada: u64,
    /// Los ficheros que dejó, el más ligero primero.
    pub salidas: Vec<FilaSalida>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FilaSalida {
    pub formato: FormatoSalida,
    pub bytes: u64,
    pub ruta: String,
}

impl From<&Hecho> for Fila {
    fn from(h: &Hecho) -> Self {
        let mut salidas: Vec<FilaSalida> = h
            .resultado
            .as_ref()
            .map(|v| {
                v.iter()
                    .map(|s| FilaSalida {
                        formato: s.formato,
                        bytes: s.bytes,
                        ruta: s.ruta.display().to_string(),
                    })
                    .collect()
            })
            .unwrap_or_default();
        salidas.sort_by_key(|s| s.bytes);
        Fila {
            relativa: h.relativa.display().to_string(),
            bytes_entrada: h.bytes_entrada,
            salidas,
            error: h.resultado.as_ref().err().cloned(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EstadoLote {
    pub total: usize,
    pub hechas: usize,
    /// Las terminadas desde `desde`, en el orden en que terminaron.
    pub nuevas: Vec<Fila>,
    pub terminado: bool,
    pub cancelado: bool,
    pub segundos: f64,
    /// Al terminar (también si se canceló): el resumen de lo hecho.
    pub resumen: Option<Resumen>,
}

pub(crate) struct Lote {
    total: usize,
    hechos: Mutex<Vec<Hecho>>,
    cancelado: AtomicBool,
    terminado: AtomicBool,
    empezado: Instant,
    acabado: Mutex<Option<f64>>,
}

fn rutas(entradas: &[String]) -> Vec<PathBuf> {
    entradas.iter().map(PathBuf::from).collect()
}

/// Cuenta lo que hay en lo soltado o elegido, y propone dónde dejarlo.
pub fn recoger_lote(entradas: &[String]) -> Recogida {
    let rutas = rutas(entradas);
    let v = lote::recoger(&rutas, None);
    Recogida {
        imagenes: v.len(),
        bytes: v.iter().map(|e| e.bytes).sum(),
        salida_sugerida: lote::salida_sugerida(&rutas, "")
            .map(|p| p.display().to_string().trim_end_matches('-').to_string()),
        muestra: v
            .iter()
            .take(8)
            .map(|e| e.relativa.display().to_string())
            .collect(),
    }
}

impl Servicio {
    /// Empieza a convertir en otro hilo y vuelve en seguida.
    /// Con `solo_mas_ligero`, de cada imagen se guarda solo el formato que
    /// menos pese; si no, uno por ajuste.
    pub fn empezar_lote(
        &self,
        entradas: &[String],
        ajustes: &[Ajuste],
        solo_mas_ligero: bool,
        salida: &str,
    ) -> R<LoteEmpezado> {
        if ajustes.is_empty() {
            return Err(Fallo::nuevo("Falta al menos un formato de salida"));
        }
        if ajustes.iter().any(|a| !a.webp.validar()) {
            return Err(Fallo::nuevo("La configuración no es válida"));
        }
        if salida.trim().is_empty() {
            return Err(Fallo::nuevo("Falta la carpeta de salida"));
        }
        let destino = std::path::absolute(salida).unwrap_or_else(|_| PathBuf::from(salida));
        let elementos = lote::recoger(&rutas(entradas), Some(&destino));
        if elementos.is_empty() {
            return Err(Fallo::nuevo("No hay imágenes que convertir"));
        }
        let id = self.siguiente.fetch_add(1, Ordering::Relaxed);
        let l = Arc::new(Lote {
            total: elementos.len(),
            hechos: Mutex::default(),
            cancelado: AtomicBool::new(false),
            terminado: AtomicBool::new(false),
            empezado: Instant::now(),
            acabado: Mutex::default(),
        });
        self.lotes.lock().unwrap().insert(id, l.clone());
        let ajustes = ajustes.to_vec();
        let d = destino.clone();
        std::thread::spawn(move || {
            lote::ejecutar(
                &elementos,
                &d,
                &ajustes,
                solo_mas_ligero,
                lote::hilos_por_defecto(),
                &l.cancelado,
                &|h| l.hechos.lock().unwrap().push(h),
            );
            *l.acabado.lock().unwrap() = Some(l.empezado.elapsed().as_secs_f64());
            l.terminado.store(true, Ordering::SeqCst);
        });
        Ok(LoteEmpezado {
            id,
            total: self.lotes.lock().unwrap()[&id].total,
            salida: destino.display().to_string(),
        })
    }

    pub fn estado_lote(&self, id: u64, desde: usize) -> R<EstadoLote> {
        let l = self
            .lotes
            .lock()
            .unwrap()
            .get(&id)
            .cloned()
            .ok_or_else(|| Fallo::nuevo("Ese lote no existe"))?;
        // Se lee antes que la lista: si ya estaba terminado, la lista está entera.
        let terminado = l.terminado.load(Ordering::SeqCst);
        let hechos = l.hechos.lock().unwrap();
        Ok(EstadoLote {
            total: l.total,
            hechas: hechos.len(),
            nuevas: hechos.iter().skip(desde).map(Fila::from).collect(),
            terminado,
            cancelado: l.cancelado.load(Ordering::SeqCst),
            segundos: l
                .acabado
                .lock()
                .unwrap()
                .unwrap_or_else(|| l.empezado.elapsed().as_secs_f64()),
            resumen: terminado.then(|| Resumen::de(&hechos)),
        })
    }

    /// Para el lote: la imagen a medias se aborta y las pendientes no empiezan.
    pub fn cancelar_lote(&self, id: u64) {
        if let Some(l) = self.lotes.lock().unwrap().get(&id) {
            l.cancelado.store(true, Ordering::SeqCst);
        }
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn un_lote_entero_por_el_servicio() {
        let base = std::env::temp_dir().join(format!("apolo-servicio-lote-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let entrada = base.join("fotos");
        std::fs::create_dir_all(&entrada).unwrap();
        let corpus = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../pruebas/corpus");
        std::fs::copy(corpus.join("foto.webp"), entrada.join("a.webp")).unwrap();
        std::fs::copy(corpus.join("orientacion-6.png"), entrada.join("b.png")).unwrap();
        let e = vec![entrada.display().to_string()];

        let r = recoger_lote(&e);
        assert_eq!(r.imagenes, 2);
        let salida = format!("{}-webp", r.salida_sugerida.unwrap());
        assert!(salida.ends_with("fotos-webp"));

        let s = Servicio::nuevo(base.join("config"));
        let l = s
            .empezar_lote(&e, &[Ajuste::default()], false, &salida)
            .unwrap();
        assert_eq!(l.total, 2);
        let mut vistas = 0;
        let fin = loop {
            let st = s.estado_lote(l.id, vistas).unwrap();
            vistas += st.nuevas.len();
            if st.terminado {
                break st;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        };
        assert_eq!(vistas, 2);
        assert_eq!(fin.resumen.unwrap().convertidas, 2);
        assert!(std::path::Path::new(&salida).join("b.webp").exists());
    }
}
