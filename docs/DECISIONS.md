# DECISIONS

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
Decisión: Abrir una consola explícita con `cmd.exe /K` y ejecutar `call "codex-auth.cmd" login --device-auth` en Windows.
Motivo: Kuota compila como app GUI en Windows (`windows_subsystem = "windows"`), por lo que lanzar directamente `codex-auth` con `Command::spawn` no garantiza una consola visible para mostrar el código de device auth ni permitir interacción.
Alternativas descartadas: Usar `hidden_command`, porque ocultaría el flujo interactivo; ejecutar `codex-auth.cmd` directo, porque no resuelve el problema de consola visible desde una app GUI; agregar una UI propia de auth, porque sería más grande y frágil que delegar al CLI existente.
Impacto: El botón de nueva cuenta en Windows abre una ventana de cmd persistente con el flujo de `codex-auth`; los comandos automáticos siguen ocultos.

## Formato

```txt
Fecha:
Decisión:
Motivo:
Alternativas descartadas:
Impacto:
```
