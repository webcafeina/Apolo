//! Cuando quien lee la salida la cierra antes de tiempo (`apolo … | head -1`), `apolo` acaba callado, como
//! cualquier otra orden de Unix, y no con un pánico de Rust.

#[cfg(unix)]
#[test]
fn salida_cortada_sin_panico() {
    use std::process::{Command, Stdio};

    // Una tubería con el extremo de lectura ya cerrado: la primera escritura de `apolo` falla siempre, sin
    // depender de cuándo se cierra (con `head` de verdad, depende de quién va más deprisa).
    let (lectura, escritura) = std::io::pipe().unwrap();
    drop(lectura);
    let salida = Command::new(env!("CARGO_BIN_EXE_apolo"))
        .arg("motores")
        .env("LC_ALL", "es_ES.UTF-8")
        .stdout(escritura)
        .stderr(Stdio::piped())
        .output()
        .unwrap();
    let errores = String::from_utf8_lossy(&salida.stderr);
    assert!(
        !errores.contains("panicked") && !errores.contains("Broken pipe"),
        "con la salida cerrada, apolo no debe quejarse: {errores}"
    );
    // La terminó la señal de la tubería, como a `cat` o a `cwebp`, no un error suyo.
    use std::os::unix::process::ExitStatusExt;
    assert_eq!(salida.status.signal(), Some(13), "{:?}", salida.status);
}
