# HANDOFF

## Actualización 2026-06-30: releases solo macOS y Windows

- `.github/workflows/release.yml` deja fuera `ubuntu-22.04` del matrix hasta nuevo aviso.
- Las notas generadas por el workflow ahora anuncian builds para macOS y Windows.
- Se canceló el workflow inicial de `v1.0.3` para evitar publicar assets Linux.

## Actualización 2026-06-30: release 1.0.3

- Versión preparada como `1.0.3` en `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json` y el modal `Acerca de Kuota`.
- Incluye fixes de login interactivo en Windows, alineación del popover al icono del tray y barra de progreso real durante descarga de actualización.
- El flujo esperado es commit en `main`, tag `v1.0.3` y push de branch/tag para que `.github/workflows/release.yml` publique los assets.

## Actualización 2026-06-30: alineación del popover y progreso de updates

- `src-tauri/src/tray.rs` ahora posiciona la ventana `popover` usando la coordenada real del click del tray y centra la ventana respecto al icono.
- El posicionamiento se limita al `work_area` del monitor activo para evitar que la ventana se salga de pantalla; el fallback previo queda para eventos sin coordenadas.
- `src/store/useStore.ts` captura eventos `Started`, `Progress` y `Finished` del updater de Tauri.
- `src/App.tsx` muestra una barra compacta de progreso en Ajustes mientras se descarga una actualización.
- Validado con `npm run build`, `cargo fmt` y `cargo check`.

## Actualización 2026-06-30: login de nueva cuenta en Windows

- Se corrigió el flujo de `Nueva cuenta` en Windows: `codex_auth_login` ahora abre `cmd.exe /K` con `CREATE_NEW_CONSOLE` para ejecutar `codex-auth login --device-auth`.
- La invocación usa `call` para que el shim `.cmd` instalado por npm funcione correctamente dentro de `cmd.exe`.
- El cambio evita que el flujo interactivo quede invisible al ejecutarse desde la app GUI de Tauri (`windows_subsystem = "windows"`).
- Se conserva el `PATH` extendido para que el ejecutable global de npm (`codex-auth.cmd`) funcione desde la ventana abierta.
- Validado con `npm run build`, `cargo fmt` y `cargo check` local.
- El check cruzado `cargo check --target x86_64-pc-windows-msvc` sigue bloqueado en esta máquina por toolchain nativo faltante para `ring` (`assert.h`), antes de compilar la crate de Kuota.

## Actualización 2026-06-28: automatización de releases

- `.github/workflows/release.yml` ahora genera notas de release desde los commits incluidos entre el tag anterior y el tag nuevo.
- En cada push de tag `v*`, el workflow intenta avanzar `main` con fast-forward al commit del tag para que el código fuente público quede en la versión más reciente.
- Las releases dejan de quedar como draft si el build de Tauri termina correctamente.

## Actualización 2026-06-28: release 1.0.2

- Versión preparada como `1.0.2` en `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json` y el modal `Acerca de Kuota`.
- Incluye los arreglos de ventanas de consola en Windows, estado de actualización al día y cierre visual del popover al perder foco.

## Actualización 2026-06-28: ventanas de consola en Windows

- Se centralizó la creación de procesos internos en `src-tauri/src/main.rs` con `hidden_command`.
- En Windows, los comandos automáticos (`npm`, `codex-auth status/list/version/switch` y aperturas internas) se ejecutan con `CREATE_NO_WINDOW` para evitar ventanas de consola repetidas mientras se usa la app.
- Se dejó `codex-auth login` como proceso visible/interactivo fuera de macOS, porque puede requerir mostrar instrucciones o recibir entrada del usuario.
- Validado con `cargo check` local y compilación mínima de `CommandExt + CREATE_NO_WINDOW` contra `x86_64-pc-windows-msvc`; el check completo de Windows queda bloqueado en esta máquina por toolchain nativo faltante para `ring` (`assert.h`).

## Actualización 2026-06-28: ajustes visuales del popover

- El botón de búsqueda de actualizaciones ahora muestra `Ya tienes la versión más reciente bro` cuando termina una revisión sin encontrar actualizaciones.
- La ventana `popover` se oculta automáticamente cuando pierde foco, para cerrar visualmente la app al hacer click fuera sin volver a presionar el icono del tray.

## Actualización 2026-06-26: codex-auth en Windows

- Versión de release preparada como `1.0.1`.
- Se corrigió la detección de comandos para Windows: Kuota ahora prioriza variantes `.cmd` y `.exe` de `npm` y `codex-auth` antes del shim sin extensión.
- Se agregó la ruta global típica de npm en Windows (`%APPDATA%/npm`) a la búsqueda y al `PATH` usado por comandos internos.
- `extended_path` ahora usa `std::env::join_paths`, evitando separar rutas con `:` en Windows donde corresponde `;`.

