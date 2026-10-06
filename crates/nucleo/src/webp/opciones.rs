//! Las opciones de WebP: lo que guarda un preset y lo que pinta la interfaz.
//!
//! Son los campos de `WebPConfig` que `cwebp` puede tocar, con nombre en
//! español, más lo que cwebp hace alrededor de la librería (recortar,
//! redimensionar, mezclar el alfa, metadatos). Los campos de `WebPConfig` que
//! cwebp no deja tocar (`partitions`, `use_delta_palette`) no están: Apolo
//! promete el mismo fichero que cwebp, y eso pasa por no salirse de lo que
//! cwebp puede expresar.

use libwebp_sys as w;
use serde::{Deserialize, Serialize};

use crate::metadatos::Conservar;

/// `-preset`: un punto de partida que reescribe varios ajustes a la vez.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Preset {
    Default,
    Photo,
    Picture,
    Drawing,
    Icon,
    Text,
}

impl Preset {
    pub const TODOS: [Preset; 6] = [
        Preset::Default,
        Preset::Photo,
        Preset::Picture,
        Preset::Drawing,
        Preset::Icon,
        Preset::Text,
    ];

    pub fn nombre_cwebp(self) -> &'static str {
        match self {
            Preset::Default => "default",
            Preset::Photo => "photo",
            Preset::Picture => "picture",
            Preset::Drawing => "drawing",
            Preset::Icon => "icon",
            Preset::Text => "text",
        }
    }

    pub fn desde_cwebp(s: &str) -> Option<Preset> {
        Preset::TODOS.into_iter().find(|p| p.nombre_cwebp() == s)
    }

    fn libwebp(self) -> w::WebPPreset {
        use w::WebPPreset::*;
        match self {
            Preset::Default => WEBP_PRESET_DEFAULT,
            Preset::Photo => WEBP_PRESET_PHOTO,
            Preset::Picture => WEBP_PRESET_PICTURE,
            Preset::Drawing => WEBP_PRESET_DRAWING,
            Preset::Icon => WEBP_PRESET_ICON,
            Preset::Text => WEBP_PRESET_TEXT,
        }
    }
}

/// `-hint`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Pista {
    #[default]
    Ninguna,
    Picture,
    Photo,
    Graph,
}

/// `-alpha_filter`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FiltradoAlfa {
    None,
    #[default]
    Fast,
    Best,
}

/// `-resize_mode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModoRedimension {
    SoloReducir,
    SoloAmpliar,
    #[default]
    Siempre,
}

/// `-crop x y w h`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Recorte {
    pub x: i32,
    pub y: i32,
    pub ancho: i32,
    pub alto: i32,
}

/// `-resize w h`. Un 0 mantiene la proporción.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Redimension {
    pub ancho: i32,
    pub alto: i32,
}

/// Todo lo que decide el fichero WebP de salida.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct OpcionesWebp {
    /// `-preset`: solo dice de qué preset se partió, para escribir la orden
    /// más corta. Los valores que el preset puso ya están en los campos.
    pub preset: Option<Preset>,

    /// `-lossless`
    pub sin_perdida: bool,
    /// `-q`: calidad con pérdida, o esfuerzo sin pérdida. 0–100.
    pub calidad: f32,
    /// `-m`: método, 0 (rápido) a 6 (lento y pequeño).
    pub metodo: i32,
    /// `-hint`
    pub pista: Pista,
    /// `-size`: tamaño objetivo en bytes. 0 = sin objetivo.
    pub tamano_objetivo: i32,
    /// `-psnr`: PSNR objetivo en dB. 0 = sin objetivo.
    pub psnr_objetivo: f32,
    /// `-segments`: 1–4.
    pub segmentos: i32,
    /// `-sns`: modelado espacial del ruido, 0–100.
    pub sns: i32,
    /// `-f`: fuerza del filtro de desbloqueo, 0–100.
    pub fuerza_filtro: i32,
    /// `-sharpness`: 0 (más nítido) a 7.
    pub nitidez_filtro: i32,
    /// `-strong` / `-nostrong`
    pub filtro_fuerte: bool,
    /// `-af`
    pub autofiltro: bool,
    /// `-alpha_method`: 0 sin comprimir, 1 sin pérdida.
    pub compresion_alfa: i32,
    /// `-alpha_filter`
    pub filtrado_alfa: FiltradoAlfa,
    /// `-alpha_q`: 0–100.
    pub calidad_alfa: i32,
    /// `-pass`: 1–10.
    pub pasadas: i32,
    /// `-pre`: preprocesado (experimental).
    pub preprocesado: i32,
    /// `-partition_limit`: 0–100.
    pub limite_particion: i32,
    /// `-jpeg_like`
    pub emular_jpeg: bool,
    /// `-mt`, tantas veces como el nivel.
    pub hilos: i32,
    /// `-low_memory`
    pub poca_memoria: bool,
    /// `-near_lossless`: 0–100; 100 es apagado.
    pub casi_sin_perdida: i32,
    /// `-exact`
    pub exacto: bool,
    /// `-sharp_yuv`
    pub yuv_nitido: bool,
    /// `-qrange min max`
    pub calidad_minima: i32,
    pub calidad_maxima: i32,

    /// `-crop`
    pub recorte: Option<Recorte>,
    /// `-resize`
    pub redimension: Option<Redimension>,
    /// `-resize_mode`
    pub modo_redimension: ModoRedimension,
    /// `-blend_alpha 0xRRGGBB`
    pub mezclar_alfa: Option<u32>,
    /// `-noalpha`
    pub sin_alfa: bool,
    /// `-metadata`
    pub metadatos: Conservar,

    /// Enderezar según la orientación EXIF (ADR 0012). **No es de cwebp**:
    /// encendida, la orden cwebp equivalente ya no da el mismo fichero.
    pub enderezar: bool,
}

