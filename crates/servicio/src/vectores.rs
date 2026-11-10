//! Vectorizar en el Estudio (ADR 0039): una imagen abierta se vectoriza, se enseña su vista previa en el
//! comparador, se mide y se exporta a SVG, PDF, EPS o DXF.
//!
//! Va aparte del comparador de formatos (no es un lado más): guarda el último dibujo de cada imagen, y
//! exportar con otras opciones de dibujo (estilo, grosor, nivel de DXF) no vuelve a vectorizar.

use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use apolo_nucleo::medir::{self, Medidas, TipoMapa};
use apolo_nucleo::vector::{self, AjusteVector, FormatoVector, Vectorizado};
use apolo_nucleo::{Error, tr, trs};
use serde::Serialize;

use crate::{Fallo, R, Servicio};

/// El último dibujo de una imagen.
#[derive(Default)]
pub(crate) struct UltimoVector {
    ajuste: Option<AjusteVector>,
    hecho: Option<Arc<Vectorizado>>,
}

/// Lo que la interfaz sabe del dibujo.
#[derive(Debug, Clone, Serialize)]
pub struct VistaVector {
    pub generacion: u64,
    /// El tamaño del dibujo (el de la vista previa).
    pub ancho: u32,
    pub alto: u32,
    /// Si la imagen se redujo antes de vectorizar, su tamaño de antes.
    pub reducida_desde: Option<(u32, u32)>,
    /// El tipo de imagen que se usó (resuelto si era automático).
    pub tipo: vector::TipoImagen,
    pub detalle: u8,
    pub nota: Option<f64>,
    pub hallado: Option<vector::HalladoVector>,
    pub estadisticas: vector::Estadisticas,
    /// Lo que pesa en SVG relleno (la referencia de peso).
    pub bytes_svg: usize,
    pub milisegundos: u64,
}

impl Servicio {
    fn ultimo_vector(&self, id: u64) -> Arc<Mutex<UltimoVector>> {
        self.vectores.lock().unwrap().entry(id).or_default().clone()
    }

    fn generacion_vector(&self, id: u64) -> Arc<AtomicU64> {
        self.generaciones_vector
            .lock()
            .unwrap()
            .entry(id)
            .or_default()
            .clone()
    }

    fn hecho(&self, id: u64) -> R<Arc<Vectorizado>> {
        self.ultimo_vector(id)
            .lock()
            .unwrap()
            .hecho
            .clone()
            .ok_or_else(|| Fallo::nuevo(trs!("Todavía no hay dibujo", "There's no drawing yet")))
    }

    /// Vectoriza una imagen abierta. `generacion` crece con cada cambio; lo que se esté vectorizando con
    /// una anterior se cancela entre prueba y prueba (al buscar una nota).
    pub fn vectorizar(&self, id: u64, ajuste: &AjusteVector, generacion: u64) -> R<VistaVector> {
        self.vectorizar_con(id, ajuste, generacion, &|_| {})
    }

    /// Como [`Servicio::vectorizar`], y `avisar` recibe el progreso (de 0 a 100).
    pub fn vectorizar_con(
        &self,
        id: u64,
        ajuste: &AjusteVector,
        generacion: u64,
        avisar: &(dyn Fn(i32) + Sync),
    ) -> R<VistaVector> {
        let a = self.abierta(id)?;
        let g = self.generacion_vector(id);
        g.fetch_max(generacion, Ordering::SeqCst);
        let t = Instant::now();
        let mut progreso = |p: i32| {
            avisar(p);
            g.load(Ordering::SeqCst) == generacion
        };
        let hecho = Arc::new(vector::vectorizar(&a.imagen, ajuste, Some(&mut progreso))?);
        if g.load(Ordering::SeqCst) != generacion {
            return Err(Error::Cancelado.into());
        }
        let bytes_svg = vector::exportar(
            &hecho.documento,
            FormatoVector::Svg,
            &AjusteVector::default(),
        )?
        .len();
        let vista = VistaVector {
            generacion,
            ancho: hecho.documento.ancho,
            alto: hecho.documento.alto,
            reducida_desde: hecho.reducida_desde,
            tipo: hecho.tipo,
            detalle: hecho.detalle,
            nota: hecho.nota,
            hallado: hecho.hallado,
            estadisticas: hecho.estadisticas,
            bytes_svg,
            milisegundos: t.elapsed().as_millis() as u64,
        };
        let u = self.ultimo_vector(id);
        let mut u = u.lock().unwrap();
        u.ajuste = Some(ajuste.clone());
        u.hecho = Some(hecho);
        Ok(vista)
    }

