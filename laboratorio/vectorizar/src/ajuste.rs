//! Del contorno denso (un punto por píxel, con precisión por debajo del píxel) a geometría limpia:
//! esquinas, rectas, Bézier cúbicas y círculos.
//!
//! 1. **Esquinas**: donde el contorno gira más de `angulo` grados en una ventana de ~3 px.
//! 2. **Tramos** entre esquinas: una recta si cabe en `tol`, si no, Bézier cúbicas (algoritmo de
//!    Schneider, Graphics Gems 1990) partiendo hasta que el error no pase de `tol`.
//! 3. **Esquinas exactas**: entre dos rectas, la esquina pasa al cruce de las dos; un tramo muy corto
//!    entre dos rectas largas (la punta que el contorno al 50 % se come) se quita y las rectas se
//!    prolongan hasta cruzarse. Es lo que recupera las puntas de las estrellas.
//! 4. **Círculos**: un lazo sin esquinas que se ajusta a un círculo con error pequeño sale como círculo.

pub type P = (f64, f64);

#[derive(Clone, Copy, Debug)]
pub enum Tramo {
    Linea(P, P),
    Cubica(P, P, P, P),
}

impl Tramo {
    fn ini(&self) -> P {
        match *self { Tramo::Linea(a, _) | Tramo::Cubica(a, ..) => a }
    }
    fn fin(&self) -> P {
        match *self { Tramo::Linea(_, b) | Tramo::Cubica(.., b) => b }
    }
    fn set_ini(&mut self, p: P) {
        match self { Tramo::Linea(a, _) | Tramo::Cubica(a, ..) => *a = p }
    }
    fn set_fin(&mut self, p: P) {
        match self { Tramo::Linea(_, b) => *b = p, Tramo::Cubica(.., b) => *b = p }
    }
}

pub enum Forma {
    Circulo { c: P, r: f64 },
    /// Elipse de semiejes `rx`, `ry`, girada `ang` radianes.
    Elipse { c: P, rx: f64, ry: f64, ang: f64 },
    Tramos(Vec<Tramo>),
}

#[derive(Clone, Copy)]
pub struct Opciones {
    pub tol: f64,
    pub angulo: f64,
    pub circulos: bool,
    pub esquinas_exactas: bool,
}

fn sub(a: P, b: P) -> P { (a.0 - b.0, a.1 - b.1) }
fn add(a: P, b: P) -> P { (a.0 + b.0, a.1 + b.1) }
fn mul(a: P, k: f64) -> P { (a.0 * k, a.1 * k) }
fn dot(a: P, b: P) -> f64 { a.0 * b.0 + a.1 * b.1 }
fn len(a: P) -> f64 { dot(a, a).sqrt() }
fn norm(a: P) -> P { let l = len(a).max(1e-12); (a.0 / l, a.1 / l) }
fn dist(a: P, b: P) -> f64 { len(sub(a, b)) }

