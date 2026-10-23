# CHANGELOG_AI

Fecha: 2026-10-22
Tarea: Preparar release Kuota 1.0.10
Cambios: Version sincronizada en los manifests, lockfiles y el modal Acerca de. `docs/releases/v1.0.10.md` documenta en español e inglés las correcciones de cuenta activa, cuentas ocultas y refresco después del login por código.
Validacion: `npm run build`, `npm run lint`, `cargo fmt --check`, `cargo check --locked`, `cargo test --locked` (9 pruebas) y `git diff --check` pasaron. Commit `a90e959` y tag `v1.0.10` publicados en GitHub; workflow 39907796704 completado correctamente para macOS universal y Windows x64. Release pública verificada con notas bilingües y ocho assets, incluido `latest.json`.
Seguridad: Sin cambios en credenciales, sesiones ni dependencias.

Fecha: 2026-10-22
Tarea: Corregir cuenta activa incorrecta y cuentas ocultas que reaparecen
Cambios: La cuenta activa ahora se deriva del estado vigente de `codex-auth` y deja de conservarse en almacenamiento local como una preferencia. Al borrar una cuenta detectada, Kuota también puede recuperar el correo desde el ID `codex-auth-*`; las claves ocultas existentes se normalizan durante la migracion.
Archivos: `src/store/useStore.ts`, `docs/CHANGELOG_AI.md`, `docs/DECISIONS.md`, `docs/HANDOFF.md`.
Validacion: `npm run build`, `npm run lint` y `git diff --check` pasaron.
Seguridad: No se modifican credenciales ni el almacenamiento de sesiones de Codex; no se agregan dependencias.

Fecha: 2026-10-22
Tarea: Corregir el refresco de cuentas tras login por codigo
Cambios: Al completar un login nativo de Codex, Kuota vuelve a consultar las cuentas a los 0, 5, 15 y 45 segundos para cubrir el retraso entre la confirmacion del navegador y la disponibilidad de la cuenta en `codex-auth list`. El modal conserva su cierre automatico.
Archivos: `src/store/useStore.ts`, `docs/CHANGELOG_AI.md`, `docs/DECISIONS.md`, `docs/HANDOFF.md`.
Validacion: `npm run build`, `npm run lint`, `cargo fmt --check`, `cargo check --locked` y `cargo test --locked` (9 pruebas) pasaron.
Seguridad: Sin cambios en credenciales, almacenamiento de sesiones ni dependencias.

Fecha: 2026-09-16
Tarea: Publicar Kuota 1.0.9
Cambios: Version 1.0.9 en manifests, lockfiles y Acerca de; notas bilingues en docs/releases/v1.0.9.md para la correccion de cuotas con codex-auth 0.3.0. Publicacion mediante commit/tag/push y workflow existente para macOS universal y Windows x64.
Resultado: v1.0.9 publicada desde 1dee05a; workflow 35136661281 exitoso en ambas plataformas. Assets y manifiesto latest.json verificados.

Fecha: 2026-09-16
Tarea: Corregir cuotas con codex-auth 0.3.0
Cambios: El parser identifica las columnas 5H/WEEKLY por encabezado y separa los campos tabulados, incluidos errores de varias palabras; conserva el parser de 0.2.x. Rechaza cuotas de filas incompletas para evitar cambios automaticos con datos desplazados.
Validación: 9 pruebas Rust, build TypeScript/Vite y lint correctos. El parser extraido del codigo se probo con codex-auth list real: ambas cuentas Plus devolvieron porcentajes; Free devolvio 401 unauthorized_unknown.
Seguridad: Sin dependencias nuevas, sin exponer tokens ni modificar la logica de autenticacion.

