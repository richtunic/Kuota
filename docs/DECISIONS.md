# DECISIONS

Fecha: 2026-07-01
Decisión: Publicar el login nativo macOS/Windows como release `1.0.4`.
Motivo: El login oculto, la copia automatica del codigo y el manejo de errores de device auth son cambios visibles que deben llegar por autoupdate con version propia.
Alternativas descartadas: Mantener `1.0.3`, porque dificultaria distinguir usuarios con el flujo anterior; publicar Linux, porque sigue sin validacion activa.
Impacto: Se actualizan manifests, lockfiles, modal About y documentacion; el tag `v1.0.4` debe compilar solo macOS y Windows desde GitHub Actions.

Fecha: 2026-06-30
Decisión: Mantener releases solo para macOS y Windows y documentar cambios por version.
Motivo: Linux queda sin validacion activa hasta nuevo aviso; cada version nueva debe explicar claramente cambios funcionales, fixes y nuevas funciones para que el usuario entienda que recibira.
Alternativas descartadas: Publicar Linux por inercia, porque comunicaria soporte no probado; subir versiones sin commits descriptivos/notas, porque dificulta soporte y autoupdate.
Impacto: El workflow de release sigue sin Linux; antes de subir una version nueva se deben crear commits descriptivos y notas/release body con cambios relevantes para el usuario.

Fecha: 2026-06-30
Decisión: Portar el login nativo oculto de Codex tambien a macOS.
Motivo: El output de `codex-auth login --device-auth` en macOS es estable y capturable; mantener Terminal visible crea una experiencia distinta a Windows y obliga al usuario a copiar manualmente.
Alternativas descartadas: Mantener Terminal como flujo principal, porque ya existe UI nativa y copia automatica; unificar Windows/macOS en un solo modulo ahora, porque duplicar servicios por plataforma reduce riesgo mientras el flujo Windows esta recien estabilizado.
Impacto: macOS usa `MacCodexLoginService` con stdout/stderr capturados, `pbcopy`, `open` y cancelacion de proceso hijo; Windows queda sin cambios.

Fecha: 2026-06-30
Decisión: Reemplazar el login visible en consola por un servicio Windows oculto con UI nativa.
Motivo: La experiencia deseada es que el usuario no vea CMD/PowerShell; `codex-auth` puede ejecutarse como proceso hijo oculto mientras Kuota captura URL/codigo y presenta el flujo en la interfaz.
Alternativas descartadas: Mantener `cmd.exe /K`, porque expone una consola y obliga al usuario a interactuar fuera de Kuota; incrustar OpenAI en WebView, porque el flujo debe abrir el navegador del sistema; modificar macOS ahora, porque el alcance pedido es solo Windows.
Impacto: Windows usa `WindowsCodexLoginService` con `CREATE_NO_WINDOW`, eventos Tauri y cancelacion del proceso hijo; macOS queda sin cambios.

Fecha: 2026-06-30
Decisión: Tratar Codex CLI y su binario nativo como dependencia requerida del login Windows.
Motivo: La prueba en Windows 11 ARM mostro que `codex-auth login --device-auth` falla si no existe el ejecutable `codex` en `PATH`, aunque `@loongphy/codex-auth` este instalado.
Alternativas descartadas: Pedir instalacion manual de Codex CLI, porque rompe la experiencia nativa de Kuota; capturar y mostrar la consola, porque el objetivo Windows es no exponer CMD/PowerShell.
Impacto: En Windows, `ensure_codex_auth` y `codex_auth_login` instalan `@openai/codex@latest` si falta, agregan el `bin` vendor de `codex.exe` al PATH extendido y escriben `PATH`/`Path` al lanzar procesos hijos; macOS queda sin cambios.

Fecha: 2026-06-30
Decisión: Suspender builds Linux en releases hasta nuevo aviso.
Motivo: El foco de prueba y soporte actual es macOS y Windows; publicar Linux sin validación puede comunicar soporte que todavía no se está manteniendo.
Alternativas descartadas: Mantener Linux en el matrix, porque generaría assets no validados; borrar soporte de código, porque no es necesario para dejar de publicar builds.
Impacto: `.github/workflows/release.yml` solo genera assets para macOS y Windows.

Fecha: 2026-06-30
Decisión: Publicar los cambios como release `1.0.3`.
Motivo: Los cambios corrigen comportamiento visible y de plataforma en Windows/macOS, y deben llegar por el flujo de releases/autoupdate existente.
Alternativas descartadas: Esperar a una release mayor, porque el fix de login en Windows desbloquea una función principal; cambiar el workflow de release, porque el flujo actual por tag `v*` ya cubre el caso.
Impacto: Se actualizan manifests y modal About a `1.0.3`; al empujar el tag `v1.0.3`, GitHub Actions generará los assets de release.

Fecha: 2026-06-30
Decisión: Posicionar el popover con la coordenada real del evento de tray y mostrar progreso usando eventos nativos del updater.
Motivo: El posicionamiento hardcodeado al borde derecho descentraba la ventana respecto al icono en macOS; el plugin updater ya entrega `Started`, `Progress` y `Finished`, así que no hace falta simular progreso.
Alternativas descartadas: Mantener offsets fijos por plataforma, porque falla con diferentes posiciones de iconos/monitores; crear un backend propio de updater, porque el plugin existente ya expone progreso.
Impacto: La ventana se centra respecto al icono al desplegarse y Ajustes muestra una barra de descarga cuando hay actualización.

Fecha: 2026-06-30
Decisión: Reemplazada: abrir una consola explícita con `cmd.exe /K` y ejecutar `call "codex-auth.cmd" login --device-auth` en Windows.
Motivo: Kuota compila como app GUI en Windows (`windows_subsystem = "windows"`), por lo que lanzar directamente `codex-auth` con `Command::spawn` no garantiza una consola visible para mostrar el código de device auth ni permitir interacción.
Alternativas descartadas: Usar `hidden_command`, porque ocultaría el flujo interactivo; ejecutar `codex-auth.cmd` directo, porque no resuelve el problema de consola visible desde una app GUI; agregar una UI propia de auth, porque sería más grande y frágil que delegar al CLI existente.
Impacto: Esta decision quedo superada por `WindowsCodexLoginService`, que ejecuta `codex-auth` oculto y presenta URL/codigo dentro de Kuota.

## Formato

```txt
Fecha:
Decisión:
Motivo:
Alternativas descartadas:
Impacto:
```
