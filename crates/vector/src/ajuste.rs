//! Ajustes geométricos: rectas por mínimos cuadrados, Bézier cúbicas (Schneider, Graphics Gems 1990),
//! círculos (Kåsa) y elipses (Halíř y Flusser, 1998).

use crate::geometria::{P, add, dist, dot, mul, norm, sub};
use crate::modelo::Tramo;

/// Recta por mínimos cuadrados totales: punto medio y dirección.
pub fn recta(p: &[P]) -> (P, P) {
    let n = p.len() as f64;
    let c = (
        p.iter().map(|q| q.0).sum::<f64>() / n,
        p.iter().map(|q| q.1).sum::<f64>() / n,
    );
    let (mut sxx, mut sxy, mut syy) = (0.0, 0.0, 0.0);
    for q in p {
        let d = sub(*q, c);
        sxx += d.0 * d.0;
        sxy += d.0 * d.1;
        syy += d.1 * d.1;
    }
    let th = 0.5 * (2.0 * sxy).atan2(sxx - syy);
    (c, (th.cos(), th.sin()))
}

fn bezier(b: &[P; 4], t: f64) -> P {
    let u = 1.0 - t;
    add(
        add(mul(b[0], u * u * u), mul(b[1], 3.0 * u * u * t)),
        add(mul(b[2], 3.0 * u * t * t), mul(b[3], t * t * t)),
    )
}

fn parametros(p: &[P]) -> Vec<f64> {
    let mut u = vec![0.0; p.len()];
    for i in 1..p.len() {
        u[i] = u[i - 1] + dist(p[i], p[i - 1]);
    }
    let total = u[p.len() - 1].max(1e-12);
    u.iter().map(|x| x / total).collect()
}

fn generar(p: &[P], u: &[f64], t1: P, t2: P) -> [P; 4] {
    let (a, b) = (p[0], p[p.len() - 1]);
    let (mut c00, mut c01, mut c11, mut x0, mut x1) = (0.0, 0.0, 0.0, 0.0, 0.0);
    for (i, &t) in u.iter().enumerate() {
        let s = 1.0 - t;
        let (b0, b1, b2, b3) = (s * s * s, 3.0 * s * s * t, 3.0 * s * t * t, t * t * t);
        let (a1, a2) = (mul(t1, b1), mul(t2, b2));
        c00 += dot(a1, a1);
        c01 += dot(a1, a2);
        c11 += dot(a2, a2);
        let tmp = sub(p[i], add(mul(a, b0 + b1), mul(b, b2 + b3)));
        x0 += dot(a1, tmp);
        x1 += dot(a2, tmp);
    }
    let det = c00 * c11 - c01 * c01;
    let seg = dist(a, b);
    let (mut al, mut ar) = if det.abs() > 1e-12 {
        ((x0 * c11 - x1 * c01) / det, (c00 * x1 - c01 * x0) / det)
    } else {
        (seg / 3.0, seg / 3.0)
    };
    if al < 1e-6 * seg || ar < 1e-6 * seg || al > 2.0 * seg || ar > 2.0 * seg {
        al = seg / 3.0;
        ar = seg / 3.0;
    }
    [a, add(a, mul(t1, al)), add(b, mul(t2, ar)), b]
}

fn error_max(p: &[P], b: &[P; 4], u: &[f64]) -> (f64, usize) {
    let mut m = (0.0, p.len() / 2);
    for i in 1..p.len() - 1 {
        let d = dist(bezier(b, u[i]), p[i]);
        if d > m.0 {
            m = (d, i);
        }
    }
    m
}

fn newton(b: &[P; 4], q: P, t: f64) -> f64 {
    let d1 = [
        mul(sub(b[1], b[0]), 3.0),
        mul(sub(b[2], b[1]), 3.0),
        mul(sub(b[3], b[2]), 3.0),
    ];
    let d2 = [mul(sub(d1[1], d1[0]), 2.0), mul(sub(d1[2], d1[1]), 2.0)];
    let s = 1.0 - t;
    let q1 = add(
        add(mul(d1[0], s * s), mul(d1[1], 2.0 * s * t)),
        mul(d1[2], t * t),
    );
    let q2 = add(mul(d2[0], s), mul(d2[1], t));
    let diff = sub(bezier(b, t), q);
    let num = dot(diff, q1);
    let den = dot(q1, q1) + dot(diff, q2);
    if den.abs() < 1e-12 {
        t
    } else {
        (t - num / den).clamp(0.0, 1.0)
    }
}

