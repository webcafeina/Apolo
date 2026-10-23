//! Lo que pide un agente para codificar (ADR 0023), y cómo se convierte en un
//! `Ajuste` del núcleo.
//!
//! No se expone `Ajuste` entero: serían unos cien campos en cada lista de
//! herramientas, y el modelo rellenaría los que no debe. Se pide poco, y el
//! que sabe lo que quiere pasa la orden de la herramienta oficial en `orden`.

use apolo_nucleo::entrada::Formato;
use apolo_nucleo::presets::PresetGuardado;
use apolo_nucleo::proceso::{Medida, Paleta, Recorte, Redimension};
use apolo_nucleo::salida::{self, Ajuste, FormatoSalida};
use apolo_nucleo::{tr, trs};
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

/// El esquema de un rectángulo `{x, y, ancho, alto}`, escrito en el sitio y
/// sin `$ref`: Claude Desktop mandaba como texto los objetos que llegaban por
/// una referencia a `$defs` («invalid type: string, expected struct
/// RecorteMcp»; prueba guiada de la v0.9.0). Claude Code los mandaba bien.
pub fn esquema_rectangulo(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
    schemars::json_schema!({
        "type": "object",
        "properties": {
            "x": { "type": "integer", "minimum": 0, "description": "Desde la izquierda, en píxeles." },
            "y": { "type": "integer", "minimum": 0, "description": "Desde arriba, en píxeles." },
            "ancho": { "type": "integer", "minimum": 1 },
            "alto": { "type": "integer", "minimum": 1 }
        },
        "required": ["x", "y", "ancho", "alto"]
    })
}

