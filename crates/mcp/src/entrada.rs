//! Lo que pide un agente para codificar (ADR 0023), y cómo se convierte en un
//! `Ajuste` del núcleo.
//!
//! No se expone `Ajuste` entero: serían unos cien campos en cada lista de
//! herramientas, y el modelo rellenaría los que no debe. Se pide poco, y el
//! que sabe lo que quiere pasa la orden de la herramienta oficial en `orden`.

use apolo_nucleo::entrada::Formato;
use apolo_nucleo::presets::PresetGuardado;
use apolo_nucleo::proceso::{Paleta, Recorte, Redimension};
use apolo_nucleo::salida::{self, Ajuste, FormatoSalida};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::errores::{ErrorMcp, Hecho};

/// La nota SSIMULACRA 2 que se busca si no se dice otra cosa: 80, «no se
/// distingue lado a lado» (decidido con el cliente, ADR 0023).
pub const NOTA_POR_DEFECTO: f32 = 80.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum FormatoMcp {
    Webp,
    Jpeg,
    Png,
    Qoi,
    Avif,
    Jxl,
}

impl From<FormatoMcp> for FormatoSalida {
    fn from(f: FormatoMcp) -> Self {
        match f {
            FormatoMcp::Webp => FormatoSalida::Webp,
            FormatoMcp::Jpeg => FormatoSalida::Jpeg,
            FormatoMcp::Png => FormatoSalida::Png,
            FormatoMcp::Qoi => FormatoSalida::Qoi,
            FormatoMcp::Avif => FormatoSalida::Avif,
            FormatoMcp::Jxl => FormatoSalida::Jxl,
        }
    }
}

/// Un rectángulo de la imagen, en píxeles (ya enderezada).
#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
pub struct RecorteMcp {
    /// Desde la izquierda.
    pub x: u32,
    /// Desde arriba.
    pub y: u32,
    pub ancho: u32,
    pub alto: u32,
}

/// Cómo codificar: todo opcional. Sin nada, WebP con la calidad más baja que
/// da una nota SSIMULACRA 2 de 80.
#[derive(Debug, Clone, Default, Deserialize, JsonSchema)]
pub struct EntradaFormato {
    /// webp, jpeg, png, qoi, avif o jxl. Por defecto, el del preset, o webp.
    #[serde(default)]
    pub formato: Option<FormatoMcp>,
    /// Un preset guardado (listar_presets los da). Es la base de lo demás.
    #[serde(default)]
    pub preset: Option<String>,
    /// Opciones de la herramienta oficial, tal cual: «-q 80 -m 6» (cwebp),
    /// «-quality 85 -progressive» (cjpeg), «-o 4» (oxipng), «-q 60 -s 6»
    /// (avifenc), «-d 1 -e 7» (cjxl). Puede empezar por el nombre de la
    /// herramienta. Para su formato, sustituye las opciones del preset.
    #[serde(default)]
    pub orden: Option<String>,
    /// Calidad fija de 0 a 100. No a la vez que nota_objetivo.
    #[serde(default)]
    #[schemars(range(min = 0, max = 100))]
    pub calidad: Option<u8>,
    /// Buscar la calidad más baja que da esta nota SSIMULACRA 2 (90 no se
    /// distingue; 80 muy alta; 70 alta; 50 media). Tarda unos segundos.
    #[serde(default)]
    #[schemars(range(min = 0, max = 100))]
    pub nota_objetivo: Option<f32>,
    /// Sin pérdida (WebP, AVIF, JPEG XL). PNG y QOI siempre lo son.
    #[serde(default)]
    pub sin_perdida: bool,
    /// Ancho máximo, en píxeles: reduce si es mayor y nunca agranda (con alto solo, proporcional).
    #[serde(default)]
    pub ancho: Option<u32>,
    /// Alto máximo, en píxeles: reduce si es mayor y nunca agranda (con ancho solo, proporcional).
    #[serde(default)]
    pub alto: Option<u32>,
    /// Girar según la orientación EXIF si la foto lo necesita. Por defecto, sí.
    #[serde(default)]
    pub enderezar: Option<bool>,
    /// Quedarse con este rectángulo, antes de redimensionar (en píxeles de la
    /// imagen ya enderezada; lo que se salga del borde se ajusta).
    #[serde(default)]
    pub recorte: Option<RecorteMcp>,
    /// Reducir a este número de colores (de 2 a 256), con tramado: sobre todo
    /// para PNG de logos, iconos y capturas.
    #[serde(default)]
    #[schemars(range(min = 2, max = 256))]
    pub paleta: Option<u32>,
}

