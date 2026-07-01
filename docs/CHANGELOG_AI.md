# CHANGELOG_AI

Fecha: 2026-06-30
Tarea: Diagnosticar error de codigo en version lanzada
Cambios:
- Se reviso la app instalada y se detecto que `/Applications/Kuota.app` estaba desactualizada frente al bundle nuevo generado en `src-tauri/target/release/bundle/macos/Kuota.app`.
- El probe real de `codex-auth login --device-auth` en macOS devolvio `429 Too Many Requests`, explicando por que no se generaba codigo tras varios reintentos.
- macOS y Windows ahora muestran un error especifico cuando OpenAI limita temporalmente la generacion de codigos.
Archivos:
- `src-tauri/src/mac_codex_login.rs`
- `src-tauri/src/windows_codex_login.rs`
- `docs/CHANGELOG_AI.md`
Validación:
- `npm run build` pasó.
- `cargo fmt` pasó.
- `cargo check` pasó.
- `cargo check` en `C:\Kuota-windows-test\src-tauri` devolvio exit 0 en la VM Windows.
- `npm run tauri:build -- --bundles app` genero `Kuota.app` correctamente.
- `/Applications/Kuota.app` fue actualizado con el bundle recien compilado.
Notas:
- No se mostro ni registro ningun codigo de dispositivo; el probe redacta URL/codigos.
- El build local termino con error solo al firmar el artefacto updater porque no esta configurada `TAURI_SIGNING_PRIVATE_KEY` en esta shell; el `.app` si se genero.

Fecha: 2026-06-30
Tarea: Documentar soporte macOS/Windows y regla sin Linux
Cambios:
- README explica que en macOS el codigo de autorizacion se copia automaticamente al portapapeles antes de abrir OpenAI.
- README documenta que los builds oficiales se generan solo para macOS y Windows hasta nuevo aviso.
- DECISIONS/HANDOFF registran la regla de no compilar/publicar Linux y de crear commits/notas descriptivas por version nueva.
Archivos:
- `README.md`
- `docs/CHANGELOG_AI.md`
- `docs/DECISIONS.md`
- `docs/HANDOFF.md`
Validación:
- `npm run build` pasó.
- `cargo check` pasó.
- `cargo check` en `C:\Kuota-windows-test\src-tauri` devolvio exit 0 en la VM Windows.
Notas:
- No cambia runtime; solo documenta reglas y comportamiento ya implementado.

Fecha: 2026-06-30
Tarea: Portar login nativo oculto a macOS
Cambios:
- macOS deja de abrir Terminal para `codex-auth login --device-auth` y usa `MacCodexLoginService`.
- El servicio macOS ejecuta `codex-auth` oculto, captura stdout/stderr, detecta URL/codigo, copia el codigo con `pbcopy` y abre OpenAI con `open`.
- El modal nativo de autorizacion ahora se usa tanto en macOS como en Windows.
- La cancelacion desde la X mata el proceso hijo tambien en macOS.
Archivos:
- `src-tauri/src/mac_codex_login.rs`
- `src-tauri/src/main.rs`
- `src/store/useStore.ts`
- `docs/CHANGELOG_AI.md`
- `docs/HANDOFF.md`
- `docs/DECISIONS.md`
Validación:
- `npm run build` pasó.
- `cargo fmt` pasó.
- `cargo check` pasó sin warnings.
Notas:
- El parser macOS se guio por el formato estable observado en Terminal: URL `https://auth.openai.com/codex/device` y codigo posterior a `Enter this one-time code`.
- No se registra stdout completo ni se guardan tokens.

Fecha: 2026-06-30
Tarea: Evitar falso codigo ChatGPT antes del device code
Cambios:
- El parser Windows bloquea palabras de producto/flujo como `ChatGPT`, `OpenAI`, `Codex`, `sign` y `login` como posibles codigos.
- En contexto de codigo, los tokens mixtos tipo `ChatGPT` ya no pasan si no tienen digitos ni separador.
- El campo de codigo muestra solo `--------` hasta que exista un device code real.
Archivos:
- `src-tauri/src/windows_codex_login.rs`
- `src/App.tsx`
- `docs/CHANGELOG_AI.md`
Validación:
- `npm run build` pasó.
- `cargo fmt` pasó.
- `cargo check` pasó.
- `cargo check` en `C:\Kuota-windows-test\src-tauri` devolvio exit 0 en la VM Windows.
Notas:
- El cambio evita abrir/copiar un falso positivo antes de que `codex-auth` imprima el codigo real.