pub fn ajustar_cubica(p: &[P], t1: P, t2: P, tol: f64, prof: u32, out: &mut Vec<Tramo>) {
    let n = p.len();
    if n < 2 {
        return;
    }
    if n == 2 || prof > 12 {
        let d = dist(p[0], p[n - 1]) / 3.0;
        out.push(Tramo::Cubica(
            p[0],
            add(p[0], mul(t1, d)),
            add(p[n - 1], mul(t2, d)),
            p[n - 1],
        ));
        return;
    }
    let mut u = parametros(p);
    let mut b = generar(p, &u, t1, t2);
    let (mut err, mut corte) = error_max(p, &b, &u);
    if err <= tol {
        out.push(Tramo::Cubica(b[0], b[1], b[2], b[3]));
        return;
    }
    if err <= tol * 4.0 {
        for _ in 0..6 {
            u = u.iter().zip(p).map(|(&t, &q)| newton(&b, q, t)).collect();
            b = generar(p, &u, t1, t2);
            (err, corte) = error_max(p, &b, &u);
            if err <= tol {
                out.push(Tramo::Cubica(b[0], b[1], b[2], b[3]));
                return;
            }
        }
    }
    let corte = corte.clamp(1, n - 2);
    // Si en el corte hay un giro brusco (una esquina que se le escapó a la detección), se parte como
    // esquina: cada mitad con su tangente. Si no, con la tangente continua.
    let k = 2.min(corte).min(n - 1 - corte);
    let (entra, sale) = (
        norm(sub(p[corte], p[corte - k])),
        norm(sub(p[corte + k], p[corte])),
    );
    if dot(entra, sale) < 35f64.to_radians().cos() {
        ajustar_cubica(&p[..=corte], t1, mul(entra, -1.0), tol, prof + 1, out);
        ajustar_cubica(&p[corte..], sale, t2, tol, prof + 1, out);
    } else {
        let tc = norm(sub(p[corte - 1], p[corte + 1]));
        ajustar_cubica(&p[..=corte], t1, tc, tol, prof + 1, out);
        ajustar_cubica(&p[corte..], mul(tc, -1.0), t2, tol, prof + 1, out);
    }
}