impl Default for OpcionesWebp {
    fn default() -> Self {
        OpcionesWebp::desde_config(&config_base(None), None)
    }
}

/// `WebPConfigInit` o `WebPConfigPreset(preset, 75)`.
pub(crate) fn config_base(preset: Option<Preset>) -> w::WebPConfig {
    config_preset(preset.unwrap_or(Preset::Default), 75.0)
}

pub(crate) fn config_preset(preset: Preset, calidad: f32) -> w::WebPConfig {
    let mut c = std::mem::MaybeUninit::<w::WebPConfig>::zeroed();
    // SAFETY: inicializa la estructura entera; la versión es la del codificador.
    let ok = unsafe {
        w::WebPConfigInitInternal(
            c.as_mut_ptr(),
            preset.libwebp(),
            calidad,
            w::WEBP_ENCODER_ABI_VERSION as i32,
        )
    };
    assert!(
        ok != 0,
        "libwebp rechaza su propia configuración por defecto"
    );
    // SAFETY: inicializada por WebPConfigInitInternal.
    unsafe { c.assume_init() }
}

impl OpcionesWebp {
    /// Lee los campos de una `WebPConfig`.
    pub(crate) fn desde_config(c: &w::WebPConfig, preset: Option<Preset>) -> Self {
        use w::WebPImageHint::*;
        OpcionesWebp {
            preset,
            sin_perdida: c.lossless != 0,
            calidad: c.quality,
            metodo: c.method,
            pista: match c.image_hint {
                WEBP_HINT_PICTURE => Pista::Picture,
                WEBP_HINT_PHOTO => Pista::Photo,
                WEBP_HINT_GRAPH => Pista::Graph,
                _ => Pista::Ninguna,
            },
            tamano_objetivo: c.target_size,
            psnr_objetivo: c.target_PSNR,
            segmentos: c.segments,
            sns: c.sns_strength,
            fuerza_filtro: c.filter_strength,
            nitidez_filtro: c.filter_sharpness,
            filtro_fuerte: c.filter_type != 0,
            autofiltro: c.autofilter != 0,
            compresion_alfa: c.alpha_compression,
            filtrado_alfa: match c.alpha_filtering {
                0 => FiltradoAlfa::None,
                2 => FiltradoAlfa::Best,
                _ => FiltradoAlfa::Fast,
            },
            calidad_alfa: c.alpha_quality,
            pasadas: c.pass,
            preprocesado: c.preprocessing,
            limite_particion: c.partition_limit,
            emular_jpeg: c.emulate_jpeg_size != 0,
            hilos: c.thread_level,
            poca_memoria: c.low_memory != 0,
            casi_sin_perdida: c.near_lossless,
            exacto: c.exact != 0,
            yuv_nitido: c.use_sharp_yuv != 0,
            calidad_minima: c.qmin,
            calidad_maxima: c.qmax,
            recorte: None,
            redimension: None,
            modo_redimension: ModoRedimension::default(),
            mezclar_alfa: None,
            sin_alfa: false,
            metadatos: Conservar::NINGUNO,
            enderezar: false,
        }
    }