pub fn ajustar(lazo: &[P], o: Opciones) -> Option<Forma> {
    let n = lazo.len();
    if n < 3 { return None; }
    if area(lazo).abs() < 0.8 { return None; }
    // Formas pequeñas (menos de 10 px de lado): no hay sitio para ver esquinas ni curvas; el polígono
    // afinado es lo más fiel.
    let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for q in lazo { x0 = x0.min(q.0); y0 = y0.min(q.1); x1 = x1.max(q.0); y1 = y1.max(q.1); }
    if (x1 - x0).max(y1 - y0) < 10.0 {
        let p = crate::subpixel::simplificar(lazo, 0.15);
        if p.len() < 3 { return None; }
        let n = p.len();
        return Some(Forma::Tramos((0..n).map(|i| Tramo::Linea(p[i], p[(i + 1) % n])).collect()));
    }
    let esquinas = esquinas(lazo, o.angulo);
    if o.circulos && esquinas.is_empty() {
        if let Some((c, r)) = circulo(lazo) {
            if r >= 1.5 { return Some(Forma::Circulo { c, r }); }
        }
    }
    let mut tramos: Vec<Tramo> = Vec::new();
    if esquinas.is_empty() {
        // Lazo suave: se parte en cuatro, con las tangentes continuas en los cortes.
        let cortes: Vec<usize> = (0..4).map(|k| k * n / 4).collect();
        for k in 0..4 {
            let (a, b) = (cortes[k], cortes[(k + 1) % 4]);
            let pts = trozo(lazo, a, b);
            let t1 = norm(sub(lazo[(a + 1) % n], lazo[(a + n - 1) % n]));
            let t2 = norm(sub(lazo[(b + n - 1) % n], lazo[(b + 1) % n]));
            ajustar_cubica(&pts, t1, t2, o.tol, 0, &mut tramos);
        }
    } else {
        for (k, &a) in esquinas.iter().enumerate() {
            let b = esquinas[(k + 1) % esquinas.len()];
            let pts = trozo(lazo, a, b);
            tramos.extend(tramo(&pts, o.tol));
        }
        if o.esquinas_exactas {
            afilar(&mut tramos);
        }
    }
    if tramos.is_empty() { None } else { Some(Forma::Tramos(tramos)) }
}

pub fn area(p: &[P]) -> f64 {
    let n = p.len();
    (0..n).map(|i| { let (a, b) = (p[i], p[(i + 1) % n]); a.0 * b.1 - b.0 * a.1 }).sum::<f64>() / 2.0
}

/// Los puntos de `a` a `b` (incluidos), dando la vuelta si hace falta.
fn trozo(p: &[P], a: usize, b: usize) -> Vec<P> {
    let n = p.len();
    let mut v = Vec::new();
    let mut i = a;
    loop {
        v.push(p[i]);
        if i == b && v.len() > 1 { break; }
        i = (i + 1) % n;
        if v.len() > n + 1 { break; }
    }
    v
}

/// Índices de las esquinas: giro mayor que `angulo` entre los vecinos a ~3 px, máximo local.
fn esquinas(p: &[P], angulo: f64) -> Vec<usize> {
    let n = p.len();
    if n < 8 { return Vec::new(); }
    let r = 3.0;
    let vecino = |i: usize, adelante: bool| -> P {
        let mut j = i;
        for _ in 0..n {
            j = if adelante { (j + 1) % n } else { (j + n - 1) % n };
            if dist(p[j], p[i]) >= r { break; }
        }
        p[j]
    };
    let giro: Vec<f64> = (0..n).map(|i| {
        let (a, b) = (norm(sub(p[i], vecino(i, false))), norm(sub(vecino(i, true), p[i])));
        dot(a, b).clamp(-1.0, 1.0).acos().to_degrees()
    }).collect();
    let mut v = Vec::new();
    for i in 0..n {
        if giro[i] < angulo { continue; }
        // máximo en ±r de arco
        let mut es_max = true;
        let mut j = i;
        for paso in [true, false] {
            j = i;
            loop {
                j = if paso { (j + 1) % n } else { (j + n - 1) % n };
                if j == i || dist(p[j], p[i]) > r { break; }
                if giro[j] > giro[i] || (giro[j] == giro[i] && j < i) { es_max = false; }
            }
        }
        let _ = j;
        if es_max { v.push(i); }
    }
    v
}

/// Recta por mínimos cuadrados totales: punto medio y dirección.
pub fn recta(p: &[P]) -> (P, P) {
    let n = p.len() as f64;
    let c = (p.iter().map(|q| q.0).sum::<f64>() / n, p.iter().map(|q| q.1).sum::<f64>() / n);
    let (mut sxx, mut sxy, mut syy) = (0.0, 0.0, 0.0);
    for q in p {
        let d = sub(*q, c);
        sxx += d.0 * d.0; sxy += d.0 * d.1; syy += d.1 * d.1;
    }
    let th = 0.5 * (2.0 * sxy).atan2(sxx - syy);
    (c, (th.cos(), th.sin()))
}

