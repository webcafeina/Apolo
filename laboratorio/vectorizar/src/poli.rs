//! Ajuste a partir del polígono (al estilo de PolyFit, Dominici y otros, 2020).
//!
//! El polígono afinado del contorno por debajo del píxel ya es fiel; en vez de buscar esquinas en el
//! contorno denso, se lee el polígono:
//! - un lado **largo** (≥ `LARGO` px) es una recta: se ajusta por mínimos cuadrados a sus puntos;
//! - una racha de lados **cortos** es una curva: Bézier cúbicas si ahorran nodos, si no, se queda el
//!   polígono;
//! - entre dos rectas, la esquina va al cruce de las dos; una racha corta entre dos rectas que se cruzan
//!   cerca (una punta roma o un bisel) se cambia por el cruce.

use crate::ajuste::{self, Forma, Tramo, P};

const LARGO: f64 = 4.0;
const ESQUINA: f64 = 20.0;

fn sub(a: P, b: P) -> P { (a.0 - b.0, a.1 - b.1) }
fn add(a: P, b: P) -> P { (a.0 + b.0, a.1 + b.1) }
fn mul(a: P, k: f64) -> P { (a.0 * k, a.1 * k) }
fn dot(a: P, b: P) -> f64 { a.0 * b.0 + a.1 * b.1 }
fn norm(a: P) -> P { let l = dot(a, a).sqrt().max(1e-12); (a.0 / l, a.1 / l) }
fn dist(a: P, b: P) -> f64 { dot(sub(a, b), sub(a, b)).sqrt() }

/// Douglas-Peucker que devuelve índices (polilínea abierta de `i0` a `i1`, ambos incluidos).
fn dp(p: &[P], i0: usize, i1: usize, tol: f64, out: &mut Vec<usize>) {
    let (a, b) = (p[i0], p[i1]);
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let l = (dx * dx + dy * dy).sqrt().max(1e-9);
    let (mut im, mut dm) = (i0, 0.0);
    for (i, q) in p.iter().enumerate().take(i1).skip(i0 + 1) {
        let d = ((q.0 - a.0) * dy - (q.1 - a.1) * dx).abs() / l;
        if d > dm { dm = d; im = i; }
    }
    if dm > tol {
        dp(p, i0, im, tol, out);
        out.pop();
        dp(p, im, i1, tol, out);
    } else {
        out.push(i0);
        out.push(i1);
    }
}

#[derive(Clone)]
enum Parte {
    Recta { a: usize, b: usize },
    Curva { idx: Vec<usize> },
}

/// `cob(x, y)`: la cobertura de la capa en ese punto (0 fuera, 1 dentro), para validar las puntas.
pub fn ajustar(lazo: &[P], tol_poli: f64, tol_curva: f64, cob: &dyn Fn(f64, f64) -> f32, tam: (u32, u32), sintesis: bool) -> Option<Forma> {
    let pasadas = std::env::var("APOLO_SUAVIZAR").ok().and_then(|v| v.parse().ok()).unwrap_or(5);
    let suave = crate::subpixel::suavizar(lazo, pasadas);
    let f = ajustar_sin_sintesis(&suave, tol_poli, tol_curva, cob)?;
    if !sintesis || std::env::var("APOLO_SINTESIS").map(|v| v == "0").unwrap_or(false) { return Some(f); }
    // Un polígono de solo rectas se refina por síntesis (sintesis.rs).
    if let Forma::Tramos(t) = &f {
        if t.iter().all(|s| matches!(s, Tramo::Linea(..))) {
            let mut v: Vec<P> = t.iter().map(|s| match *s { Tramo::Linea(a, _) => a, _ => unreachable!() }).collect();
            let obs = |i: i64, j: i64| cob(i as f64 + 0.5, j as f64 + 0.5) as f64;
            if crate::sintesis::refinar(&mut v, &obs, tam.0, tam.1) {
                let k = v.len();
                return Some(Forma::Tramos((0..k).map(|i| Tramo::Linea(v[i], v[(i + 1) % k])).collect()));
            }
        }
    }
    Some(f)
}