## Actualización 2026-06-25: icono de aplicación

- Icono de launcher actualizado desde `/Users/richtunic/Downloads/Kuota Icon APP.png`: `src-tauri/icons/app-source.png`, PNGs de bundle, `icon.icns` y `icon.ico`.
- El icono de menubar/tray queda separado en `src-tauri/icons/tray.png` y no se cambió.

## Actualización 2026-06-26: aclaración de cuota

- README aclarado: Kuota ejecuta `codex-auth list` sin `--skip-api`, usando la consulta API por defecto de `codex-auth`, pero la app refresca snapshots cada 60 segundos y al recuperar foco; no usa streaming en tiempo real.

## Estado actual 2026-06-25

Actualización UI: el popup se reacomodó para parecerse más a CodexBar, manteniendo el tema oscuro propio.

- Selector horizontal de cuentas arriba, con indicador de cuenta activa y mini barra semanal.
- Panel de detalle para la cuenta seleccionada con título, sync, plan, estado activo y secciones `5 h` / `Semanal`.
- Barras de uso con porcentaje restante y reset parseados desde `codex-auth list`.
- Sección `Account` con email y actividad.
- Footer compactado: se retiró `Actualizar uso`; quedan `Ajustes`, `Acerca/About Kuota` y `Cerrar app`.
- `Ajustes` permite cambiar idioma, buscar actualizaciones de `codex-auth` y eliminar/ocultar cuentas que ya no se usan.
- `Acerca/About Kuota` muestra versión, autor RichTunic y base técnica en Codex-Auth de terminal.
- El popup invierte esquinas por plataforma: macOS redondea abajo; Windows/Linux redondean arriba porque se despliega desde la bandeja inferior.
- README preparado para GitHub con logo `docs/assets/kuota-logo.png`, capturas demostrativas, funcionamiento de la app, flujo de `codex-auth`, desarrollo local y notas de seguridad/privacidad.
- Autoupdate configurado para `richtunic/Kuota`: plugins Tauri updater/process instalados, permisos agregados, UI llama `checkAllUpdates`, `bundle.createUpdaterArtifacts` activado, workflow `.github/workflows/release.yml` creado, endpoint/public key en `tauri.conf.json`, secrets `TAURI_SIGNING_PRIVATE_KEY` y `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` creados en GitHub, y guía `docs/AUTOUPDATE.md` agregada.
- Iconos de app/tray actualizados desde `/Users/richtunic/Downloads/Kuota.png`: `src-tauri/icons/32x32.png`, `128x128.png`, `128x128@2x.png` y `tray.png`.
- El tray de macOS ahora carga `src-tauri/icons/tray.png` embebido y lo marca como template para evitar que se vea como recuadro negro en la menu bar.
- Validado con `npx tsc --noEmit` y `cargo check`.

Actualización final de enfoque: se eliminó el proxy del flujo principal. La app ahora funciona nativamente sobre `@loongphy/codex-auth`.

- Frontend ya no muestra ni controla proxy.
- Backend ya no expone comandos `start_proxy`, `stop_proxy`, `proxy_status`, `read_usage` ni `set_system_proxy`.
- Al iniciar, la UI llama `ensure_codex_auth`: instala `@loongphy/codex-auth@latest` si falta y lo actualiza automáticamente si npm reporta una versión distinta.
- `codex-auth list` es la fuente de cuentas y uso; se parsean `plan`, `5H USAGE`, `WEEKLY USAGE` y `LAST ACTIVITY`.
- `Usar en Codex` ejecuta `codex-auth switch <email|alias>`, reinicia/abre Codex y refresca la lista. Importante: `codex-auth switch <query>` no acepta `--skip-api`; ese flag solo aplica al switch interactivo.
- Antes del switch, Tauri detecta si se estaba usando Codex Desktop o CLI:
  - Si Desktop estaba corriendo, ejecuta el switch, cierra `Codex.app`, espera brevemente y vuelve a abrir `Codex.app`.
  - Si no detecta Desktop pero detecta CLI, ejecuta el switch y abre una Terminal nueva con `codex`.
  - Si no detecta nada, abre Codex Desktop por defecto.
- `Nueva cuenta` ejecuta `codex-auth login --device-auth` en una Terminal guiada en macOS, porque `codex-auth login` es interactivo; al terminar, el usuario vuelve a la app y presiona `Actualizar cuentas`.
- Después de iniciar `Nueva cuenta`, la app agenda refrescos automáticos de `codex-auth list` al momento, 5s, 15s y 45s.
- La app también refresca `codex-auth list` cada 60s y cuando la ventana vuelve a tener foco/visibilidad.
- El formulario manual quedó reducido a nombre, email/alias, color y notas; ya no pide navegador/perfil.
- Si se editan preferencias visuales de una cuenta detectada por `codex-auth`, se guardan localmente.
- `src-tauri/src/lib.rs` y `src-tauri/src/main.rs` dejaron de compilar módulos proxy/usage.
- Validación final: `npx tsc --noEmit` pasó y `cargo check` pasó sin warnings.
- Limpieza de proxy del sistema: Wi‑Fi HTTP/HTTPS y Thunderbolt Bridge HTTP/HTTPS quedaron `Enabled: No`.