Fecha: 2026-06-30
Tarea: Reforzar deteccion del codigo de autorizacion Windows
Cambios:
- El parser de `WindowsCodexLoginService` ahora conserva contexto cuando detecta textos tipo device code, authorization code o one-time code.
- La deteccion acepta codigos en la misma linea o en las siguientes lineas, con guion, underscore, mayusculas/minusculas o digitos, sin depender de longitud fija.
- Se agregaron stopwords para evitar confundir texto de ayuda o advertencias de phishing con el codigo real.
Archivos:
- `src-tauri/src/windows_codex_login.rs`
- `docs/CHANGELOG_AI.md`
- `docs/HANDOFF.md`
Validación:
- `npm run build` pasó.
- `cargo fmt` pasó.
- `cargo check` pasó.
- `cargo check` en `C:\Kuota-windows-test\src-tauri` devolvio exit 0 en la VM Windows.
Notas:
- No se registra stdout completo ni se guardan tokens; solo se detecta el device code necesario para copiarlo al portapapeles.

Fecha: 2026-06-30
Tarea: Mejorar cierre y copia automatica del codigo en login Windows
Cambios:
- El modal de autorizacion Windows ahora se cierra de inmediato al presionar la X y cancela el proceso hijo en segundo plano.
- Al detectar el codigo de autorizacion, Kuota lo copia automaticamente al portapapeles con un proceso oculto de PowerShell.
- La pagina de OpenAI se abre automaticamente solo despues de copiar el codigo; si la copia falla, el modal queda abierto para copia manual.
- La UI conserva el mensaje del backend para indicar que el codigo fue copiado antes de abrir OpenAI.
Archivos:
- `src-tauri/src/windows_codex_login.rs`
- `src/store/useStore.ts`
- `docs/CHANGELOG_AI.md`
- `docs/HANDOFF.md`
Validación:
- `npm run build` pasó.
- `cargo fmt` pasó.
- `cargo check` pasó.
Notas:
- No se copian tokens ni credenciales; solo el device code mostrado por `codex-auth`.

Fecha: 2026-06-30
Tarea: Corregir autorizacion Windows por Codex CLI faltante
Cambios:
- Kuota ahora verifica `codex` en Windows antes de iniciar `codex-auth login --device-auth`.
- Si falta Codex CLI, instala `@openai/codex@latest` con npm en el mismo preflight que prepara `@loongphy/codex-auth`.
- El `PATH` extendido de Windows agrega la carpeta vendor donde `@openai/codex` instala `codex.exe`, para que `codex-auth` no dependa del shim `codex.cmd`.
- Los procesos hijos reciben tanto `PATH` como `Path` en Windows para evitar que Node y binarios nativos lean rutas distintas.
- El servicio oculto de login reconoce el error de `codex` faltante y devuelve un mensaje especifico, sin mostrar stacks ni datos sensibles.
- La UI muestra un mensaje claro cuando falta Codex CLI en vez del error generico de autorizacion.
Archivos:
- `src-tauri/src/main.rs`
- `src-tauri/src/windows_codex_login.rs`
- `src/store/useStore.ts`
- `docs/CHANGELOG_AI.md`
- `docs/DECISIONS.md`
- `docs/HANDOFF.md`
Validación:
- `npm run build` pasó.
- `cargo fmt` pasó.
- `cargo check` pasó.
- `@openai/codex@latest` se instaló correctamente en Windows 11 ARM dentro de UTM.
- El probe Windows de `codex-auth login --device-auth` llegó a URL/código de autorizacion sin el error de `codex` faltante.
- `cargo check` en `C:\Kuota-windows-test\src-tauri` devolvio exit 0 en la VM Windows.
Notas:
- `codex-auth` requiere que el ejecutable `codex` exista en `PATH`; el fallo de Windows no era por ausencia de `codex-auth`, sino por ausencia de Codex CLI.

Fecha: 2026-06-30
Tarea: Instalacion robusta de codex-auth y diagnostico de login Windows
Cambios:
- `Agregar cuenta` ahora intenta instalar `@loongphy/codex-auth@latest` si no lo encuentra antes de iniciar el login.
- Las ejecuciones de npm usan `node.exe` con `npm-cli.js` en Windows cuando esta disponible, evitando depender del shim `npm.cmd`.
- El login oculto de Windows conserva un error amigable y sanitizado cuando `codex-auth` termina con fallo.
- Vite ignora `src-tauri/target` para evitar errores `EBUSY` mientras Cargo compila en Windows.
Archivos:
- `src-tauri/src/main.rs`
- `src-tauri/src/windows_codex_login.rs`
- `vite.config.ts`
- `docs/CHANGELOG_AI.md`
Validación:
- `npm run build` pasó.
- `cargo fmt` pasó.
- `cargo check` pasó.
Notas:
- Esta entrada quedo ampliada por el fix posterior: `codex-auth` si necesita Codex CLI disponible como `codex`.
- No se exponen tokens ni stdout completo en la UI.