Fecha: 2026-09-02
Tarea: Preparar Kuota 1.0.8 con relanzamiento compatible con ChatGPT Desktop
Cambios:
- Version actualizada a `1.0.8` en npm, Cargo, Tauri y el modal Acerca de Kuota.
- Se agregaron notas bilingues en `docs/releases/v1.0.8.md` para la compatibilidad de relanzamiento con ChatGPT Desktop y Codex Desktop.
- La release conserva intactas las sesiones de Codex CLI y no incorpora renovacion OAuth propia; el diagnostico de snapshots caducados permanece fuera de este alcance.
Archivos:
- `package.json`
- `package-lock.json`
- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`
- `src-tauri/tauri.conf.json`
- `src/App.tsx`
- `docs/releases/v1.0.8.md`
- `docs/CHANGELOG_AI.md`
- `docs/DECISIONS.md`
- `docs/HANDOFF.md`
Validación:
- `npm run build` paso mostrando `kuota@1.0.8`.
- `npm run lint`, `cargo fmt --check`, `cargo check`, `cargo test` y `git diff --check` pasaron.
- Las 7 pruebas Rust pasaron, incluida la deteccion de ChatGPT Desktop y Codex Desktop.
Notas:
- La entrega se publica desde `main` mediante el tag `v1.0.8`, que dispara los builds oficiales de macOS y Windows.
- No se agregaron dependencias ni secretos.

Fecha: 2026-09-02
Tarea: Reabrir Codex o ChatGPT Desktop despues de cambiar cuenta
Cambios:
- La deteccion de Desktop reconoce tanto `ChatGPT.app`/`ChatGPT.exe` como los nombres anteriores `Codex.app`/`Codex.exe`.
- En macOS, el reinicio usa el bundle estable `com.openai.codex`, confirmado en la instalacion actual de `ChatGPT.app`, para no depender del nombre visible de la app.
- En Windows, Kuota conserva la ruta y el nombre del ejecutable Desktop que estaba activo, lo cierra y verifica que vuelva a abrir ese mismo proceso.
- Se mantienen las sesiones de Codex CLI sin cerrarlas y no se agregaron dependencias.
Archivos:
- `src-tauri/src/main.rs`
- `docs/CHANGELOG_AI.md`
- `docs/DECISIONS.md`
- `docs/HANDOFF.md`
Validación:
- Prueba Rust agregada para reconocer los nombres anterior y actual de Desktop sin confundir Codex CLI.
- `cargo test` paso con 7 pruebas y `cargo check` paso en macOS.
- `npm run build`, `npm run lint`, `cargo fmt --check` y `git diff --check` pasaron.
- El proceso real activo se confirmo en `/Applications/ChatGPT.app/Contents/MacOS/ChatGPT` y su bundle identifier es `com.openai.codex`.
- El check cruzado de Windows quedo bloqueado en la dependencia `ring` por falta del toolchain C/MSVC (`assert.h`), antes de compilar Kuota.

Fecha: 2026-08-31
Tarea: Preparar Kuota 1.0.7 con auto-switch estricto
Cambios:
- Kuota controla la seleccion automatica y deshabilita el auto-switch nativo de `codex-auth` para no aceptar candidatas con cuota desconocida.
- La cuenta destino exige semanal valida mayor que `0%` y, cuando existe la columna de 5 horas, tambien un porcentaje valido mayor que `0%`.
- Sin candidata valida no se cambia `auth.json`, no se reinicia Codex Desktop y no se altera ninguna sesion CLI.
- Un watcher local detecta `rate_limits` agotados en el rollout activo y dispara una unica validacion completa; el snapshot de 60 segundos queda como respaldo.
- La preferencia de auto-switch se persiste en Kuota y permanece desactivada por defecto.
- Version actualizada a `1.0.7` en todos los manifests y el modal Acerca de; se agregaron notas bilingues y se actualizo README.
Archivos:
- `src-tauri/src/main.rs`
- `src/store/useStore.ts`
- `src/App.tsx`
- `README.md`
- `package.json`
- `package-lock.json`
- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`
- `src-tauri/tauri.conf.json`
- `docs/releases/v1.0.7.md`
- `docs/CHANGELOG_AI.md`
- `docs/DECISIONS.md`
- `docs/HANDOFF.md`
Validación:
- `npm run build` paso.
- `npm run lint` paso.
- `cargo fmt --check` paso.
- `cargo check` paso.
- `cargo test` paso con 6 pruebas.
- `git diff --check` paso.
- `npm run tauri:dev` recompilo y arranco `target/debug/kuota` como version `1.0.7`.
Notas:
- No se agregaron dependencias ni secretos.