Actualización posterior: la app ahora trata `@loongphy/codex-auth` como motor real de cuentas.

- `codex_auth_status` detecta `codex-auth` aunque Tauri no herede el PATH de terminal (`~/.npm-global/bin`, Homebrew, `/usr/local/bin`, etc.).
- La UI muestra estado/version de `codex-auth`, y un botón `Instalar` si falta.
- `install_codex_auth` ejecuta `npm install -g @loongphy/codex-auth` desde Tauri.
- `codex-auth list` se parsea y sus cuentas aparecen como cards aunque no se hayan creado manualmente.
- `switch_codex_account` ya no solo abre navegador: exige `codex-auth` si está instalado y ejecuta `codex-auth switch <selector> --skip-api`.
- Validado otra vez con `npx tsc --noEmit` y `cargo check`.

Se reorientó la app hacia una experiencia tipo `loongphy/codex-auth`, pero visual para usuarios no técnicos:

- Cards únicas por cuenta, deduplicadas por email/id.
- Botón `Usar en Codex` que guarda la cuenta activa y llama al backend.
- Backend `switch_codex_account` intenta `codex-auth switch <email|alias> --skip-api` si `codex-auth` está instalado.
- Si `codex-auth` no está disponible, la app abre `https://chatgpt.com/codex` con el navegador/perfil configurado como fallback.
- Si `codex-auth` cambia la cuenta, se intenta reiniciar/abrir la app `Codex` en macOS.
- No se implementó refresh API con access token por ahora; `codex-auth` advierte que puede tener riesgo de ToS/suspensión. El uso sigue viniendo de proxy/local.
- Se corrigió el ciclo del proxy: si falla el arranque, se apaga el proxy del sistema; al detener, siempre intenta apagar HTTP/HTTPS; al iniciar/cerrar app también limpia proxy en macOS.
- Se apagó manualmente el proxy HTTP/HTTPS de `Wi-Fi` para restaurar internet normal.

Archivos reconstruidos por sintaxis inválida previa:

- `src-tauri/src/main.rs`
- `src-tauri/src/proxy.rs`
- `src-tauri/src/tray.rs`
- `src-tauri/src/usage.rs`
- `src/store/useStore.ts`
- `src/App.tsx`
- `src/components/AccountCard.tsx`
- `src/components/AccountForm.tsx`

Validación:

- `npx tsc --noEmit` pasó.
- `cargo check` pasó en `src-tauri`.

Siguiente paso:

1. Levantar `npm run tauri:dev`.
2. Abrir la app desde menu bar.
3. Agregar una cuenta cuyo email/alias coincida con `codex-auth list`.
4. Presionar `Usar en Codex`.
5. Confirmar que Codex toma la cuenta tras reinicio/refresh y que internet sigue funcionando al detener proxy.

## Última tarea
2026-06-25: Se corrigió la repetición de cuentas: deduplicación por email/id en interceptor y store, polling cambiado a 60s, y `Abrir Codex` apunta directo a `https://chatgpt.com/codex`.
2026-06-25: Se unificó el diseño de la app en un sistema visual compacto tipo Codex: fondos, bordes, botones, modal, tarjeta y footer usan la misma escala y paleta.
2026-06-25: Se cambió la visualización de cuotas para parecerse a Codex: bloque `Uso restante` con filas `5 h` y `Semanal`, porcentaje restante y reset.
2026-06-25: Se diagnosticó por qué no marcaba uso Codex: el puerto 8080 estaba ocupado por otro proxy, el interceptor no encontraba el script en dev y el endpoint actual de uso es `/backend-api/wham/usage`.
2026-06-25: Se eliminaron cuentas demo/confusas y se ajustó la UI para describir el producto como router de cuentas Codex con progreso semanal y de 5 horas.
2026-06-25: Se corrigió el wiring del tray/menu bar para que el icono no quede como tray pasivo y el frontend pueda escuchar eventos Tauri.
2026-06-25: Se ejecutó `npm run tauri:dev` y se corrigieron bloqueos mínimos de arranque en Tauri/Vite.