Fecha: 2026-06-30
Tarea: Login nativo oculto de Codex en Windows
Cambios:
- Windows deja de abrir `cmd.exe` para `codex-auth login --device-auth`.
- Se agrega `WindowsCodexLoginService` para iniciar `codex-auth` oculto, capturar stdout/stderr, detectar URL/codigo y abrir el navegador del sistema.
- La UI muestra modal nativo con estado, codigo, copiar codigo, abrir OpenAI, reintento y cierre con cancelacion del proceso hijo.
- Al terminar correctamente, Kuota refresca cuentas/cuota automaticamente sin reiniciar.
Archivos:
- `src-tauri/src/windows_codex_login.rs`
- `src-tauri/src/main.rs`
- `src/store/useStore.ts`
- `src/App.tsx`
- `docs/CHANGELOG_AI.md`
- `docs/DECISIONS.md`
- `docs/HANDOFF.md`
Validación:
- `npm run build` pasó.
- `cargo fmt` pasó.
- `cargo check` pasó.
- `cargo check --target aarch64-pc-windows-msvc` queda bloqueado en macOS por toolchain C/MSVC faltante en `ring` (`assert.h`), antes de compilar la crate de Kuota.
Notas:
- No se agregaron dependencias ni secretos.
- El flujo Windows no emite logs con stdout completo ni tokens; solo eventos de progreso, URL/codigo de device auth y errores genericos.

Fecha: 2026-06-30
Tarea: Limitar releases a macOS y Windows
Cambios:
- El workflow de release deja de compilar Linux hasta nuevo aviso.
- Las notas automáticas de release ahora anuncian assets solo para macOS y Windows.
Archivos:
- `.github/workflows/release.yml`
- `docs/CHANGELOG_AI.md`
- `docs/DECISIONS.md`
- `docs/HANDOFF.md`
Validación:
- Pendiente de ejecución en GitHub Actions.
Notas:
- Se canceló el workflow inicial de `v1.0.3` para evitar publicar assets Linux.

Fecha: 2026-06-30
Tarea: Preparar release 1.0.3
Cambios:
- Versión actualizada a `1.0.3` en package, Tauri, Cargo y modal Acerca de Kuota.
- Release incluye fixes de login interactivo en Windows, alineación del popover al icono del tray y barra de progreso de actualización.
Archivos:
- `package.json`
- `package-lock.json`
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`
- `src/App.tsx`
- `docs/CHANGELOG_AI.md`
- `docs/DECISIONS.md`
- `docs/HANDOFF.md`
Validación:
- `npm run build` pasó.
- `cargo check` pasó.
- `git diff --check` pasó.
Notas:
- No se agregaron dependencias ni secretos.

Fecha: 2026-06-30
Tarea: Alinear popover al icono de tray y mostrar progreso de update
Cambios:
- El popover ahora usa la posición real del click del tray para centrarse respecto al icono, con límites al área visible del monitor.
- El fallback de posición sigue disponible si el sistema no entrega coordenadas del tray.
- El flujo de actualización ahora captura eventos reales del updater y muestra una barra de progreso en Ajustes durante la descarga.
Archivos:
- `src-tauri/src/tray.rs`
- `src/store/useStore.ts`
- `src/App.tsx`
- `docs/CHANGELOG_AI.md`
- `docs/DECISIONS.md`
- `docs/HANDOFF.md`
Validación:
- `npm run build` pasó.
- `cargo fmt` pasó.
- `cargo check` pasó.
Notas:
- No se agregaron dependencias ni secretos.

Fecha: 2026-06-30
Tarea: Corregir login de nueva cuenta en Windows
Cambios:
- `codex_auth_login` ahora abre `cmd.exe /K` con una consola nueva en Windows para ejecutar `codex-auth login --device-auth`.
- El comando de Windows usa `call` para soportar correctamente el shim `.cmd` global de npm.
- Se mantiene el `PATH` extendido al abrir el login, para que `codex-auth.cmd` resuelva Node/npm igual que los comandos internos.
- macOS y Linux conservan su comportamiento anterior.
Archivos:
- `src-tauri/src/main.rs`
- `docs/CHANGELOG_AI.md`
- `docs/DECISIONS.md`
- `docs/HANDOFF.md`
Validación:
- `npm run build` pasó.
- `cargo fmt` pasó.
- `cargo check` pasó en el target local.
- `cargo check --target x86_64-pc-windows-msvc` queda bloqueado por toolchain nativo faltante en `ring` (`assert.h`), antes de compilar la crate de la app.
Notas:
- No se agregaron dependencias ni secretos.

## Formato

```txt
Fecha:
Tarea:
Cambios:
Archivos:
Validación:
Notas:
```