Fecha: 2026-08-31
Tarea: Reañadir cuota de 5 horas por capacidad y auto-switch reactivo
Cambios:
- Se aumento la altura real del popover de 520 a 590 px en React, Tauri y el fallback de posicionamiento para evitar scroll interno en la vista normal de cuotas.
- La actividad de `codex-auth` ahora se localiza: `Now`, `Never` y formatos `ago` compactos o completos se muestran en español cuando corresponde.
- Los reinicios con formato `00:26 on 1 Sep` ahora se muestran como `00:26 el 1 sep` en español, tanto en el encabezado de la cuota como en su detalle; inglés conserva el valor original.
- Se corrigio el corte de informacion en Ajustes: los modales ahora respetan la altura disponible, mantienen el encabezado fijo y desplazan verticalmente su contenido.
- Se reañadio la cuota de 5 horas para cualquier cuenta que entregue un porcentaje valido; Free deja de estar excluido por nombre de plan.
- Valores `401`, `RequestFailed`, `N/A` o ausentes no se muestran como cuota ni disparan auto-switch.
- El parser de `codex-auth list` ahora detecta el encabezado `5H USAGE` y conserva correctamente la cuota semanal cuando esa columna no existe.
- Se agregaron dos pruebas Rust para ambos formatos de salida.
- Ajustes permite habilitar o deshabilitar el auto-switch estricto controlado por Kuota.
- Kuota confirma que la cuenta activa llego a 0%, valida la cuota destino y reinicia Codex Desktop una sola vez; datos fallidos no disparan el cambio y no se terminan sesiones CLI activas.
- Tauri vigila `auth.json` cada 500 ms y emite un evento local con debounce; el frontend refresca al instante cuando `codex-auth` cambia la cuenta, sin esperar 60 s.
- El polling API de 60 s se conserva como respaldo para porcentajes porque la fuente no ofrece streaming oficial.
- El estado del toggle se persiste en Kuota y queda desactivado por defecto; el auto-switch nativo de `codex-auth` se mantiene apagado.
Archivos:
- `src-tauri/src/main.rs`
- `src-tauri/src/tray.rs`
- `src-tauri/tauri.conf.json`
- `src/store/useStore.ts`
- `src/types/index.ts`
- `src/App.tsx`
- `src/components/AccountCard.tsx`
- `README.md`
- `docs/CHANGELOG_AI.md`
- `docs/DECISIONS.md`
- `docs/HANDOFF.md`
Validación:
- `npm run build` paso.
- `npm run lint` paso.
- `cargo fmt --check` paso.
- `cargo check` paso.
- `cargo test` paso con 3 pruebas.
- `npm run tauri:dev` compilo y arranco `target/debug/kuota`.
- QA visual del modal Ajustes paso a 360x520.
- `git diff --check` paso.
Notas:
- No se agregaron dependencias, tokens ni logs de credenciales.
- No se activo el auto-switch ni se cambiaron cuentas durante la validacion.

Fecha: 2026-07-19
Tarea: Preparar release 1.0.6
Cambios:
- Version actualizada a `1.0.6` en manifests de npm, Tauri, Cargo y modal Acerca de Kuota.
- Se agregaron notas bilingues para retirar visualmente la cuota de 5 horas y conservar solo la semanal.
- Las notas incluyen en ambos idiomas el aviso solicitado sobre nuevos bugs para proximas versiones.
- La release mantiene builds oficiales solo para macOS y Windows.
Archivos:
- `package.json`
- `package-lock.json`
- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`
- `src-tauri/tauri.conf.json`
- `src/App.tsx`
- `docs/releases/v1.0.6.md`
- `docs/CHANGELOG_AI.md`
- `docs/DECISIONS.md`
- `docs/HANDOFF.md`
Validación:
- `npm run build` pasó.
- `npm run lint` pasó.
- `cargo fmt --check` pasó.
- `cargo check` pasó.
- `git diff --check` pasó.
Notas:
- No se agregaron dependencias ni secretos.

Fecha: 2026-07-19
Tarea: Simplificar la visualizacion a la cuota semanal de Codex
Cambios:
- Se retiro la cuota de 5 horas de la vista principal y de la tarjeta de cuenta.
- La interfaz conserva solamente la cuota semanal, que es el limite vigente relevante para el usuario.
- Se mantiene el parseo interno de la columna de 5 horas por compatibilidad con salidas anteriores de `codex-auth` y para no desplazar la columna semanal.
Archivos:
- `src/App.tsx`
- `src/components/AccountCard.tsx`
- `README.md`
- `docs/CHANGELOG_AI.md`
- `docs/DECISIONS.md`
- `docs/HANDOFF.md`
Validación:
- `npm run build` pasó.
- `npm run lint` pasó.
- `git diff --check` pasó.
Notas:
- No se agregaron dependencias ni se cambiaron flujos de autenticacion, cuentas o actualizacion de cuota.

Fecha: 2026-07-01
Tarea: Preparar release 1.0.5
Cambios:
- Version actualizada a `1.0.5` en manifests de npm, Tauri, Cargo y modal Acerca de Kuota.
- Se confirma que `docs/releases/v1.0.5.md` contiene cambios visibles en español e ingles para GitHub Releases.
- La release mantiene builds oficiales solo para macOS y Windows.
Archivos:
- `package.json`
- `package-lock.json`
- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`
- `src-tauri/tauri.conf.json`
- `src/App.tsx`
- `docs/releases/v1.0.5.md`
- `docs/CHANGELOG_AI.md`
- `docs/HANDOFF.md`
Validación:
- `npm run build` pasó.
- `cargo check` pasó.
- `git diff --check` pasó.
Notas:
- No se agregan dependencias nuevas.

