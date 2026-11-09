//! Enlace a mano con la biblioteca de Potrace (`vendor/potrace/potracelib.h`).
//!
//! Potrace traza una máscara de un bit y devuelve curvas cerradas: las de signo «+» son contornos y las
//! de «−», agujeros. Con `fill-rule="evenodd"` se pintan bien sin mirar el signo.

use std::os::raw::{c_double, c_int, c_ulong, c_void};

#[repr(C)]
struct Progreso {
    callback: Option<extern "C" fn(c_double, *mut c_void)>,
    data: *mut c_void,
    min: c_double,
    max: c_double,
    epsilon: c_double,
}

#[repr(C)]
struct Parametros {
    turdsize: c_int,
    turnpolicy: c_int,
    alphamax: c_double,
    opticurve: c_int,
    opttolerance: c_double,
    progress: Progreso,
}

#[repr(C)]
struct Mapa {
    w: c_int,
    h: c_int,
    dy: c_int,
    map: *mut c_ulong,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Punto {
    x: c_double,
    y: c_double,
}

#[repr(C)]
struct Curva {
    n: c_int,
    tag: *mut c_int,
    c: *mut [Punto; 3],
}

#[repr(C)]
struct Camino {
    area: c_int,
    sign: c_int,
    curve: Curva,
    next: *mut Camino,
    childlist: *mut Camino,
    sibling: *mut Camino,
    privado: *mut c_void,
}

#[repr(C)]
struct Estado {
    status: c_int,
    plist: *mut Camino,
    privado: *mut c_void,
}

unsafe extern "C" {
    fn potrace_param_default() -> *mut Parametros;
    fn potrace_param_free(p: *mut Parametros);
    fn potrace_trace(p: *const Parametros, bm: *const Mapa) -> *mut Estado;
    fn potrace_state_free(st: *mut Estado);
}

const CURVETO: c_int = 1;
const BITS: usize = c_ulong::BITS as usize;

/// Ajustes de Potrace; los de por defecto son los de la herramienta `potrace`.
#[derive(Clone, Copy)]
pub struct Ajustes {
    /// Descarta manchas de hasta este área, en píxeles.
    pub turdsize: i32,
    /// Umbral de esquinas: 0 todo esquinas, 1,33 nada.
    pub alphamax: f64,
    /// Une curvas vecinas si el error no pasa de `opttolerance`.
    pub opticurve: bool,
    pub opttolerance: f64,
}

impl Default for Ajustes {
    fn default() -> Self {
        Self { turdsize: 2, alphamax: 1.0, opticurve: true, opttolerance: 0.2 }
    }
}

/// Traza una máscara (`true` = primer plano), fila 0 arriba. Devuelve el atributo `d` de un trazado
/// SVG en píxeles, o `None` si no hay nada que trazar.
pub fn trazar(mascara: &[bool], ancho: usize, alto: usize, a: Ajustes) -> Option<String> {
    assert_eq!(mascara.len(), ancho * alto);
    let dy = ancho.div_ceil(BITS).max(1);
    let mut palabras = vec![0 as c_ulong; dy * alto];
    for y in 0..alto {
        for x in 0..ancho {
            if mascara[y * ancho + x] {
                palabras[y * dy + x / BITS] |= (1 as c_ulong) << (BITS - 1 - x % BITS);
            }
        }
    }
    let mapa = Mapa { w: ancho as c_int, h: alto as c_int, dy: dy as c_int, map: palabras.as_mut_ptr() };
    let mut d = String::new();
    unsafe {
        let p = potrace_param_default();
        if p.is_null() {
            return None;
        }
        (*p).turdsize = a.turdsize;
        (*p).alphamax = a.alphamax;
        (*p).opticurve = a.opticurve as c_int;
        (*p).opttolerance = a.opttolerance;
        let st = potrace_trace(p, &mapa);
        potrace_param_free(p);
        if st.is_null() {
            return None;
        }
        if (*st).status == 0 {
            let mut c = (*st).plist;
            while !c.is_null() {
                escribir_curva(&(*c).curve, &mut d);
                c = (*c).next;
            }
        }
        potrace_state_free(st);
    }
    if d.is_empty() { None } else { Some(d) }
}

unsafe fn escribir_curva(cu: &Curva, d: &mut String) {
    use std::fmt::Write;
    let n = cu.n as usize;
    if n == 0 {
        return;
    }
    let (tags, puntos) = unsafe {
        (std::slice::from_raw_parts(cu.tag, n), std::slice::from_raw_parts(cu.c, n))
    };
    let f = |p: Punto| format!("{:.2} {:.2}", p.x, p.y);
    let _ = write!(d, "M{}", f(puntos[n - 1][2]));
    for i in 0..n {
        if tags[i] == CURVETO {
            let _ = write!(d, "C{} {} {}", f(puntos[i][0]), f(puntos[i][1]), f(puntos[i][2]));
        } else {
            let _ = write!(d, "L{}L{}", f(puntos[i][1]), f(puntos[i][2]));
        }
    }
    d.push('Z');
}