fn proyectar(q: P, c: P, d: P) -> P { add(c, mul(d, dot(sub(q, c), d))) }

fn tramo(p: &[P], tol: f64) -> Vec<Tramo> {
    let n = p.len();
    if n < 2 { return Vec::new(); }
    let (a, b) = (p[0], p[n - 1]);
    // ¿Recta? Por mínimos cuadrados sobre el interior del tramo: el contorno al 50 % bisela las
    // esquinas medio píxel, y ese bisel no debe decidir. La esquina la pone luego el cruce de rectas.
    let interior: Vec<P> = p.iter().copied().filter(|q| dist(*q, a) >= 1.5 && dist(*q, b) >= 1.5).collect();
    let base: &[P] = if interior.len() >= 3 { &interior } else { p };
    let (c, d) = recta(base);
    let err = base.iter().map(|q| { let e = sub(*q, c); (e.0 * d.1 - e.1 * d.0).abs() }).fold(0.0, f64::max);
    if err <= tol || n <= 3 {
        return vec![Tramo::Linea(proyectar(a, c, d), proyectar(b, c, d))];
    }
    if std::env::var("APOLO_SIN_CURVAS").is_ok() {
        let q = crate::subpixel::simplificar_abierto(p, 0.15);
        return q.windows(2).map(|w| Tramo::Linea(w[0], w[1])).collect();
    }
    let k = 3.min(n - 1);
    let t1 = norm(sub(p[k], a));
    let t2 = norm(sub(p[n - 1 - k], b));
    let mut out = Vec::new();
    ajustar_cubica(p, t1, t2, tol, 0, &mut out);
    // Una Bézier casi recta se guarda como recta (el afilado podrá cruzarla con su vecina).
    out.into_iter().map(|s| match s {
        Tramo::Cubica(a, c1, c2, b) => {
            let (dx, dy) = (b.0 - a.0, b.1 - a.1);
            let l = (dx * dx + dy * dy).sqrt();
            let d = |q: P| ((q.0 - a.0) * dy - (q.1 - a.1) * dx).abs() / l.max(1e-9);
            if l > 1.0 && d(c1) <= tol && d(c2) <= tol { Tramo::Linea(a, b) } else { s }
        }
        _ => s,
    }).collect()
}

// --- Schneider ---

fn bezier(b: &[P; 4], t: f64) -> P {
    let u = 1.0 - t;
    add(add(mul(b[0], u * u * u), mul(b[1], 3.0 * u * u * t)), add(mul(b[2], 3.0 * u * t * t), mul(b[3], t * t * t)))
}

fn parametros(p: &[P]) -> Vec<f64> {
    let mut u = vec![0.0; p.len()];
    for i in 1..p.len() { u[i] = u[i - 1] + dist(p[i], p[i - 1]); }
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
        c00 += dot(a1, a1); c01 += dot(a1, a2); c11 += dot(a2, a2);
        let tmp = sub(p[i], add(mul(a, b0 + b1), mul(b, b2 + b3)));
        x0 += dot(a1, tmp); x1 += dot(a2, tmp);
    }
    let det = c00 * c11 - c01 * c01;
    let seg = dist(a, b);
    let (mut al, mut ar) = if det.abs() > 1e-12 { ((x0 * c11 - x1 * c01) / det, (c00 * x1 - c01 * x0) / det) } else { (seg / 3.0, seg / 3.0) };
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
        if d > m.0 { m = (d, i); }
    }
    m
}

fn newton(b: &[P; 4], q: P, t: f64) -> f64 {
    let d1 = [mul(sub(b[1], b[0]), 3.0), mul(sub(b[2], b[1]), 3.0), mul(sub(b[3], b[2]), 3.0)];
    let d2 = [mul(sub(d1[1], d1[0]), 2.0), mul(sub(d1[2], d1[1]), 2.0)];
    let s = 1.0 - t;
    let q1 = add(add(mul(d1[0], s * s), mul(d1[1], 2.0 * s * t)), mul(d1[2], t * t));
    let q2 = add(mul(d2[0], s), mul(d2[1], t));
    let diff = sub(bezier(b, t), q);
    let num = dot(diff, q1);
    let den = dot(q1, q1) + dot(diff, q2);
    if den.abs() < 1e-12 { t } else { (t - num / den).clamp(0.0, 1.0) }
}