fn ajustar_sin_sintesis(lazo: &[P], tol_poli: f64, tol_curva: f64, cob: &dyn Fn(f64, f64) -> f32) -> Option<Forma> {
    let n = lazo.len();
    if n < 3 || ajuste::area(lazo).abs() < 0.8 { return None; }
    // Un círculo entero, si lo es (error ≤ 0,35 px).
    if let Some((cc, r)) = ajuste::circulo(lazo) {
        if r >= 1.5 { return Some(Forma::Circulo { c: cc, r }); }
    }
    // Una elipse entera, si lo es.
    if let Some((cc, rx, ry, ang)) = ajuste::elipse(lazo) {
        return Some(Forma::Elipse { c: cc, rx, ry, ang });
    }
    // Formas pequeñas (menos de 10 px): el polígono afinado tal cual.
    let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for q in lazo { x0 = x0.min(q.0); y0 = y0.min(q.1); x1 = x1.max(q.0); y1 = y1.max(q.1); }
    if (x1 - x0).max(y1 - y0) < 10.0 {
        let q = crate::subpixel::simplificar(lazo, 0.15);
        if q.len() < 3 { return None; }
        let k = q.len();
        return Some(Forma::Tramos((0..k).map(|i| Tramo::Linea(q[i], q[(i + 1) % k])).collect()));
    }
    // El lazo, girado para que empiece en el punto más lejano del centro (una esquina, si la hay).
    let c = (lazo.iter().map(|q| q.0).sum::<f64>() / n as f64, lazo.iter().map(|q| q.1).sum::<f64>() / n as f64);
    let ini = (0..n).max_by(|&i, &j| dist(lazo[i], c).total_cmp(&dist(lazo[j], c))).unwrap_or(0);
    let p: Vec<P> = (0..=n).map(|k| lazo[(ini + k) % n]).collect();
    // Lazo cerrado: la cuerda del primero al último mide cero, así que se parte por el más lejano.
    let lejos = (1..n).max_by(|&i, &j| dist(p[i], p[0]).total_cmp(&dist(p[j], p[0]))).unwrap_or(n / 2);
    let mut v = Vec::new();
    dp(&p, 0, lejos, tol_poli, &mut v);
    v.pop();
    dp(&p, lejos, n, tol_poli, &mut v);
    v.pop(); // el último es el primero
    let m = v.len();
    if m < 3 { return None; }
    // Giro en cada vértice del polígono, en grados.
    let giro: Vec<f64> = (0..m).map(|k| {
        let (a, b, c) = (p[v[(k + m - 1) % m]], p[v[k]], p[v[(k + 1) % m]]);
        dot(norm(sub(b, a)), norm(sub(c, b))).clamp(-1.0, 1.0).acos().to_degrees()
    }).collect();
    // Un lado es recta si es largo y al menos uno de sus extremos es una esquina de verdad. Los lados
    // largos de una curva grande (una circunferencia de radio 200 da lados de 18 px) giran poco en
    // ambos extremos: son curva.
    let largo: Vec<bool> = (0..m).map(|k| {
        dist(p[v[k]], p[v[(k + 1) % m]]) >= LARGO && (giro[k] >= ESQUINA || giro[(k + 1) % m] >= ESQUINA)
    }).collect();
    // Partes: cada lado largo es una recta; los cortos seguidos, una curva. Se empieza tras un lado largo.
    let arranque = (0..m).find(|&k| largo[k]).map(|k| (k + 1) % m).unwrap_or(0);
    let mut partes: Vec<Parte> = Vec::new();
    let mut racha: Vec<usize> = Vec::new();
    for s in 0..m {
        let k = (arranque + s) % m;
        let (a, b) = (v[k], v[(k + 1) % m]);
        let b = if b == 0 && a != 0 { n } else { b };
        if largo[k] {
            if !racha.is_empty() { partes.push(Parte::Curva { idx: std::mem::take(&mut racha) }); }
            partes.push(Parte::Recta { a, b });
        } else {
            if racha.is_empty() { racha.push(a); }
            racha.push(b);
        }
    }
    if !racha.is_empty() { partes.push(Parte::Curva { idx: racha }); }

    // Índices a puntos: tramo denso de i a j (con la vuelta en n).
    let trozo = |i: usize, j: usize| -> Vec<P> { if j >= i { p[i..=j].to_vec() } else { let mut t = p[i..=n].to_vec(); t.extend_from_slice(&p[1..=j]); t } };

    let mut tramos: Vec<Tramo> = Vec::new();
    let mut es_recta: Vec<bool> = Vec::new();
    for parte in &partes {
        match parte {
            Parte::Recta { a, b } => {
                let pts = trozo(*a, *b);
                let (pa, pb) = (pts[0], pts[pts.len() - 1]);
                let interior: Vec<P> = pts.iter().copied().filter(|q| dist(*q, pa) >= 1.0 && dist(*q, pb) >= 1.0).collect();
                // Solo los puntos fiables: a 1,5 px a cada lado, uno claramente dentro y otro fuera. Donde
                // la forma mide menos de un píxel (el brazo de una estrella), el contorno al 50 % va por el
                // centro y no por el borde; esos puntos torcerían la recta.
                let nrm = { let d = norm(sub(pb, pa)); (-d.1, d.0) };
                let fiables: Vec<P> = interior.iter().copied().filter(|q| {
                    let (c1, c2) = (cob(q.0 + 1.5 * nrm.0, q.1 + 1.5 * nrm.1), cob(q.0 - 1.5 * nrm.0, q.1 - 1.5 * nrm.1));
                    c1.max(c2) > 0.9 && c1.min(c2) < 0.1
                }).collect();
                let _ = fiables; // probado el 2026-11-09: no mejora (ADR 0038); se deja la recta con todo el interior
                let base: &[P] = if interior.len() >= 3 { &interior } else { &pts };
                let (cc, d) = ajuste::recta(base);
                let pr = |q: P| add(cc, mul(d, dot(sub(q, cc), d)));
                tramos.push(Tramo::Linea(pr(pa), pr(pb)));
                es_recta.push(true);
            }
            Parte::Curva { idx } => {
                let (i, j) = (idx[0], idx[idx.len() - 1]);
                let pts = trozo(i, j);
                let segs = idx.len() - 1;
                let mut cubicas = Vec::new();
                if pts.len() >= 4 && segs >= 3 {
                    let k = 2.min(pts.len() - 1);
                    let t1 = norm(sub(pts[k], pts[0]));
                    let t2 = norm(sub(pts[pts.len() - 1 - k], pts[pts.len() - 1]));
                    ajuste::ajustar_cubica(&pts, t1, t2, tol_curva, 0, &mut cubicas);
                }
                if std::env::var("APOLO_DEPURAR").is_ok() && !cubicas.is_empty() {
                    let mut peor: f64 = 0.0;
                    let muestras: Vec<P> = cubicas.iter().flat_map(|c| match *c {
                        Tramo::Cubica(a, b, cc, d) => (0..=2000).map(move |k| { let t = k as f64 / 2000.0; let u = 1.0 - t;
                            (u*u*u*a.0 + 3.0*u*u*t*b.0 + 3.0*u*t*t*cc.0 + t*t*t*d.0, u*u*u*a.1 + 3.0*u*u*t*b.1 + 3.0*u*t*t*cc.1 + t*t*t*d.1) }).collect::<Vec<_>>(),
                        _ => vec![],
                    }).collect();
                    for q in &pts { peor = peor.max(muestras.iter().map(|m| dist(*m, *q)).fold(f64::MAX, f64::min)); }
                    if peor > 0.4 && std::env::var("APOLO_VOLCAR").is_ok() {
                        let s: String = pts.iter().map(|q| format!("{} {}\n", q.0, q.1)).collect();
                        let _ = std::fs::write(std::env::var("APOLO_VOLCAR").unwrap(), s);
                    }
                    if peor > 0.4 { eprintln!("curva {} pts {} lados {} cúbicas, desvío {peor:.2} de {:?} a {:?}", pts.len(), segs, cubicas.len(), pts[0], pts[pts.len()-1]); }
                }
                if !cubicas.is_empty() && cubicas.len() * 2 <= segs {
                    for c in cubicas { tramos.push(c); es_recta.push(false); }
                } else {
                    for w in idx.windows(2) {
                        tramos.push(Tramo::Linea(p[w[0]], p[if w[1] == 0 { n } else { w[1] }]));
                        es_recta.push(false); // lado corto: no se afila
                    }
                }
            }
        }
    }
    afilar(&mut tramos, &mut es_recta, cob);
    if tramos.len() < 2 { return None; }
    Some(Forma::Tramos(tramos))
}