impl EntradaFormato {
    /// El ajuste para una imagen de formato `entrada` y orientación EXIF
    /// `orientacion` (1 si no se sabe, como en un lote). Por orden: preset,
    /// formato, orden, calidad o nota, sin pérdida, y el proceso.
    pub fn ajuste(
        &self,
        presets: &[PresetGuardado],
        entrada: Option<Formato>,
        orientacion: u8,
    ) -> Hecho<Ajuste> {
        if self.calidad.is_some() && self.nota_objetivo.is_some() {
            return Err(ErrorMcp::nuevo(
                "nota_y_calidad",
                "Se ha pedido una calidad fija y una nota a la vez.",
            )
            .con_pista(
                "Quita calidad (para buscar la nota) o nota_objetivo (para la calidad fija).",
            ));
        }
        let mut a = match &self.preset {
            Some(nombre) => presets
                .iter()
                .find(|p| p.nombre.eq_ignore_ascii_case(nombre.trim()))
                .map(|p| p.ajuste.clone())
                .ok_or_else(|| {
                    ErrorMcp::nuevo(
                        "preset_no_existe",
                        format!("No hay ningún preset «{nombre}»."),
                    )
                    .con_pista("listar_presets da los que hay.")
                })?,
            None => Ajuste::default(),
        };
        if let Some(f) = self.formato {
            a.formato = f.into();
        }
        if let Some(orden) = self.orden.as_deref().filter(|o| !o.trim().is_empty()) {
            let leido = apolo_servicio::leer_orden(orden, &a).map_err(|f| {
                ErrorMcp::nuevo("orden_no_valida", f.mensaje).con_pista(
                    "Opciones como «-q 80 -m 6» (cwebp), «-quality 85» (cjpeg), «-o 4» (oxipng), «-q 60 -s 6» (avifenc) o «-d 1 -e 7» (cjxl); o usa preset.",
                )
            })?;
            if let Some(f) = self.formato
                && FormatoSalida::from(f) != leido.formato
            {
                return Err(ErrorMcp::nuevo(
                    "formato_y_orden",
                    format!(
                        "La orden es de {} y el formato pedido, {}.",
                        leido.formato.herramienta(),
                        FormatoSalida::from(f).nombre()
                    ),
                )
                .con_pista("Quita formato, o da una orden de su herramienta."));
            }
            a = leido;
        }
        if let Some(q) = self.calidad {
            if matches!(a.formato, FormatoSalida::Png | FormatoSalida::Qoi) {
                return Err(sin_calidad(a.formato));
            }
            a = salida::con_calidad(&a, q.min(100));
        }
        if self.sin_perdida {
            match a.formato {
                FormatoSalida::Webp => a.webp.sin_perdida = true,
                FormatoSalida::Avif => a.avif.sin_perdida = true,
                FormatoSalida::Jxl => {
                    a.jxl.distancia = Some(0.0);
                    a.jxl.calidad = None;
                }
                FormatoSalida::Png | FormatoSalida::Qoi => {}
                FormatoSalida::Jpeg => {
                    return Err(ErrorMcp::nuevo(
                        "sin_perdida_no",
                        "JPEG no tiene modo sin pérdida.",
                    )
                    .con_pista(
                        "Usa png, webp con sin_perdida, o jxl (un JPEG se recomprime sin pérdida).",
                    ));
                }
            }
        }
        if let Some(n) = self.nota_objetivo {
            a.objetivo = Some(n.clamp(0.0, 100.0));
        } else if self.calidad.is_none()
            && self.orden.is_none()
            && !self.sin_perdida
            && self.preset.is_none()
        {
            a.objetivo = Some(NOTA_POR_DEFECTO);
        }
        if a.objetivo.is_some() {
            let admite = match entrada {
                Some(e) => salida::admite_objetivo(&a, e),
                // En un lote no se sabe la entrada: el núcleo lo mira imagen a
                // imagen, y sin calidad que buscar codifica tal cual.
                None => salida::admite_objetivo(&a, Formato::Png),
            };
            if !admite {
                if self.nota_objetivo.is_some() {
                    return Err(sin_calidad(a.formato));
                }
                a.objetivo = None;
            }
        }
        if self.ancho.is_some() || self.alto.is_some() {
            // Un máximo: nunca agranda (ADR 0023). Los agentes lo entienden así.
            a.proceso.redimension = Some(Redimension {
                ancho: self.ancho,
                alto: self.alto,
                solo_reducir: true,
                ..Default::default()
            });
        }
        if let Some(r) = self.recorte {
            if r.ancho == 0 || r.alto == 0 {
                return Err(ErrorMcp::nuevo("recorte_vacio", "El recorte no tiene ancho o alto.")
                    .con_pista("recorte es {x, y, ancho, alto}, en píxeles, con ancho y alto mayores que 0."));
            }
            a.proceso.recorte = Some(Recorte {
                x: r.x,
                y: r.y,
                ancho: r.ancho,
                alto: r.alto,
            });
        }
        if let Some(colores) = self.paleta {
            if !(2..=256).contains(&colores) {
                return Err(ErrorMcp::nuevo(
                    "paleta_fuera",
                    format!("Una paleta de {colores} colores no vale."),
                )
                .con_pista("paleta va de 2 a 256 colores."));
            }
            a.proceso.paleta = Some(Paleta {
                colores,
                ..Default::default()
            });
        }
        // Solo si la foto lo necesita: con orientación 1, «enderezada» diría
        // que la orden oficial no da el fichero, y sí lo da.
        let enderezar = self.enderezar.unwrap_or(true) && orientacion != 1;
        a.proceso.enderezar = enderezar;
        a.webp.enderezar = false;
        a.jpeg.enderezar = false;
        a.png.enderezar = false;
        a.avif.enderezar = false;
        a.jxl.enderezar = false;
        Ok(a)
    }
}