    /// Cancela lo que se esté vectorizando de esa imagen.
    pub fn cancelar_vector(&self, id: u64) {
        self.generacion_vector(id).fetch_add(1, Ordering::SeqCst);
    }

    /// La vista previa del último dibujo.
    pub fn pixeles_vector(&self, id: u64) -> R<(u32, u32, Vec<u8>)> {
        let h = self.hecho(id)?;
        Ok((h.documento.ancho, h.documento.alto, h.vista.clone()))
    }

    /// Lo que recibió el motor (enderezado, reducido y sobre blanco), del tamaño de la vista previa: el otro
    /// lado del comparador.
    pub fn pixeles_referencia_vector(&self, id: u64) -> R<(u32, u32, Vec<u8>)> {
        let h = self.hecho(id)?;
        Ok((h.documento.ancho, h.documento.alto, h.referencia.clone()))
    }

    /// Los contornos y los nodos del último dibujo, sobre el dibujo apagado (el modo «Contornos»).
    pub fn pixeles_contornos_vector(&self, id: u64) -> R<(u32, u32, Vec<u8>)> {
        let h = self.hecho(id)?;
        let px = apolo_nucleo::vector::contornos(&h.documento)?;
        Ok((h.documento.ancho, h.documento.alto, px))
    }

    /// Las tres medidas del último dibujo frente a lo que recibió el motor.
    pub fn medir_vector(&self, id: u64) -> R<Medidas> {
        let h = self.hecho(id)?;
        Ok(medir::medir(
            &h.referencia,
            &h.vista,
            h.documento.ancho,
            h.documento.alto,
        )?)
    }

    /// El mapa de diferencias del último dibujo.
    pub fn pixeles_mapa_vector(&self, id: u64, tipo: TipoMapa) -> R<(u32, u32, Vec<u8>)> {
        let h = self.hecho(id)?;
        let (w, hh) = (h.documento.ancho, h.documento.alto);
        Ok((w, hh, medir::mapa(&h.referencia, &h.vista, w, hh, tipo)?))
    }

    /// Los bytes del fichero: del último dibujo si el ajuste da el mismo dibujo (solo cambian las opciones
    /// de exportar), o de uno nuevo.
    pub fn bytes_vector(
        &self,
        id: u64,
        formato: FormatoVector,
        ajuste: &AjusteVector,
    ) -> R<Vec<u8>> {
        let previo = {
            let u = self.ultimo_vector(id);
            let u = u.lock().unwrap();
            match (&u.ajuste, &u.hecho) {
                (Some(a), Some(h)) if a.mismo_dibujo(ajuste) => Some(h.clone()),
                _ => None,
            }
        };
        let hecho = match previo {
            Some(h) => h,
            None => Arc::new(vector::vectorizar(&self.abierta(id)?.imagen, ajuste, None)?),
        };
        Ok(vector::exportar(&hecho.documento, formato, ajuste)?)
    }

    /// Escribe el fichero.
    pub fn exportar_vector(
        &self,
        id: u64,
        formato: FormatoVector,
        ajuste: &AjusteVector,
        ruta: &Path,
    ) -> R<usize> {
        let datos = self.bytes_vector(id, formato, ajuste)?;
        std::fs::write(ruta, &datos).map_err(|e| {
            Fallo::nuevo(tr!(
                "No se puede escribir «{}»: {e}",
                "Can't write “{}”: {e}",
                ruta.display()
            ))
        })?;
        Ok(datos.len())
    }

