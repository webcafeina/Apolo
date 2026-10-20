//! El banco del Estudio (entrega 8b, scripts/banco.sh): lo que hace la ventana con
//! una foto, con el mismo `Servicio` y sin la interfaz. Abrir, los píxeles del
//! original, la vista previa en cada formato con sus píxeles, medir y los dos mapas.
//!
//!     cargo run --release -p apolo-servicio --example banco_estudio -- foto.jpg
//!
//! De cada paso, el tiempo y el pico de memoria del proceso. El pico se pone a
//! cero antes de cada paso (`/proc/self/clear_refs`), así que solo sale en Linux;
//! en otros sistemas, «—», y el pico del proceso entero lo da `/usr/bin/time`.

use std::path::PathBuf;
use std::time::Instant;

use apolo_nucleo::medir::TipoMapa;
use apolo_nucleo::salida::{Ajuste, FormatoSalida};
use apolo_servicio::Servicio;

fn main() {
    let foto = PathBuf::from(std::env::args().nth(1).expect("uso: banco_estudio <foto>"));
    let config = std::env::temp_dir().join(format!("apolo-banco-{}", std::process::id()));
    let s = Servicio::nuevo(config.clone());
    let datos = std::fs::read(&foto).expect("no se puede leer la foto");

    let info = paso("abrir", || s.abrir_bytes("foto", datos).expect("abrir"));
    println!(
        "# {} × {} ({:.1} MP)",
        info.ancho,
        info.alto,
        (info.ancho * info.alto) as f64 / 1e6
    );
    let id = info.id;
    paso("original (píxeles)", || {
        s.pixeles_original(id, false).expect("original").2.len()
    });

    let mut generacion = 0;
    for formato in [
        FormatoSalida::Webp,
        FormatoSalida::Jpeg,
        FormatoSalida::Avif,
        FormatoSalida::Jxl,
        FormatoSalida::Png,
    ] {
        generacion += 1;
        let ajuste = Ajuste {
            formato,
            ..Ajuste::default()
        };
        let g = generacion;
        paso(&format!("vista previa {}", formato.extension()), || {
            s.codificar(id, &ajuste, 0, g).expect("codificar");
            s.pixeles_resultado(id, 0).expect("resultado").2.len()
        });
    }
    // Se mide y se pintan los mapas sobre la última vista previa, como la ventana.
    paso("medir", || s.medir(id, 0, generacion).expect("medir"));
    for (nombre, tipo) in [
        ("mapa de diferencias", TipoMapa::Diferencia),
        ("mapa de estructura", TipoMapa::Estructura),
    ] {
        paso(nombre, || {
            s.pixeles_mapa(id, 0, tipo).expect("mapa").2.len()
        });
    }
    // El pico de toda la sesión es el mayor de los pasos: /usr/bin/time solo ve el
    // del último, porque cada paso lo pone a cero.
    println!("pico de la sesión\t\t{:.0}", PICO.with(|p| p.get()));
    s.cerrar(id);
    let _ = std::fs::remove_dir_all(config);
}

/// Hace un paso y escribe su línea: nombre, segundos y pico de memoria en MB.
fn paso<T>(nombre: &str, f: impl FnOnce() -> T) -> T {
    poner_pico_a_cero();
    let t = Instant::now();
    let r = f();
    let segundos = t.elapsed().as_secs_f64();
    let mb = pico_mb();
    if let Some(m) = mb {
        PICO.with(|p| p.set(p.get().max(m)));
    }
    let pico = mb.map_or("—".into(), |m| format!("{m:.0}"));
    println!("{nombre}\t{segundos:.2}\t{pico}");
    r
}

thread_local! {
    static PICO: std::cell::Cell<f64> = const { std::cell::Cell::new(0.0) };
}

fn poner_pico_a_cero() {
    let _ = std::fs::write("/proc/self/clear_refs", "5");
}

/// VmHWM: el pico de memoria residente desde la última puesta a cero.
fn pico_mb() -> Option<f64> {
    let estado = std::fs::read_to_string("/proc/self/status").ok()?;
    let linea = estado.lines().find(|l| l.starts_with("VmHWM:"))?;
    let kb: f64 = linea.split_whitespace().nth(1)?.parse().ok()?;
    Some(kb / 1024.0)
}