pub fn ajustar_cubica(p: &[P], t1: P, t2: P, tol: f64, prof: u32, out: &mut Vec<Tramo>) {
    let n = p.len();
    if n < 2 { return; }
    if n == 2 || prof > 12 {
        let d = dist(p[0], p[n - 1]) / 3.0;
        out.push(Tramo::Cubica(p[0], add(p[0], mul(t1, d)), add(p[n - 1], mul(t2, d)), p[n - 1]));
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
    let (entra, sale) = (norm(sub(p[corte], p[corte - k])), norm(sub(p[corte + k], p[corte])));
    if dot(entra, sale) < 35f64.to_radians().cos() {
        ajustar_cubica(&p[..=corte], t1, mul(entra, -1.0), tol, prof + 1, out);
        ajustar_cubica(&p[corte..], sale, t2, tol, prof + 1, out);
    } else {
        let tc = norm(sub(p[corte - 1], p[corte + 1]));
        ajustar_cubica(&p[..=corte], t1, tc, tol, prof + 1, out);
        ajustar_cubica(&p[corte..], mul(tc, -1.0), t2, tol, prof + 1, out);
    }
}

// --- Esquinas exactas ---

pub fn cruce(a0: P, a1: P, b0: P, b1: P) -> Option<P> {
    let (r, s) = (sub(a1, a0), sub(b1, b0));
    let den = r.0 * s.1 - r.1 * s.0;
    if den.abs() < 1e-9 { return None; }
    let t = ((b0.0 - a0.0) * s.1 - (b0.1 - a0.1) * s.0) / den;
    Some(add(a0, mul(r, t)))
}

fn afilar(t: &mut Vec<Tramo>) {
    // 1. Quitar puntas romas: recta corta entre dos rectas más largas que se cruzan cerca.
    let mut cambiado = true;
    while cambiado && t.len() > 3 {
        cambiado = false;
        let n = t.len();
        for i in 0..n {
            let (ia, ib) = ((i + n - 1) % n, (i + 1) % n);
            if let (Tramo::Linea(s0, s1), Tramo::Linea(a0, a1), Tramo::Linea(b0, b1)) = (t[i], t[ia], t[ib]) {
                let corta = dist(s0, s1);
                if corta < 2.5 && dist(a0, a1) > corta * 1.5 && dist(b0, b1) > corta * 1.5 {
                    if let Some(x) = cruce(a0, a1, b0, b1) {
                        let medio = mul(add(s0, s1), 0.5);
                        if dist(x, medio) < limite_punta() {
                            t[ia].set_fin(x);
                            t[ib].set_ini(x);
                            t.remove(i);
                            cambiado = true;
                            break;
                        }
                    }
                }
            }
        }
    }
    // 2. Juntas entre rectas: al cruce, si está cerca.
    let n = t.len();
    for i in 0..n {
        let j = (i + 1) % n;
        if let (Tramo::Linea(a0, a1), Tramo::Linea(b0, b1)) = (t[i], t[j]) {
            if let Some(x) = cruce(a0, a1, b0, b1) {
                if dist(x, a1) < limite_junta() && dist(x, b0) < limite_junta() {
                    t[i].set_fin(x);
                    t[j].set_ini(x);
                    continue;
                }
            }
            let m = mul(add(a1, b0), 0.5);
            t[i].set_fin(m);
            t[j].set_ini(m);
        } else {
            // Recta y curva: se juntan en el punto de la curva para no dejar rendija.
            let p = match t[j] { Tramo::Cubica(..) => t[j].ini(), _ => t[i].fin() };
            t[i].set_fin(p);
            t[j].set_ini(p);
        }
    }
}

/// Círculo por mínimos cuadrados (Kåsa); `None` si algún punto se aparta más de 0,35 px.
pub fn circulo(p: &[P]) -> Option<(P, f64)> {
    let n = p.len() as f64;
    let (mx, my) = (p.iter().map(|q| q.0).sum::<f64>() / n, p.iter().map(|q| q.1).sum::<f64>() / n);
    let (mut suu, mut svv, mut suv, mut suuu, mut svvv, mut suvv, mut svuu) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    for q in p {
        let (u, v) = (q.0 - mx, q.1 - my);
        suu += u * u; svv += v * v; suv += u * v;
        suuu += u * u * u; svvv += v * v * v; suvv += u * v * v; svuu += v * u * u;
    }
    let det = suu * svv - suv * suv;
    if det.abs() < 1e-9 { return None; }
    let (b1, b2) = (0.5 * (suuu + suvv), 0.5 * (svvv + svuu));
    let uc = (b1 * svv - b2 * suv) / det;
    let vc = (suu * b2 - suv * b1) / det;
    let c = (uc + mx, vc + my);
    let r = (uc * uc + vc * vc + (suu + svv) / n).sqrt();
    let err = p.iter().map(|q| (dist(*q, c) - r).abs()).fold(0.0, f64::max);
    if err <= 0.35 { Some((c, r)) } else { None }
}

/// El atributo `d` de una forma.
pub fn d(f: &Forma, out: &mut String) {
    use std::fmt::Write;
    let r2 = |x: f64| (x * 100.0).round() / 100.0;
    match f {
        Forma::Circulo { c, r } => {
            // Cuatro cuartos de arco: dos medios arcos entre puntos opuestos son ambiguos en SVG, y con
            // el redondeo el visor puede deformarlos.
            let (cx, cy, rr) = (c.0, c.1, *r);
            let _ = write!(out, "M{} {}", r2(cx + rr), r2(cy));
            for (x, y) in [(cx, cy + rr), (cx - rr, cy), (cx, cy - rr), (cx + rr, cy)] {
                let _ = write!(out, "A{} {} 0 0 1 {} {}", r2(rr), r2(rr), r2(x), r2(y));
            }
            out.push('Z');
        }
        Forma::Elipse { c, rx, ry, ang } => {
            let (s, co) = ang.sin_cos();
            let pto = |u: f64, v: f64| (c.0 + u * co - v * s, c.1 + u * s + v * co);
            let pts = [pto(*rx, 0.0), pto(0.0, *ry), pto(-rx, 0.0), pto(0.0, -ry), pto(*rx, 0.0)];
            let _ = write!(out, "M{} {}", r2(pts[0].0), r2(pts[0].1));
            for q in &pts[1..] {
                let _ = write!(out, "A{} {} {} 0 1 {} {}", r2(*rx), r2(*ry), r2(ang.to_degrees()), r2(q.0), r2(q.1));
            }
            out.push('Z');
        }
        Forma::Tramos(t) => {
            let p0 = t[0].ini();
            let _ = write!(out, "M{} {}", r2(p0.0), r2(p0.1));
            for s in t {
                match *s {
                    Tramo::Linea(_, b) => { let _ = write!(out, "L{} {}", r2(b.0), r2(b.1)); }
                    Tramo::Cubica(_, c1, c2, b) => { let _ = write!(out, "C{} {} {} {} {} {}", r2(c1.0), r2(c1.1), r2(c2.0), r2(c2.1), r2(b.0), r2(b.1)); }
                }
            }
            out.push('Z');
        }
    }
}

fn limite_punta() -> f64 { std::env::var("APOLO_PUNTA").ok().and_then(|v| v.parse().ok()).unwrap_or(3.5) }
fn limite_junta() -> f64 { std::env::var("APOLO_JUNTA").ok().and_then(|v| v.parse().ok()).unwrap_or(2.5) }

#[cfg(test)]
mod pruebas {
    use super::*;
    fn desvio(t: &[Tramo], c: P, r: f64) -> f64 {
        let mut m: f64 = 0.0;
        for s in t {
            if let Tramo::Cubica(a, b, cc, d) = *s {
                for i in 0..=50 {
                    let q = bezier(&[a, b, cc, d], i as f64 / 50.0);
                    m = m.max((dist(q, c) - r).abs());
                }
            }
        }
        m
    }
    #[test]
    fn volcado() {
        // Reproduce una curva volcada con APOLO_VOLCAR=<fichero> (motor apolo-poli con APOLO_DEPURAR).
        let Some(s) = std::env::var("APOLO_CURVA").ok().and_then(|f| std::fs::read_to_string(f).ok()) else { return };
        let pts: Vec<P> = s.lines().map(|l| { let v: Vec<f64> = l.split(' ').map(|x| x.parse().unwrap()).collect(); (v[0], v[1]) }).collect();
        let n = pts.len();
        let t1 = norm(sub(pts[2], pts[0]));
        let t2 = norm(sub(pts[n - 3], pts[n - 1]));
        let mut out = Vec::new();
        ajustar_cubica(&pts, t1, t2, 0.3, 0, &mut out);
        let mut peor: f64 = 0.0;
        for q in &pts {
            let mut m = f64::MAX;
            for s in &out { if let Tramo::Cubica(a, b, c, d) = *s { for k in 0..=4000 { m = m.min(dist(bezier(&[a, b, c, d], k as f64 / 4000.0), *q)); } } }
            peor = peor.max(m);
        }
        println!("puntos {n} cúbicas {} desvío {peor:.3}  primeros {:?} últimos {:?}", out.len(), &pts[..3], &pts[n-3..]);
    }
    #[test]
    fn arco() {
        let (c, r) = ((0.0, 0.0), 100.0);
        let pts: Vec<P> = (0..=157).map(|i| { let a = i as f64 / 100.0; (c.0 + r * a.cos(), c.1 + r * a.sin()) }).collect();
        let n = pts.len();
        let t1 = norm(sub(pts[2], pts[0]));
        let t2 = norm(sub(pts[n - 3], pts[n - 1]));
        let mut out = Vec::new();
        ajustar_cubica(&pts, t1, t2, 0.3, 0, &mut out);
        let d = desvio(&out, c, r);
        println!("tramos {} desvío {d:.3}", out.len());
        assert!(d < 0.4, "desvío {d}");
    }
}

/// Elipse por mínimos cuadrados (Halíř y Flusser, 1998, la versión estable del método de Fitzgibbon).
/// `None` si no es una elipse o algún punto se aparta más de 0,35 px.
pub fn elipse(p: &[P]) -> Option<(P, f64, f64, f64)> {
    use nalgebra::{Matrix3, Vector3};
    let n = p.len() as f64;
    if p.len() < 8 { return None; }
    let (mx, my) = (p.iter().map(|q| q.0).sum::<f64>() / n, p.iter().map(|q| q.1).sum::<f64>() / n);
    let s = p.iter().map(|q| ((q.0 - mx).powi(2) + (q.1 - my).powi(2)).sqrt()).sum::<f64>() / n;
    if s < 1e-6 { return None; }
    let (mut s1, mut s2, mut s3) = (Matrix3::<f64>::zeros(), Matrix3::<f64>::zeros(), Matrix3::<f64>::zeros());
    for q in p {
        let (x, y) = ((q.0 - mx) / s, (q.1 - my) / s);
        let d1 = Vector3::new(x * x, x * y, y * y);
        let d2 = Vector3::new(x, y, 1.0);
        s1 += d1 * d1.transpose();
        s2 += d1 * d2.transpose();
        s3 += d2 * d2.transpose();
    }
    let t = -s3.try_inverse()? * s2.transpose();
    let m = s1 + s2 * t;
    let m = Matrix3::new(m[(2, 0)] / 2.0, m[(2, 1)] / 2.0, m[(2, 2)] / 2.0, -m[(1, 0)], -m[(1, 1)], -m[(1, 2)], m[(0, 0)] / 2.0, m[(0, 1)] / 2.0, m[(0, 2)] / 2.0);
    let mut mejor: Option<Vector3<f64>> = None;
    for l in m.complex_eigenvalues().iter() {
        if l.im.abs() > 1e-9 { continue; }
        let a = m - Matrix3::identity() * l.re;
        // vector del núcleo: el producto vectorial de dos filas independientes
        let filas = [a.row(0).transpose(), a.row(1).transpose(), a.row(2).transpose()];
        let mut v = filas[0].cross(&filas[1]);
        if v.norm() < 1e-12 { v = filas[0].cross(&filas[2]); }
        if v.norm() < 1e-12 { v = filas[1].cross(&filas[2]); }
        if 4.0 * v[0] * v[2] - v[1] * v[1] > 0.0 { mejor = Some(v); }
    }
    let a1 = mejor?;
    let a2 = t * a1;
    let (a, b, c, d, e, f) = (a1[0], a1[1], a1[2], a2[0], a2[1], a2[2]);
    let den = b * b - 4.0 * a * c;
    if den >= 0.0 { return None; }
    let (x0, y0) = ((2.0 * c * d - b * e) / den, (2.0 * a * e - b * d) / den);
    let num = 2.0 * (a * e * e + c * d * d - b * d * e + den * f);
    let raiz = ((a - c).powi(2) + b * b).sqrt();
    let rx = -(num * ((a + c) + raiz)).sqrt() / den;
    let ry = -(num * ((a + c) - raiz)).sqrt() / den;
    let ang = if b.abs() < 1e-12 { if a < c { 0.0 } else { std::f64::consts::FRAC_PI_2 } } else { 0.5 * (-b).atan2(c - a) };
    if !(rx.is_finite() && ry.is_finite()) { return None; }
    let (cx, cy, rx, ry) = (x0 * s + mx, y0 * s + my, rx * s, ry * s);
    if rx.min(ry) < 1.5 { return None; }
    // Comprobación geométrica: cada punto, a menos de 0,35 px de la elipse.
    let (sn, cs) = ang.sin_cos();
    let muestras: Vec<P> = (0..1440).map(|k| { let u = k as f64 * std::f64::consts::TAU / 1440.0; let (x, y) = (rx * u.cos(), ry * u.sin()); (cx + x * cs - y * sn, cy + x * sn + y * cs) }).collect();
    for q in p {
        let dmin = muestras.iter().map(|m| dist(*m, *q)).fold(f64::MAX, f64::min);
        if dmin > 0.35 { return None; }
    }
    Some(((cx, cy), rx, ry, ang))
}

#[cfg(test)]
mod pruebas_elipse {
    use super::*;
    #[test]
    fn elipse_girada() {
        let (cx, cy, rx, ry, ang) = (100.0, 80.0, 40.0, 15.0, 0.6f64);
        let (s, c) = ang.sin_cos();
        let p: Vec<P> = (0..200).map(|k| { let u = k as f64 * std::f64::consts::TAU / 200.0; let (x, y) = (rx * u.cos(), ry * u.sin()); (cx + x * c - y * s, cy + x * s + y * c) }).collect();
        let ((x, y), a, b, g) = elipse(&p).expect("elipse");
        println!("{x:.3} {y:.3} {a:.3} {b:.3} {g:.3}");
        assert!((x - cx).abs() < 0.01 && (y - cy).abs() < 0.01 && (a - rx).abs() < 0.01 && (b - ry).abs() < 0.01);
        let cuadrado: Vec<P> = (0..80).map(|i| match i / 20 { 0 => (i as f64, 0.0), 1 => (20.0, (i - 20) as f64), 2 => (20.0 - (i - 40) as f64, 20.0), _ => (0.0, 20.0 - (i - 60) as f64) }).collect();
        assert!(elipse(&cuadrado).is_none());
    }
}