fn sin_calidad(f: FormatoSalida) -> ErrorMcp {
    ErrorMcp::nuevo(
        "nota_no_admitida",
        format!(
            "{} no tiene calidad que elegir o buscar (es sin pérdida).",
            f.nombre()
        ),
    )
    .con_pista("Quita nota_objetivo y calidad, o elige webp, jpeg, avif o jxl.")
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn preset(nombre: &str, a: Ajuste) -> PresetGuardado {
        PresetGuardado {
            nombre: nombre.into(),
            ajuste: a,
        }
    }

    #[test]
    fn sin_nada_es_webp_con_nota_80() {
        let a = EntradaFormato::default()
            .ajuste(&[], Some(Formato::Png), 1)
            .unwrap();
        assert_eq!(a.formato, FormatoSalida::Webp);
        assert_eq!(a.objetivo, Some(NOTA_POR_DEFECTO));
        assert!(!a.proceso.enderezar);
    }

    #[test]
    fn la_calidad_fija_quita_la_nota_y_png_no_tiene_nota() {
        let e = EntradaFormato {
            formato: Some(FormatoMcp::Jpeg),
            calidad: Some(70),
            ..Default::default()
        };
        let a = e.ajuste(&[], Some(Formato::Png), 1).unwrap();
        assert_eq!((a.jpeg.calidad.clone(), a.objetivo), (vec![70.0], None));
        let png = EntradaFormato {
            formato: Some(FormatoMcp::Png),
            ..Default::default()
        };
        assert_eq!(
            png.ajuste(&[], Some(Formato::Png), 1).unwrap().objetivo,
            None
        );
        let png_nota = EntradaFormato {
            nota_objetivo: Some(85.0),
            ..png
        };
        assert_eq!(
            png_nota
                .ajuste(&[], Some(Formato::Png), 1)
                .unwrap_err()
                .error,
            "nota_no_admitida"
        );
    }

    #[test]
    fn el_preset_es_la_base_y_la_orden_manda_en_su_formato() {
        let mut base = Ajuste::default();
        base.webp.calidad = 55.0;
        base.webp.metodo = 6;
        let p = [preset("Fotos web", base)];
        let solo = EntradaFormato {
            preset: Some("fotos WEB".into()),
            ..Default::default()
        };
        let a = solo.ajuste(&p, Some(Formato::Png), 1).unwrap();
        assert_eq!((a.webp.calidad, a.webp.metodo, a.objetivo), (55.0, 6, None));
        let con_orden = EntradaFormato {
            orden: Some("-q 90".into()),
            ..solo
        };
        let a = con_orden.ajuste(&p, Some(Formato::Png), 1).unwrap();
        assert_eq!(a.webp.calidad, 90.0);
        let falta = EntradaFormato {
            preset: Some("No existe".into()),
            ..Default::default()
        };
        assert_eq!(
            falta.ajuste(&p, None, 1).unwrap_err().error,
            "preset_no_existe"
        );
    }

    #[test]
    fn una_orden_de_otra_herramienta_que_el_formato_es_un_error() {
        let e = EntradaFormato {
            formato: Some(FormatoMcp::Webp),
            orden: Some("cjxl -d 1".into()),
            ..Default::default()
        };
        assert_eq!(
            e.ajuste(&[], Some(Formato::Png), 1).unwrap_err().error,
            "formato_y_orden"
        );
        let solo_orden = EntradaFormato {
            orden: Some("cjxl -d 1 -e 7".into()),
            ..Default::default()
        };
        assert_eq!(
            solo_orden
                .ajuste(&[], Some(Formato::Png), 1)
                .unwrap()
                .formato,
            FormatoSalida::Jxl
        );
    }

    #[test]
    fn se_endereza_solo_si_la_foto_lo_necesita() {
        let e = EntradaFormato::default();
        assert!(
            e.ajuste(&[], Some(Formato::Jpeg), 6)
                .unwrap()
                .proceso
                .enderezar
        );
        assert!(
            !e.ajuste(&[], Some(Formato::Jpeg), 1)
                .unwrap()
                .proceso
                .enderezar
        );
        let no = EntradaFormato {
            enderezar: Some(false),
            ..Default::default()
        };
        assert!(
            !no.ajuste(&[], Some(Formato::Jpeg), 6)
                .unwrap()
                .proceso
                .enderezar
        );
    }

    #[test]
    fn calidad_y_nota_a_la_vez_no() {
        let e = EntradaFormato {
            calidad: Some(70),
            nota_objetivo: Some(80.0),
            ..Default::default()
        };
        assert_eq!(e.ajuste(&[], None, 1).unwrap_err().error, "nota_y_calidad");
    }
}