fn ini(t: &Tramo) -> P { match *t { Tramo::Linea(a, _) | Tramo::Cubica(a, ..) => a } }
fn fin(t: &Tramo) -> P { match *t { Tramo::Linea(_, b) | Tramo::Cubica(.., b) => b } }
fn mover_ini(t: &mut Tramo, q: P) {
    match t {
        Tramo::Linea(a, _) => *a = q,
        Tramo::Cubica(a, c1, ..) => { let d = sub(q, *a); *a = q; *c1 = add(*c1, d); }
    }
}
fn mover_fin(t: &mut Tramo, q: P) {
    match t {
        Tramo::Linea(_, b) => *b = q,
        Tramo::Cubica(_, _, c2, b) => { let d = sub(q, *b); *b = q; *c2 = add(*c2, d); }
    }
}

/// ¿Hay un rasgo fino entre `a` y `b`? Los píxeles a medio cubrir (ni dentro ni fuera) dicen que el
/// contorno al 50 % se ha comido algo de menos de un píxel de ancho, como el brazo de una estrella.
fn rasgo_fino(a: P, b: P, cob: &dyn Fn(f64, f64) -> f32) -> bool {
    let l = dist(a, b);
    let pasos = (l / 0.5).ceil().max(2.0) as usize;
    let mut suma = 0.0;
    let mut n = 0;
    for k in 1..pasos {
        let t = k as f64 / pasos as f64;
        if t > 0.9 { break; }
        let q = add(a, mul(sub(b, a), t));
        let c = cob(q.0, q.1) as f64;
        suma += c.min(1.0 - c);
        n += 1;
    }
    n > 0 && suma / n as f64 > 0.15
}