    /// Escribe los campos en una `WebPConfig`, partiendo de la base.
    pub(crate) fn a_config(&self) -> w::WebPConfig {
        let mut c = self.a_config_sin_normalizar();
        // Lo que cwebp hace después de leer las opciones: con un objetivo de
        // tamaño o de PSNR y una sola pasada, fuerza seis.
        if (c.target_size > 0 || c.target_PSNR > 0.0) && c.pass == 1 {
            c.pass = 6;
        }
        c
    }

    /// Los campos tal cual, sin lo que cwebp añade al acabar de leer: es el
    /// punto de partida para seguir leyendo opciones encima.
    pub(crate) fn a_config_sin_normalizar(&self) -> w::WebPConfig {
        use w::WebPImageHint::*;
        let mut c = config_base(None);
        c.lossless = self.sin_perdida as i32;
        c.quality = self.calidad;
        c.method = self.metodo;
        c.image_hint = match self.pista {
            Pista::Ninguna => WEBP_HINT_DEFAULT,
            Pista::Picture => WEBP_HINT_PICTURE,
            Pista::Photo => WEBP_HINT_PHOTO,
            Pista::Graph => WEBP_HINT_GRAPH,
        };
        c.target_size = self.tamano_objetivo;
        c.target_PSNR = self.psnr_objetivo;
        c.segments = self.segmentos;
        c.sns_strength = self.sns;
        c.filter_strength = self.fuerza_filtro;
        c.filter_sharpness = self.nitidez_filtro;
        c.filter_type = self.filtro_fuerte as i32;
        c.autofilter = self.autofiltro as i32;
        c.alpha_compression = self.compresion_alfa;
        c.alpha_filtering = match self.filtrado_alfa {
            FiltradoAlfa::None => 0,
            FiltradoAlfa::Fast => 1,
            FiltradoAlfa::Best => 2,
        };
        c.alpha_quality = self.calidad_alfa;
        c.pass = self.pasadas;
        c.preprocessing = self.preprocesado;
        c.partition_limit = self.limite_particion;
        c.emulate_jpeg_size = self.emular_jpeg as i32;
        c.thread_level = self.hilos;
        c.low_memory = self.poca_memoria as i32;
        c.near_lossless = self.casi_sin_perdida;
        c.exact = self.exacto as i32;
        c.use_sharp_yuv = self.yuv_nitido as i32;
        c.qmin = self.calidad_minima;
        c.qmax = self.calidad_maxima;
        c
    }

    /// Aplica un preset como `-preset` en cwebp: reescribe la configuración
    /// entera desde el preset, conservando la calidad.
    pub fn aplicar_preset(&mut self, preset: Preset) {
        let c = config_preset(preset, self.calidad);
        let fuera = (
            self.recorte,
            self.redimension,
            self.modo_redimension,
            self.mezclar_alfa,
            self.sin_alfa,
            self.metadatos,
            self.enderezar,
        );
        *self = OpcionesWebp::desde_config(&c, Some(preset));
        (
            self.recorte,
            self.redimension,
            self.modo_redimension,
            self.mezclar_alfa,
            self.sin_alfa,
            self.metadatos,
            self.enderezar,
        ) = fuera;
    }

    /// `-z nivel`: sin pérdida con el método y la calidad de ese nivel (0–9).
    pub fn aplicar_nivel_sin_perdida(&mut self, nivel: i32) -> bool {
        let mut c = self.a_config();
        // SAFETY: c es una configuración válida y propia.
        if unsafe { w::WebPConfigLosslessPreset(&mut c, nivel) } == 0 {
            return false;
        }
        self.sin_perdida = true;
        self.metodo = c.method;
        self.calidad = c.quality;
        true
    }

    /// Si libwebp da la configuración por buena.
    pub fn validar(&self) -> bool {
        let c = self.a_config();
        // SAFETY: c es una configuración válida y propia.
        unsafe { w::WebPValidateConfig(&c) != 0 }
    }

    /// Si para codificar hace falta ARGB en vez de YUV (`picture.use_argb`).
    pub(crate) fn usa_argb(&self) -> bool {
        self.sin_perdida
            || self.yuv_nitido
            || self.preprocesado > 0
            || self.recorte.is_some()
            || self.redimension.is_some_and(|r| (r.ancho | r.alto) > 0)
    }
}
