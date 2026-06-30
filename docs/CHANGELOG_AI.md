# CHANGELOG_AI

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