fn afilar(t: &mut Vec<Tramo>, recta: &mut Vec<bool>, cob: &dyn Fn(f64, f64) -> f32) {
    // 1. Rachas cortas (1 o 2 lados de < 3 px en total) entre dos rectas que se cruzan cerca: al cruce.
    let mut cambiado = true;
    while cambiado && t.len() > 3 {
        cambiado = false;
        let n = t.len();
        'busca: for i in 0..n {
            if !recta[i] { continue; }
            for largo_racha in 1..=2 {
                let j = (i + largo_racha + 1) % n;
                if !recta[j] || j == i { continue; }
                let medio: Vec<usize> = (1..=largo_racha).map(|k| (i + k) % n).collect();
                if medio.iter().any(|&k| recta[k] || matches!(t[k], Tramo::Cubica(..))) { continue; }
                let total: f64 = medio.iter().map(|&k| dist(ini(&t[k]), fin(&t[k]))).sum();
                if total >= 3.0 { continue; }
                let (Tramo::Linea(a0, a1), Tramo::Linea(b0, b1)) = (t[i], t[j]) else { continue };
                if let Some(x) = ajuste::cruce(a0, a1, b0, b1) {
                    let centro = mul(add(ini(&t[medio[0]]), fin(&t[*medio.last().unwrap()])), 0.5);
                    let d = dist(x, centro);
                    if d < 3.0 || (d < 12.0 && rasgo_fino(centro, x, cob)) {
                        mover_fin(&mut t[i], x);
                        mover_ini(&mut t[j], x);
                        let mut quitar = medio.clone();
                        quitar.sort_unstable_by(|a, b| b.cmp(a));
                        for k in quitar { t.remove(k); recta.remove(k); }
                        cambiado = true;
                        break 'busca;
                    }
                }
            }
        }
    }
    // 2. Juntas. Recta con recta: al cruce si está cerca, si no al punto medio. Con una curva o un lado
    // corto de por medio manda ese, que pasa por un punto real del contorno: solo se mueve la recta.
    // (Mover el extremo de una curva arrastra su punto de control y desplaza la curva entera.)
    let n = t.len();
    for i in 0..n {
        let j = (i + 1) % n;
        let (a1, b0) = (fin(&t[i]), ini(&t[j]));
        let mut x = if !recta[i] { a1 } else if !recta[j] { b0 } else { mul(add(a1, b0), 0.5) };
        if recta[i] && recta[j] {
            if let (Tramo::Linea(p0, p1), Tramo::Linea(q0, q1)) = (t[i], t[j]) {
                if let Some(c) = ajuste::cruce(p0, p1, q0, q1) {
                    if dist(c, a1) < 2.5 && dist(c, b0) < 2.5 { x = c; }
                }
            }
        }
        mover_fin(&mut t[i], x);
        mover_ini(&mut t[j], x);
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;
    #[test]
    fn cuadrado() {
        let mut l = Vec::new();
        for i in 0..20 { l.push((i as f64, 0.0)); }
        for i in 0..20 { l.push((20.0, i as f64)); }
        for i in 0..20 { l.push((20.0 - i as f64, 20.0)); }
        for i in 0..20 { l.push((0.0, 20.0 - i as f64)); }
        let f = ajustar(&l, 0.2, 0.3, &|_, _| 0.0, (100, 100), false);
        match f {
            Some(Forma::Tramos(t)) => { let mut s = String::new(); ajuste::d(&Forma::Tramos(t), &mut s); println!("{s}"); }
            Some(Forma::Circulo { .. }) => panic!("círculo"),
            Some(Forma::Elipse { .. }) => panic!("elipse"),
            None => panic!("nada"),
        }
    }
}