## Archivos modificados
- `proxy/gpt_interceptor.py`: deduplica cuentas antes de escribir `usage.json`, conservando la captura más reciente con datos.
- `src/store/useStore.ts`: deduplica capturas antes de generar cards; una card por email/id aunque `usage.json` tenga entradas viejas.
- `src-tauri/src/usage.rs`: polling de `usage.json` cada 60 segundos.
- `src-tauri/src/main.rs`: `open_browser` abre `https://chatgpt.com/codex`.
- `src/App.tsx`: shell, header, empty state y footer reescritos con el mismo sistema visual.
- `src/components/AccountForm.tsx`: modal/formulario alineado al mismo estilo compacto oscuro.
- `src/components/AccountCard.tsx`: tarjeta, menú, detalles y `Uso restante` alineados al mismo lenguaje visual.
- `src/index.css`: tipografía base y estilos de controles unificados.
- `src/store/useStore.ts`: listener Tauri protegido para que la UI también pueda renderizarse en navegador normal durante QA.
- `src/components/AccountCard.tsx`: reemplazadas barras de uso por bloque compacto estilo Codex (`Uso restante`, `5 h`, `Semanal`).
- `src/store/useStore.ts`: las sesiones capturadas por proxy se muestran automáticamente como cuentas detectadas aunque no estén configuradas manualmente.
- `src-tauri/src/proxy.rs`: `find_script()` ahora encuentra `proxy/gpt_interceptor.py` en modo dev.
- `src/store/useStore.ts`: puerto por defecto migrado de `8080` a `18080`; fallback para mostrar una sesión capturada cuando todavía no hay email emparejado.
- `proxy/gpt_interceptor.py`: agregado `/backend-api/wham/usage`, parser para `rate_limit.primary_window` (5h) y `secondary_window` (semana), hash estable de sesión y sanitización de `usage_raw`.
- `src/store/useStore.ts`: estado inicial sin cuentas demo, migración para remover demos persistidas, activación automática de la primera cuenta real.
- `src/App.tsx`, `src/components/AccountCard.tsx`, `src/components/AccountForm.tsx`: copy ajustado a cuentas Codex, uso semanal y ventana de 5 horas.
- `src-tauri/tauri.conf.json`: removido `app.trayIcon` declarativo para evitar duplicar el tray `main-tray`; el tray activo queda creado en Rust con `on_tray_icon_event`.
- `src-tauri/capabilities/default.json`: agregado permiso mínimo para la ventana `popover` (`core:default`, `core:event:allow-listen`, `core:event:allow-unlisten`).
- `src/App.tsx`, `src/components/AccountForm.tsx`: imports de interfaces cambiados a `import type`.
- `src-tauri/tauri.conf.json`: removidos campos de plugin incompatibles con el esquema actual de Tauri y referencias a iconos vacíos.
- `src-tauri/icons/32x32.png`, `128x128.png`, `128x128@2x.png`: convertidos a RGBA.
- `src-tauri/icons/tray.png`: agregado desde el icono 32x32 RGBA requerido por `trayIcon.iconPath`.
- `src/components/AccountCard.tsx`, `src/store/useStore.ts`: imports de interfaces cambiados a `import type`.

## Estado actual
`npm run tauri:dev` queda corriendo. Vite responde en `http://localhost:5173/` con HTTP 200 y el binario Tauri está conectado al webview sin errores nuevos en la terminal.

## Qué funciona
El build Rust compila, la app arranca en modo dev, el tray activo se crea desde Rust y el frontend ya no muestra cuentas demo.
El proxy captura `/backend-api/wham/usage`; `usage.json` quedó con una cuenta y porcentajes válidos de semana/5h.
La UI ahora muestra porcentajes restantes, no consumidos, para evitar confusión con Codex.
La pantalla principal carga en navegador interno con copy coherente (`Codex Account Router`, `Uso restante`) y sin errores recientes de consola.
Las cuentas se actualizan cada 60s y se deduplican por cuenta para evitar cards repetidas.

## Qué falta
Quedan warnings de imports Rust no usados (`Manager`, `Runtime`) que no bloquean el arranque.
Validar manualmente el click del icono en la menu bar en macOS, porque requiere interacción GUI del usuario.
`npm run build` todavía falla en Vite porque falta `esbuild` como dependencia; `npx tsc --noEmit` pasa correctamente.

## Riesgos pendientes
Los permisos removidos de `plugins.fs` y `plugins.notification` estaban en un formato inválido para Tauri 2; si la app necesita esos permisos desde frontend, deben declararse correctamente en capabilities.
El proxy del sistema está apuntando a `127.0.0.1:18080`; hay otro proceso ajeno (`lazy_proxy.py`) usando `8080`, por eso GPT Router debe seguir usando `18080`.

## Siguiente paso recomendado
Revisar capabilities de Tauri para FS/notification si alguna funcionalidad frontend depende de esos plugins.

## Notas para el siguiente agente
No se agregaron dependencias ni secretos. El cambio fue mínimo para destrabar el arranque dev.
