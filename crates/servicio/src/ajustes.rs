//! Los ajustes de la aplicación que se guardan entre sesiones, y la puerta de
//! las comprobaciones de versión nueva (ADR 0018).
//!
//! Es lo de Esfinge (`internal/app/preferencias.go`): la aplicación se asoma
//! cada hora, pero solo pregunta a GitHub si han pasado 24 horas desde la
//! última vez, y la fecha se apunta **antes** de salir a la red y bajo el mismo
//! cerrojo que la comprobación. Sin eso, varias vueltas del reloj se colaban a
//! la vez; y si la red falla, el turno se gasta igual, para no reintentar en
//! bucle.

use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// Cada cuánto se pregunta a GitHub, como mucho.
pub const CADA_CUANTO_SEGUNDOS: u64 = 24 * 60 * 60;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Ajustes {
    /// «Avisarme cuando haya una versión nueva». Encendido por defecto.
    pub buscar_actualizaciones: bool,
    /// Cuándo se preguntó a GitHub por última vez (segundos Unix).
    pub ultima_comprobacion: Option<u64>,
}

impl Default for Ajustes {
    fn default() -> Self {
        Ajustes {
            buscar_actualizaciones: true,
            ultima_comprobacion: None,
        }
    }
}

pub(crate) struct Almacen {
    ruta: PathBuf,
    actual: Mutex<Ajustes>,
}

fn ahora() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

impl Almacen {
    pub(crate) fn abrir(ruta: PathBuf) -> Self {
        // Un fichero roto o ausente no impide arrancar: valores por defecto.
        let actual = std::fs::read(&ruta)
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default();
        Almacen {
            ruta,
            actual: Mutex::new(actual),
        }
    }

    fn escribir(&self, a: &Ajustes) {
        if let Some(d) = self.ruta.parent() {
            let _ = std::fs::create_dir_all(d);
        }
        if let Ok(json) = serde_json::to_vec_pretty(a) {
            let _ = std::fs::write(&self.ruta, json);
        }
    }

    pub(crate) fn leer(&self) -> Ajustes {
        self.actual.lock().unwrap().clone()
    }

    pub(crate) fn buscar_actualizaciones(&self, si: bool) -> Ajustes {
        let mut a = self.actual.lock().unwrap();
        a.buscar_actualizaciones = si;
        self.escribir(&a);
        a.clone()
    }

    /// Si toca preguntar a GitHub ahora; si toca, apunta la hora ya. `forzar`
    /// es «Buscar ahora»: se salta la casilla y las 24 horas.
    pub(crate) fn reservar_comprobacion(&self, forzar: bool) -> bool {
        self.reservar_en(forzar, ahora())
    }

    fn reservar_en(&self, forzar: bool, ahora: u64) -> bool {
        let mut a = self.actual.lock().unwrap();
        if !forzar {
            if !a.buscar_actualizaciones {
                return false;
            }
            if let Some(u) = a.ultima_comprobacion
                && ahora.saturating_sub(u) < CADA_CUANTO_SEGUNDOS
            {
                return false;
            }
        }
        a.ultima_comprobacion = Some(ahora);
        self.escribir(&a);
        true
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn almacen(nombre: &str) -> Almacen {
        let r = std::env::temp_dir().join(format!(
            "apolo-ajustes-{nombre}-{}.json",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&r);
        Almacen::abrir(r)
    }

    #[test]
    fn una_vez_al_dia() {
        let a = almacen("dia");
        assert!(a.reservar_en(false, 1_000_000));
        assert!(!a.reservar_en(false, 1_000_000 + 3600));
        assert!(!a.reservar_en(false, 1_000_000 + CADA_CUANTO_SEGUNDOS - 1));
        assert!(a.reservar_en(false, 1_000_000 + CADA_CUANTO_SEGUNDOS));
    }

    #[test]
    fn apagado_no_pregunta_pero_buscar_ahora_si() {
        let a = almacen("apagado");
        a.buscar_actualizaciones(false);
        assert!(!a.reservar_en(false, 5));
        assert!(a.reservar_en(true, 5));
        assert_eq!(a.leer().ultima_comprobacion, Some(5));
    }

    #[test]
    fn se_guarda_entre_sesiones() {
        let a = almacen("guarda");
        a.buscar_actualizaciones(false);
        a.reservar_en(true, 42);
        let b = Almacen::abrir(a.ruta.clone());
        assert_eq!(
            b.leer(),
            Ajustes {
                buscar_actualizaciones: false,
                ultima_comprobacion: Some(42)
            }
        );
    }
}
