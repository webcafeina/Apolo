# Trampas

Lo que costó encontrar y volvería a costar. Cada una con el síntoma tal como se ve —que es lo que se
busca cuando vuelve a pasar— y no con la causa, que es lo que no se sabe todavía.

Formato: `## Síntoma` · qué lo causa · cómo se evita · fecha.

---

## Una contraseña rompe la URL de conexión sin decir nada

`TypeError: Invalid URL`, o un fallo de autenticación que no cuadra con la contraseña del fichero. La
causa es una contraseña con `/`, `+`, `@`, `:` o `#` dentro de una URL. Se evita generándolas en
hexadecimal: `openssl rand -hex 24`. Es regla de toda la máquina (`~/.claude/CLAUDE.md`); Apolo no
tiene servidor, pero el día que tenga la actualización o una web, aplica. · 2026-10-06

## «failed to build bundler settings: invalid category»

Sale **al final** de `tauri build`, después de compilar todo, así que cuesta los minutos de la
compilación entera en cada objetivo. `bundle.category` en `tauri.conf.json` no admite cualquier
texto: es una lista cerrada con los nombres de Apple sin espacios (`GraphicsAndDesign`, no
`Graphics`). Pasó en la primera publicación, en los seis objetivos a la vez. · 2026-10-06