    /// El nombre que se propone al exportar el dibujo.
    pub fn nombre_salida_vector(&self, id: u64, formato: FormatoVector) -> R<String> {
        Ok(nombre_vector(&self.abierta(id)?.nombre, formato))
    }
}

/// `logo.png` → `logo.svg` (y `logo-apolo.svg` si ya era un SVG).
pub fn nombre_vector(nombre: &str, formato: FormatoVector) -> String {
    let p = Path::new(nombre);
    let base = p
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "imagen".into());
    let ext = formato.extension();
    if p.extension().is_some_and(|e| e.eq_ignore_ascii_case(ext)) {
        format!("{base}-apolo.{ext}")
    } else {
        format!("{base}.{ext}")
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn png() -> Vec<u8> {
        let lado = 64u32;
        let mut px = Vec::new();
        for y in 0..lado {
            for x in 0..lado {
                let dentro = (16..48).contains(&x) && (16..48).contains(&y);
                px.extend(if dentro {
                    [20, 90, 200, 255]
                } else {
                    [255, 255, 255, 255]
                });
            }
        }
        let mut out = Vec::new();
        {
            let mut e = png::Encoder::new(&mut out, lado, lado);
            e.set_color(png::ColorType::Rgba);
            e.set_depth(png::BitDepth::Eight);
            e.write_header().unwrap().write_image_data(&px).unwrap();
        }
        out
    }

    fn servicio() -> (Servicio, u64) {
        let dir =
            std::env::temp_dir().join(format!("apolo-servicio-vector-{}", std::process::id()));
        let s = Servicio::nuevo(dir);
        let id = s.abrir_bytes("logo.png", png()).unwrap().id;
        (s, id)
    }

    #[test]
    fn vectorizar_medir_y_exportar() {
        let (s, id) = servicio();
        assert!(s.pixeles_vector(id).is_err());
        let v = s.vectorizar(id, &AjusteVector::default(), 1).unwrap();
        assert_eq!((v.ancho, v.alto), (64, 64));
        assert!(v.nota.unwrap() > 85.0, "{v:?}");
        let (w, h, px) = s.pixeles_vector(id).unwrap();
        assert_eq!(px.len(), (w * h * 4) as usize);
        assert_eq!(s.pixeles_referencia_vector(id).unwrap().2.len(), px.len());
        assert!(s.medir_vector(id).unwrap().ssim > 0.9);
        assert_eq!(s.pixeles_contornos_vector(id).unwrap().2.len(), px.len());
        for f in FormatoVector::TODOS {
            let b = s.bytes_vector(id, f, &AjusteVector::default()).unwrap();
            assert!(!b.is_empty(), "{f:?}");
        }
        assert_eq!(
            s.nombre_salida_vector(id, FormatoVector::Pdf).unwrap(),
            "logo.pdf"
        );
        s.cerrar(id);
        assert!(s.pixeles_vector(id).is_err());
    }

    #[test]
    fn exportar_con_otro_estilo_no_vuelve_a_vectorizar() {
        let (s, id) = servicio();
        s.vectorizar(id, &AjusteVector::default(), 1).unwrap();
        let bordes = AjusteVector {
            estilo: vector::Estilo::TrazarBordes,
            ..Default::default()
        };
        assert!(AjusteVector::default().mismo_dibujo(&bordes));
        let svg =
            String::from_utf8(s.bytes_vector(id, FormatoVector::Svg, &bordes).unwrap()).unwrap();
        assert!(svg.contains("stroke="), "{svg}");
    }

    #[test]
    fn una_generacion_vieja_se_cancela() {
        let (s, id) = servicio();
        s.vectorizar(id, &AjusteVector::default(), 5).unwrap();
        let r = s.vectorizar(id, &AjusteVector::default(), 3);
        assert!(r.is_err_and(|f| f.cancelado));
    }

    #[test]
    fn nombres() {
        assert_eq!(nombre_vector("foto.jpg", FormatoVector::Svg), "foto.svg");
        assert_eq!(
            nombre_vector("logo.SVG", FormatoVector::Svg),
            "logo-apolo.svg"
        );
        assert_eq!(nombre_vector("plano", FormatoVector::Dxf), "plano.dxf");
    }
}