/// Lee un objeto opcional que puede llegar como objeto o como texto con el
/// JSON dentro (`"{\"x\":0,…}"`), que es como lo mandaba Claude Desktop.
pub fn objeto_o_texto<'de, D, T>(d: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::de::DeserializeOwned,
{
    use serde::de::Error;
    match serde_json::Value::deserialize(d)? {
        serde_json::Value::Null => Ok(None),
        serde_json::Value::String(t) if t.trim().is_empty() => Ok(None),
        serde_json::Value::String(t) => {
            serde_json::from_str(&t).map(Some).map_err(D::Error::custom)
        }
        v => serde_json::from_value(v)
            .map(Some)
            .map_err(D::Error::custom),
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
    /// Lado mayor máximo, en píxeles: el ancho si es apaisada, el alto si es
    /// vertical; nunca agranda. No a la vez que ancho, alto o porcentaje.
    #[serde(default)]
    pub lado_mayor: Option<u32>,
    /// Reducir al tanto por ciento (50 = la mitad de ancho y de alto). De más
    /// de 0 a 100. No a la vez que ancho, alto o lado_mayor.
    #[serde(default)]
    pub porcentaje: Option<f64>,
    /// Girar según la orientación EXIF si la foto lo necesita. Por defecto, sí.
    #[serde(default)]
    pub enderezar: Option<bool>,
    /// Quedarse con este rectángulo, antes de redimensionar (en píxeles de la
    /// imagen ya enderezada; lo que se salga del borde se ajusta).
    #[serde(default, deserialize_with = "objeto_o_texto")]
    #[schemars(schema_with = "esquema_rectangulo")]
    pub recorte: Option<RecorteMcp>,
    /// Reducir a este número de colores (de 2 a 256), con tramado: sobre todo
    /// para PNG de logos, iconos y capturas.
    #[serde(default)]
    #[schemars(range(min = 2, max = 256))]
    pub paleta: Option<u32>,
}

/// La redimensión que pide un agente: ancho y alto, lado mayor o porcentaje,
/// una sola de las tres. Siempre es un máximo y nunca agranda (ADR 0023): los
/// agentes lo entienden así.
pub fn redimension_pedida(
    ancho: Option<u32>,
    alto: Option<u32>,
    lado_mayor: Option<u32>,
    porcentaje: Option<f64>,
) -> Hecho<Option<Redimension>> {
    let medidas = ancho.is_some() || alto.is_some();
    if [medidas, lado_mayor.is_some(), porcentaje.is_some()]
        .iter()
        .filter(|&&x| x)
        .count()
        > 1
    {
        return Err(ErrorMcp::nuevo(
            "tamano_doble",
            trs!(
                "Se ha pedido el tamaño de dos maneras a la vez.",
                "The size was asked for in two ways at the same time."
            ),
        )
        .con_pista(trs!(
            "Usa ancho y alto, o lado_mayor, o porcentaje: solo una.",
            "Use ancho and alto, or lado_mayor, or porcentaje: only one."
        )));
    }
    let medida = match (lado_mayor, porcentaje) {
        (Some(0), _) => {
            return Err(ErrorMcp::nuevo(
                "lado_mayor_cero",
                trs!(
                    "Un lado mayor de 0 px no vale.",
                    "A longest side of 0 px isn't valid."
                ),
            )
            .con_pista(trs!(
                "lado_mayor es en píxeles, como 1920.",
                "lado_mayor is in pixels, like 1920."
            )));
        }
        (Some(l), _) => Medida::LadoMayor(l),
        (_, Some(p)) if !(p.is_finite() && p > 0.0 && p <= 100.0) => {
            return Err(ErrorMcp::nuevo(
                "porcentaje_fuera",
                tr!(
                    "Un porcentaje de {p} no vale.",
                    "A percentage of {p} isn't valid."
                ),
            )
            .con_pista(trs!(
                "porcentaje va de más de 0 a 100: Apolo nunca agranda.",
                "porcentaje goes from above 0 to 100: Apolo never enlarges."
            )));
        }
        (_, Some(p)) => Medida::Porcentaje(p),
        (None, None) if medidas => Medida::Medidas { ancho, alto },
        (None, None) => return Ok(None),
    };
    Ok(Some(Redimension {
        medida,
        solo_reducir: true,
        ..Default::default()
    }))
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
                trs!(
                    "Se ha pedido una calidad fija y una nota a la vez.",
                    "A fixed quality and a target score were asked for at the same time."
                ),
            )
            .con_pista(trs!(
                "Quita calidad (para buscar la nota) o nota_objetivo (para la calidad fija).",
                "Remove calidad (to search for the score) or nota_objetivo (for the fixed quality)."
            )));
        }
        let mut a = match &self.preset {
            Some(nombre) => presets
                .iter()
                .find(|p| p.nombre.eq_ignore_ascii_case(nombre.trim()))
                .map(|p| p.ajuste.clone())
                .ok_or_else(|| {
                    ErrorMcp::nuevo(
                        "preset_no_existe",
                        tr!(
                            "No hay ningún preset «{nombre}».",
                            "There is no preset “{nombre}”."
                        ),
                    )
                    .con_pista(trs!(
                        "listar_presets da los que hay.",
                        "listar_presets lists the ones there are."
                    ))
                })?,
            None => Ajuste::default(),
        };
        if let Some(f) = self.formato {
            a.formato = f.into();
        }
        if let Some(orden) = self.orden.as_deref().filter(|o| !o.trim().is_empty()) {
            let leido = apolo_servicio::leer_orden(orden, &a).map_err(|f| {
                ErrorMcp::nuevo("orden_no_valida", f.mensaje).con_pista(trs!(
                    "Opciones como «-q 80 -m 6» (cwebp), «-quality 85» (cjpeg), «-o 4» (oxipng), «-q 60 -s 6» (avifenc) o «-d 1 -e 7» (cjxl); o usa preset.",
                    "Options like “-q 80 -m 6” (cwebp), “-quality 85” (cjpeg), “-o 4” (oxipng), “-q 60 -s 6” (avifenc) or “-d 1 -e 7” (cjxl); or use preset."
                ))
            })?;
            if let Some(f) = self.formato
                && FormatoSalida::from(f) != leido.formato
            {
                return Err(ErrorMcp::nuevo(
                    "formato_y_orden",
                    tr!(
                        "La orden es de {} y el formato pedido, {}.",
                        "The command is for {} and the format asked for is {}.",
                        leido.formato.herramienta(),
                        FormatoSalida::from(f).nombre()
                    ),
                )
                .con_pista(trs!(
                    "Quita formato, o da una orden de su herramienta.",
                    "Remove formato, or give a command for its tool."
                )));
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
                        trs!(
                            "JPEG no tiene modo sin pérdida.",
                            "JPEG has no lossless mode."
                        ),
                    )
                    .con_pista(trs!(
                        "Usa png, webp con sin_perdida, o jxl (un JPEG se recomprime sin pérdida).",
                        "Use png, webp with sin_perdida, or jxl (a JPEG is recompressed losslessly)."
                    )));
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
        if let Some(r) =
            redimension_pedida(self.ancho, self.alto, self.lado_mayor, self.porcentaje)?
        {
            a.proceso.redimension = Some(r);
        }
        if let Some(r) = self.recorte {
            if r.ancho == 0 || r.alto == 0 {
                return Err(ErrorMcp::nuevo(
                    "recorte_vacio",
                    trs!(
                        "El recorte no tiene ancho o alto.",
                        "The crop has no width or height."
                    ),
                )
                .con_pista(trs!(
                    "recorte es {x, y, ancho, alto}, en píxeles, con ancho y alto mayores que 0.",
                    "recorte is {x, y, ancho, alto}, in pixels, with ancho and alto greater than 0."
                )));
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
                    tr!(
                        "Una paleta de {colores} colores no vale.",
                        "A palette of {colores} colors isn't valid."
                    ),
                )
                .con_pista(trs!(
                    "paleta va de 2 a 256 colores.",
                    "paleta goes from 2 to 256 colors."
                )));
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
        tr!(
            "{} no tiene calidad que elegir o buscar (es sin pérdida).",
            "{} has no quality to choose or search for (it's lossless).",
            f.nombre()
        ),
    )
    .con_pista(trs!(
        "Quita nota_objetivo y calidad, o elige webp, jpeg, avif o jxl.",
        "Remove nota_objetivo and calidad, or choose webp, jpeg, avif or jxl."
    ))
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