/// Círculo por mínimos cuadrados (Kåsa); `None` si algún punto se aparta más de 0,35 px.
pub fn circulo(p: &[P]) -> Option<(P, f64)> {
    let n = p.len() as f64;
    let (mx, my) = (
        p.iter().map(|q| q.0).sum::<f64>() / n,
        p.iter().map(|q| q.1).sum::<f64>() / n,
    );
    let (mut suu, mut svv, mut suv, mut suuu, mut svvv, mut suvv, mut svuu) =
        (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    for q in p {
        let (u, v) = (q.0 - mx, q.1 - my);
        suu += u * u;
        svv += v * v;
        suv += u * v;
        suuu += u * u * u;
        svvv += v * v * v;
        suvv += u * v * v;
        svuu += v * u * u;
    }
    let det = suu * svv - suv * suv;
    if det.abs() < 1e-9 {
        return None;
    }
    let (b1, b2) = (0.5 * (suuu + suvv), 0.5 * (svvv + svuu));
    let uc = (b1 * svv - b2 * suv) / det;
    let vc = (suu * b2 - suv * b1) / det;
    let c = (uc + mx, vc + my);
    let r = (uc * uc + vc * vc + (suu + svv) / n).sqrt();
    let err = p
        .iter()
        .map(|q| (dist(*q, c) - r).abs())
        .fold(0.0, f64::max);
    if err <= 0.35 { Some((c, r)) } else { None }
}

/// Elipse por mínimos cuadrados (Halíř y Flusser, 1998, la versión estable del método de Fitzgibbon).
/// `None` si no es una elipse o algún punto se aparta más de 0,35 px.
pub fn elipse(p: &[P]) -> Option<(P, f64, f64, f64)> {
    let n = p.len() as f64;
    if p.len() < 8 {
        return None;
    }
    let (mx, my) = (
        p.iter().map(|q| q.0).sum::<f64>() / n,
        p.iter().map(|q| q.1).sum::<f64>() / n,
    );
    let s = p
        .iter()
        .map(|q| ((q.0 - mx).powi(2) + (q.1 - my).powi(2)).sqrt())
        .sum::<f64>()
        / n;
    if s < 1e-6 {
        return None;
    }
    let (mut s1, mut s2, mut s3) = ([[0.0f64; 3]; 3], [[0.0f64; 3]; 3], [[0.0f64; 3]; 3]);
    for q in p {
        let (x, y) = ((q.0 - mx) / s, (q.1 - my) / s);
        let d1 = [x * x, x * y, y * y];
        let d2 = [x, y, 1.0];
        for i in 0..3 {
            for j in 0..3 {
                s1[i][j] += d1[i] * d1[j];
                s2[i][j] += d1[i] * d2[j];
                s3[i][j] += d2[i] * d2[j];
            }
        }
    }
    // t = −S3⁻¹ · S2ᵀ ;  M = S1 + S2 · t
    let t = m3::mul(&m3::escalar(&m3::inversa(&s3)?, -1.0), &m3::traspuesta(&s2));
    let m = m3::suma(&s1, &m3::mul(&s2, &t));
    let m = [
        [m[2][0] / 2.0, m[2][1] / 2.0, m[2][2] / 2.0],
        [-m[1][0], -m[1][1], -m[1][2]],
        [m[0][0] / 2.0, m[0][1] / 2.0, m[0][2] / 2.0],
    ];
    let mut mejor: Option<[f64; 3]> = None;
    for l in m3::valores_propios(&m) {
        let mut a = m;
        for (i, fila) in a.iter_mut().enumerate() {
            fila[i] -= l;
        }
        // Vector del núcleo: el producto vectorial de dos filas independientes.
        let mut v = m3::cruz(&a[0], &a[1]);
        if m3::norma(&v) < 1e-12 {
            v = m3::cruz(&a[0], &a[2]);
        }
        if m3::norma(&v) < 1e-12 {
            v = m3::cruz(&a[1], &a[2]);
        }
        if 4.0 * v[0] * v[2] - v[1] * v[1] > 0.0 {
            mejor = Some(v);
        }
    }
    let a1 = mejor?;
    let a2 = m3::por_vector(&t, &a1);
    let (a, b, c, d, e, f) = (a1[0], a1[1], a1[2], a2[0], a2[1], a2[2]);
    let den = b * b - 4.0 * a * c;
    if den >= 0.0 {
        return None;
    }
    let (x0, y0) = ((2.0 * c * d - b * e) / den, (2.0 * a * e - b * d) / den);
    let num = 2.0 * (a * e * e + c * d * d - b * d * e + den * f);
    let raiz = ((a - c).powi(2) + b * b).sqrt();
    let rx = -(num * ((a + c) + raiz)).sqrt() / den;
    let ry = -(num * ((a + c) - raiz)).sqrt() / den;
    let ang = if b.abs() < 1e-12 {
        if a < c {
            0.0
        } else {
            std::f64::consts::FRAC_PI_2
        }
    } else {
        0.5 * (-b).atan2(c - a)
    };
    if !(rx.is_finite() && ry.is_finite()) {
        return None;
    }
    let (cx, cy, rx, ry) = (x0 * s + mx, y0 * s + my, rx * s, ry * s);
    if rx.min(ry) < 1.5 {
        return None;
    }
    // Comprobación geométrica: cada punto, a menos de 0,35 px de la elipse.
    let (sn, cs) = ang.sin_cos();
    let muestras: Vec<P> = (0..1440)
        .map(|k| {
            let u = k as f64 * std::f64::consts::TAU / 1440.0;
            let (x, y) = (rx * u.cos(), ry * u.sin());
            (cx + x * cs - y * sn, cy + x * sn + y * cs)
        })
        .collect();
    for q in p {
        let dmin = muestras
            .iter()
            .map(|m| dist(*m, *q))
            .fold(f64::MAX, f64::min);
        if dmin > 0.35 {
            return None;
        }
    }
    Some(((cx, cy), rx, ry, ang))
}

/// Álgebra de 3 × 3, lo justo para la elipse.
mod m3 {
    pub type M = [[f64; 3]; 3];
    pub fn mul(a: &M, b: &M) -> M {
        let mut r = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                r[i][j] = (0..3).map(|k| a[i][k] * b[k][j]).sum();
            }
        }
        r
    }
    pub fn suma(a: &M, b: &M) -> M {
        let mut r = *a;
        for i in 0..3 {
            for j in 0..3 {
                r[i][j] += b[i][j];
            }
        }
        r
    }
    pub fn escalar(a: &M, k: f64) -> M {
        let mut r = *a;
        for f in r.iter_mut() {
            for x in f.iter_mut() {
                *x *= k;
            }
        }
        r
    }
    pub fn traspuesta(a: &M) -> M {
        let mut r = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                r[i][j] = a[j][i];
            }
        }
        r
    }
    pub fn det(a: &M) -> f64 {
        a[0][0] * (a[1][1] * a[2][2] - a[1][2] * a[2][1])
            - a[0][1] * (a[1][0] * a[2][2] - a[1][2] * a[2][0])
            + a[0][2] * (a[1][0] * a[2][1] - a[1][1] * a[2][0])
    }
    pub fn inversa(a: &M) -> Option<M> {
        let d = det(a);
        if d.abs() < 1e-12 {
            return None;
        }
        let c = |i: usize, j: usize| {
            let f = |k: usize, l: usize| a[(i + k) % 3][(j + l) % 3];
            f(1, 1) * f(2, 2) - f(1, 2) * f(2, 1)
        };
        let mut r = [[0.0; 3]; 3];
        for (j, fila) in r.iter_mut().enumerate() {
            for (i, x) in fila.iter_mut().enumerate() {
                *x = c(i, j) / d;
            }
        }
        Some(r)
    }
    pub fn por_vector(a: &M, v: &[f64; 3]) -> [f64; 3] {
        [0, 1, 2].map(|i| (0..3).map(|k| a[i][k] * v[k]).sum())
    }
    pub fn cruz(a: &[f64; 3], b: &[f64; 3]) -> [f64; 3] {
        [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ]
    }
    pub fn norma(v: &[f64; 3]) -> f64 {
        (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
    }
    /// Los valores propios reales: raíces del polinomio característico λ³ − tr·λ² + c₂·λ − det.
    pub fn valores_propios(a: &M) -> Vec<f64> {
        let tr = a[0][0] + a[1][1] + a[2][2];
        let c2 = a[0][0] * a[1][1] - a[0][1] * a[1][0] + a[0][0] * a[2][2] - a[0][2] * a[2][0]
            + a[1][1] * a[2][2]
            - a[1][2] * a[2][1];
        raices_cubica(-tr, c2, -det(a))
    }
    /// Raíces reales de x³ + b·x² + c·x + d.
    fn raices_cubica(b: f64, c: f64, d: f64) -> Vec<f64> {
        let (p, q) = (c - b * b / 3.0, 2.0 * b * b * b / 27.0 - b * c / 3.0 + d);
        let desplaza = -b / 3.0;
        let disc = q * q / 4.0 + p * p * p / 27.0;
        if disc > 1e-14 {
            let s = disc.sqrt();
            vec![(-q / 2.0 + s).cbrt() + (-q / 2.0 - s).cbrt() + desplaza]
        } else if p.abs() < 1e-14 {
            vec![desplaza]
        } else {
            let r = (-p / 3.0).sqrt();
            let phi = ((-q / 2.0) / (r * r * r)).clamp(-1.0, 1.0).acos();
            (0..3)
                .map(|k| {
                    2.0 * r * ((phi + 2.0 * std::f64::consts::PI * k as f64) / 3.0).cos() + desplaza
                })
                .collect()
        }
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn arco_en_un_tramo() {
        let (c, r) = ((0.0, 0.0), 100.0);
        let pts: Vec<P> = (0..=157)
            .map(|i| {
                let a = i as f64 / 100.0;
                (c.0 + r * a.cos(), c.1 + r * a.sin())
            })
            .collect();
        let n = pts.len();
        let mut out = Vec::new();
        ajustar_cubica(
            &pts,
            norm(sub(pts[2], pts[0])),
            norm(sub(pts[n - 3], pts[n - 1])),
            0.3,
            0,
            &mut out,
        );
        let mut peor: f64 = 0.0;
        for s in &out {
            if let Tramo::Cubica(a, b, cc, d) = *s {
                for i in 0..=200 {
                    peor = peor.max((dist(bezier(&[a, b, cc, d], i as f64 / 200.0), c) - r).abs());
                }
            }
        }
        assert!(peor < 0.4, "desvío {peor}");
    }

    #[test]
    fn elipse_girada() {
        let (cx, cy, rx, ry, ang) = (100.0, 80.0, 40.0, 15.0, 0.6f64);
        let (s, c) = ang.sin_cos();
        let p: Vec<P> = (0..200)
            .map(|k| {
                let u = k as f64 * std::f64::consts::TAU / 200.0;
                let (x, y) = (rx * u.cos(), ry * u.sin());
                (cx + x * c - y * s, cy + x * s + y * c)
            })
            .collect();
        let ((x, y), a, b, _) = elipse(&p).expect("elipse");
        assert!(
            (x - cx).abs() < 0.01
                && (y - cy).abs() < 0.01
                && (a - rx).abs() < 0.01
                && (b - ry).abs() < 0.01,
            "{x} {y} {a} {b}"
        );
        let cuadrado: Vec<P> = (0..80)
            .map(|i| match i / 20 {
                0 => (i as f64, 0.0),
                1 => (20.0, (i - 20) as f64),
                2 => (20.0 - (i - 40) as f64, 20.0),
                _ => (0.0, 20.0 - (i - 60) as f64),
            })
            .collect();
        assert!(elipse(&cuadrado).is_none());
    }

    #[test]
    fn circulo_exacto() {
        let p: Vec<P> = (0..90)
            .map(|k| {
                let u = k as f64 * std::f64::consts::TAU / 90.0;
                (50.0 + 20.0 * u.cos(), 40.0 + 20.0 * u.sin())
            })
            .collect();
        let ((x, y), r) = circulo(&p).expect("círculo");
        assert!((x - 50.0).abs() < 1e-6 && (y - 40.0).abs() < 1e-6 && (r - 20.0).abs() < 1e-6);
    }
}
