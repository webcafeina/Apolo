# Seguridad · Security

[Español](#español) · [English](#english)

## Español

### Cómo avisar de un fallo de seguridad

**No abras un issue público.** Usa el aviso privado de GitHub:
[Report a vulnerability](https://github.com/webcafeina/Apolo/security/advisories/new). Solo lo ve
quien mantiene Apolo.

Cuenta qué versión usas, en qué sistema, y cómo reproducirlo. Contestamos en unos días, y
publicamos el arreglo y el aviso cuando la versión corregida ya está disponible.

### Qué versiones se mantienen

Solo la última versión publicada. Apolo se actualiza solo desde la aplicación; los arreglos de
seguridad salen como una versión nueva.

### Cómo comprobar lo que descargas

- **Sumas SHA-256**: cada versión trae `SHA256SUMS.txt` con la suma de todos sus ficheros.
- **Procedencia**: desde la 1.0.0, cada fichero de la Release lleva una atestación firmada por
  GitHub que dice qué workflow lo compiló y desde qué commit:
  `gh attestation verify <fichero> --repo webcafeina/Apolo`.
- **Actualizaciones**: la aplicación solo instala una versión nueva si su firma minisign coincide
  con la clave pública que lleva dentro (`src-tauri/tauri.conf.json`).
- **Linux**: los repositorios APT y RPM y los `.rpm` van firmados con la clave GPG
  `6D95 3C94 D0D6 6AB6 B9B0 FA26 CA21 CD7A B7FA 57FC`
  ([apolo.asc](https://apolo.webcafeina.com/apolo.asc)).
- **macOS y Windows**: los instaladores todavía no llevan firma de Apple ni de Microsoft, por eso
  el sistema avisa al abrirlos la primera vez.

## English

### Reporting a security issue

**Please don't open a public issue.** Use GitHub's private reporting:
[Report a vulnerability](https://github.com/webcafeina/Apolo/security/advisories/new). Only the
maintainers can see it.

Tell us which version you use, on which system, and how to reproduce it. We reply within a few
days, and publish the fix and the advisory once the fixed version is available.

### Supported versions

Only the latest release. Apolo updates itself from the app; security fixes ship as a new release.

### Verifying what you download

- **SHA-256 sums**: every release includes `SHA256SUMS.txt` with the sum of all its files.
- **Provenance**: from 1.0.0 on, every file in the release has an attestation signed by GitHub
  that states which workflow built it and from which commit:
  `gh attestation verify <file> --repo webcafeina/Apolo`.
- **Updates**: the app only installs a new version if its minisign signature matches the public
  key it carries (`src-tauri/tauri.conf.json`).
- **Linux**: the APT and RPM repositories and the `.rpm` files are signed with the GPG key
  `6D95 3C94 D0D6 6AB6 B9B0 FA26 CA21 CD7A B7FA 57FC`
  ([apolo.asc](https://apolo.webcafeina.com/apolo.asc)).
- **macOS and Windows**: the installers are not signed by Apple or Microsoft yet, which is why the
  system warns the first time you open them.