Fecha: 2026-07-01
Tarea: Reiniciar Codex automaticamente en Windows despues de switch
Cambios:
- Despues de `codex-auth switch`, Windows detecta si `Codex.exe` estaba corriendo y lo trata como destino Desktop.
- `restart_codex_desktop` en Windows captura la ruta real del `Codex.exe` en ejecucion antes de cerrarlo.
- Luego cierra `Codex.exe` con `taskkill` oculto, espera a que desaparezca del proceso y vuelve a abrir esa misma ruta.
- El relanzamiento solo se considera exitoso si `Codex.exe` vuelve a aparecer en `tasklist`; si no, intenta `Start-Process` con PowerShell oculto y despues `explorer.exe` como fallback.
- Antes de relanzar deja un margen breve despues de que el proceso anterior desaparece para evitar choques con locks de instancia.
- Si no puede recuperar la ruta del proceso, intenta rutas comunes de instalacion.
- Si no encuentra Codex Desktop, intenta abrir Codex CLI como fallback usando el `PATH` extendido.
- El flujo evita `cmd.exe` y no abre ventanas intermedias de consola.
Archivos:
- `src-tauri/src/main.rs`
- `docs/CHANGELOG_AI.md`
- `docs/HANDOFF.md`
Validación:
- `npm run build` pasó.
- `cargo fmt` pasó.
- `cargo check` pasó.
- `git diff --check` pasó.
Notas:
- La deteccion de Desktop en Windows depende de que el proceso se llame `Codex.exe`; la ruta se obtiene desde WMI/CIM mediante PowerShell oculto.

Fecha: 2026-07-01
Tarea: Documentar regla permanente de releases bilingues y retirar Linux del README
Cambios:
- README ya no describe comportamiento de Linux como soportado.
- README aclara que Linux no esta soportado ni publicado oficialmente hasta nuevo aviso.
- README y AUTOUPDATE registran como regla permanente que GitHub Releases debe mostrar cambios en español e ingles.
- DECISIONS registra que la app solo debe mostrar el bloque del idioma configurado por el usuario.
Archivos:
- `README.md`
- `docs/AUTOUPDATE.md`
- `docs/DECISIONS.md`
- `docs/CHANGELOG_AI.md`
Validación:
- `git diff --check` pasó.
Notas:
- La regla tecnica usa `docs/releases/<tag>.md` con `<!-- kuota:es -->` y `<!-- kuota:en -->`.

Fecha: 2026-07-01
Tarea: Agregar dialogo localizado de actualizaciones de Kuota
Cambios:
- La app revisa GitHub Releases mediante Tauri Updater y muestra un dialogo si existe una nueva version.
- El dialogo permite actualizar ahora u omitir por el momento; la descarga/instalacion ya no ocurre automaticamente al detectar la update.
- El dialogo muestra version actual, nueva version y cambios de release en el idioma activo de Kuota.
- Se agrego parser de release notes con marcadores `<!-- kuota:es -->` y `<!-- kuota:en -->`.
- Se creo `docs/releases/v1.0.5.md` con notas bilingues para la proxima release.
Archivos:
- `src/store/useStore.ts`
- `src/App.tsx`
- `docs/releases/v1.0.5.md`
- `docs/CHANGELOG_AI.md`
- `docs/HANDOFF.md`
- `docs/AUTOUPDATE.md`
Validación:
- `npm run build` pasó.
- `cargo check` pasó.
- `git diff --check` pasó.
Notas:
- Se reutiliza el updater firmado de Tauri; no se agrega dependencia ni cliente paralelo para GitHub API.

