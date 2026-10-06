# ADR 0009 — Sin firmar, por ahora

**Fecha:** 2026-10-06 · **Estado:** aceptada · **Revisar si** se vende o los avisos ahuyentan a quien instala

## Contexto

Sin firma, macOS avisa de un «desarrollador no identificado» y Windows SmartScreen pone su pantalla
azul. Firmar cuesta 99 $ al año en Apple y unos 10 $ al mes con Azure Trusted Signing.

## Decisión

**Se arranca sin firmar, como Esfinge** (su ADR 0012), pero `publicar.yml` se escribe con los pasos
de firma ya en su sitio y desactivados mientras no existan los secretos. Activarla es crear las
cuentas y cargar los secretos, no reescribir el flujo.

La **actualización automática no depende de esto**: `tauri-plugin-updater` verifica con una clave
minisign propia, gratuita y distinta de la firma de código.

## Alternativas descartadas

- **Firmar las dos desde el principio.** Coste fijo antes de que haya nadie usándolo.

## Consecuencias

- Quien instale verá el aviso la primera vez. El README lleva cómo saltarlo.
- En Apple Silicon el binario necesita al menos firma ad-hoc; Tauri la pone.

## Verificación

Elegida con el cliente el 2026-10-06.
