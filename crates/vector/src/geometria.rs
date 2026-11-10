//! Puntos y operaciones de vectores en el plano.

pub type P = (f64, f64);

pub fn sub(a: P, b: P) -> P {
    (a.0 - b.0, a.1 - b.1)
}
pub fn add(a: P, b: P) -> P {
    (a.0 + b.0, a.1 + b.1)
}
pub fn mul(a: P, k: f64) -> P {
    (a.0 * k, a.1 * k)
}
pub fn dot(a: P, b: P) -> f64 {
    a.0 * b.0 + a.1 * b.1
}
pub fn len(a: P) -> f64 {
    dot(a, a).sqrt()
}
pub fn norm(a: P) -> P {
    let l = len(a).max(1e-12);
    (a.0 / l, a.1 / l)
}
pub fn dist(a: P, b: P) -> f64 {
    len(sub(a, b))
}

/// Área con signo de un polígono cerrado.
pub fn area(p: &[P]) -> f64 {
    let n = p.len();
    (0..n)
        .map(|i| {
            let (a, b) = (p[i], p[(i + 1) % n]);
            a.0 * b.1 - b.0 * a.1
        })
        .sum::<f64>()
        / 2.0
}

/// Cruce de las rectas a0→a1 y b0→b1 (infinitas).
pub fn cruce(a0: P, a1: P, b0: P, b1: P) -> Option<P> {
    let (r, s) = (sub(a1, a0), sub(b1, b0));
    let den = r.0 * s.1 - r.1 * s.0;
    if den.abs() < 1e-9 {
        return None;
    }
    let t = ((b0.0 - a0.0) * s.1 - (b0.1 - a0.1) * s.0) / den;
    Some(add(a0, mul(r, t)))
}
