// Compila la biblioteca de Potrace 1.16 (vendor/potrace, GPL-2.0-or-later), solo lo que hace falta para
// trazar: sin el programa `potrace` ni sus salidas.
fn main() {
    let fuentes = ["curve.c", "trace.c", "decompose.c", "potracelib.c"];
    let mut cc = cc::Build::new();
    cc.include("vendor/potrace")
        .define("VERSION", "\"1.16\"")
        .define("HAVE_INTTYPES_H", "1")
        .warnings(false)
        .opt_level(2);
    for f in fuentes {
        cc.file(format!("vendor/potrace/{f}"));
        println!("cargo:rerun-if-changed=vendor/potrace/{f}");
    }
    cc.compile("potrace");
}