Fecha: 2026-07-01
Tarea: Agregar invitacion Ko-fi en Ajustes y Acerca de
Cambios:
- Ajustes y Acerca de Kuota muestran una leyenda localizada sobre los Rokaccinos del Caffenio.
- Se agrego boton localizado de Ko-fi: `Invitarle un Rokaccino` en espanol y `Treat him to a Rokaccino` en ingles.
- El boton abre `https://ko-fi.com/O4L621U2AQ` con el navegador del sistema usando el plugin shell de Tauri.
- Se evito el widget remoto con script para no ejecutar JavaScript de terceros dentro de Tauri.
Archivos:
- `src/App.tsx`
- `docs/CHANGELOG_AI.md`
Validación:
- `npm run build` pasó.
- `cargo check` pasó.
- `git diff --check` pasó.
Notas:
- El enlace apunta a `https://ko-fi.com/O4L621U2AQ`.

Fecha: 2026-07-01
Tarea: Corregir switch de cuenta en Windows sin depender de CMD
Cambios:
- `switch_codex_account` ahora ejecuta `codex-auth switch` mediante el launcher seguro de `codex-auth` en Windows.
- El launcher compartido resuelve shims npm de `codex-auth` a `node.exe <script>` para evitar depender de `.cmd`, `cmd.exe` o una consola visible.
- `codex-auth --version`, `status`, `list` y `switch` usan el mismo camino de ejecucion, reduciendo diferencias entre login y switch.
- `WindowsCodexLoginService` reutiliza el launcher compartido en vez de mantener una copia local.
- La busqueda de comandos en Windows ahora prefiere `.exe` antes de `.cmd` cuando ambos existen.
Archivos:
- `src-tauri/src/main.rs`
- `src-tauri/src/windows_codex_login.rs`
- `docs/CHANGELOG_AI.md`
- `docs/HANDOFF.md`
Validación:
- `npm run build` pasó.
- `cargo fmt` pasó.
- `cargo check` pasó.
- En UTM Windows ARM, `cargo check` queda bloqueado antes de compilar Kuota porque `ring` no encuentra `clang`/toolchain C (`EXIT_101`).
Notas:
- No se registran tokens ni stdout completo; el cambio solo modifica la forma de ejecutar el comando local.
- Politica vigente: evitar CMD/PowerShell visibles en Windows. PowerShell solo queda permitido como proceso oculto y no interactivo para tareas nativas puntuales como copiar al portapapeles.

Fecha: 2026-07-01
Tarea: Corregir notas visibles de release 1.0.4
Cambios:
- Se actualizo la release publicada `v1.0.4` con notas funcionales y visibles para el usuario.
- Se agrego `docs/releases/v1.0.4.md` como fuente explicita de notas de release.
- El workflow de release ahora usa `docs/releases/<tag>.md` cuando existe, evitando depender solo de subjects de commits.
Archivos:
- `.github/workflows/release.yml`
- `docs/releases/v1.0.4.md`
- `docs/CHANGELOG_AI.md`
Validación:
- `gh release view v1.0.4` muestra los cambios principales, documentacion/reglas y assets.
- `git diff --check` pasó.
Notas:
- Esta regla queda operativa: cada nueva version debe agregar notas explicitas si los commits no describen suficiente el cambio para usuarios.

Fecha: 2026-07-01
Tarea: Preparar release 1.0.4
Cambios:
- Version actualizada a `1.0.4` en package, Tauri, Cargo, lockfiles y modal Acerca de Kuota.
- README documenta que macOS y Windows usan login nativo de Codex sin consola, con codigo copiado automaticamente al portapapeles.
- README deja como regla que cada version nueva debe incluir commits descriptivos, cambios visibles en releases y builds solo para macOS/Windows hasta nuevo aviso.
Archivos:
- `package.json`
- `package-lock.json`
- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`
- `src-tauri/tauri.conf.json`
- `src/App.tsx`
- `README.md`
- `docs/CHANGELOG_AI.md`
- `docs/HANDOFF.md`
- `docs/DECISIONS.md`
Validación:
- `npm run build` pasó.
- `cargo check` pasó.
- `git diff --check` pasó.
Notas:
- Linux permanece fuera de las compilaciones oficiales hasta nuevo aviso.

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
