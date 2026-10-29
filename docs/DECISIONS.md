# DECISIONS

Fecha: 2026-10-29
Decisión: Publicar la espera y verificación de reapertura de Desktop en Kuota 1.0.12.
Motivo: El usuario confirmó que el cambio de cuenta con la compilación local ahora vuelve a abrir ChatGPT/Codex. El arreglo necesita llegar a usuarios instalados mediante el actualizador firmado.
Alternativas descartadas: Mantener 1.0.11, porque los clientes instalados no recibirían el arreglo.
Impacto: Versiones en npm, Cargo, Tauri y Acerca de; notas bilingües y publicación por el tag v1.0.12 para builds macOS/Windows y latest.json.

Fecha: 2026-10-29
Decisión: Esperar el cierre real de Desktop y comprobar su proceso tras la apertura en macOS.
Motivo: La espera fija de 800 ms permite enviar open mientras Electron aún termina; un código cero de open no demuestra que se haya iniciado otro proceso. Es un defecto verificable del flujo anterior, compatible con el síntoma reportado; la causa concreta en la máquina no se ha confirmado con reproducción.
Alternativas descartadas: Aumentar otra espera fija o forzar el cierre, porque no resuelven la sincronización de forma fiable o podrían interrumpir trabajo.
Impacto: Esperas acotadas, propagación de errores y compatibilidad con nombres existentes. El cambio de cuenta no se revierte si falla la reapertura. Windows conserva su comportamiento previo.

Fecha: 2026-10-22
Decisión: Durante el login exitoso, revelar la cuenta autenticada si Kuota la mantenía oculta localmente.
Motivo: El estado local de la app instalada puede tener claves ocultas que no existen en el almacenamiento de desarrollo; eso hacía parecer que la cuenta se añadió en debug pero no en la app instalada.
Alternativas descartadas: Borrar todas las claves ocultas después de cualquier login, porque volvería a mostrar cuentas que el usuario había quitado deliberadamente.
Impacto: El login revela la cuenta activa y las cuentas nuevas detectadas durante ese flujo; las demás cuentas ocultas se conservan. Publicado como Kuota 1.0.11.

Fecha: 2026-10-22
Decisión: Publicar las correcciones de cuenta activa, ocultamiento persistente y refresco de login como Kuota `1.0.10`.
Motivo: Son correcciones visibles que deben llegar a usuarios instalados mediante el actualizador firmado.
Alternativas descartadas: Mantener la versión `1.0.9`, porque los clientes no detectarían la actualización.
Impacto: Versiones sincronizadas en npm, Cargo, Tauri y Acerca de Kuota; notas bilingües en `docs/releases/v1.0.10.md`; el tag `v1.0.10` dispara builds oficiales de macOS y Windows.

Fecha: 2026-10-22
Decisión: Tratar el marcador activo reportado por `codex-auth` como fuente de verdad y persistir los borrados de cuentas detectadas por correo normalizado.
Motivo: Conservar un `activeAccountId` anterior podia marcar como activa otra cuenta cuando el estado actual no encontraba una coincidencia; borrar una cuenta detectada por un ID sin poder resolver su correo podia dejarla visible despues del siguiente refresco.
Alternativas descartadas: Mantener la ultima cuenta activa localmente, porque puede contradecir a Codex; eliminar la cuenta de `codex-auth`, porque el usuario pidio ocultarla de Kuota y esa accion afectaria la autenticacion.
Impacto: La marca activa se recalcula desde cada respuesta actual, no se guarda como preferencia y las cuentas ocultas siguen filtradas aunque se resuelvan desde IDs generados por Kuota.

Fecha: 2026-10-22
Decisión: Reintentar la lectura de cuentas durante 45 segundos despues de completar el login nativo.
Motivo: La confirmacion de device auth en el navegador puede preceder a que `codex-auth list` refleje la cuenta; unicamente consultar en ese instante y 1.8 segundos despues podia dejar Kuota sin mostrarla hasta el polling de un minuto.
Alternativas descartadas: Cambiar el login o guardar tokens manualmente, porque `codex-auth` sigue siendo la fuente de autenticacion y ya informa el exito del flujo.
Impacto: Kuota refresca al momento y a los 5, 15 y 45 segundos tras el exito; no cambia la gestion de credenciales.

Fecha: 2026-09-16
Decisión: Publicar la correccion de cuotas como v1.0.9.
Motivo: El usuario solicita actualizar GitHub con una nueva version y sus notas; el actualizador necesita un incremento de version.
Impacto: Notas bilingues, manifests y lockfiles sincronizados; se conserva el workflow de releases y no se agregan dependencias.

Fecha: 2026-09-16
Decisión: Leer las columnas tabuladas de codex-auth 0.3.0 por encabezado y conservar el formato legado.
Motivo: 0.3.0 agrega CREDITS/RESET CREDITS y renombra 5H USAGE/WEEKLY USAGE a 5H/WEEKLY; el parser anterior confundia creditos con cuotas.
Alternativas descartadas: Migrar toda la integracion a JSON en este hotfix, porque requiere mapear resets, actividad y estados de refresco y mantener compatibilidad con versiones antiguas sin JSON.
Impacto: Recupera las cuotas actuales con un cambio localizado. Filas incompletas y errores de autorizacion no habilitan candidatas ni disparan auto-switch por agotamiento. La tabla sigue siendo una dependencia de formato externo.

Fecha: 2026-09-02
Decisión: Publicar la compatibilidad de relanzamiento con ChatGPT Desktop como `v1.0.8`.
Motivo: El cambio corrige un comportamiento visible despues de seleccionar una cuenta y debe llegar mediante el actualizador firmado con notas bilingues.
Alternativas descartadas: Mantener `1.0.7`, porque los clientes instalados no detectarian el arreglo; incluir renovacion OAuth propia, porque el fix upstream de snapshots caducados sigue sin release oficial y requiere una revision de seguridad separada.
Impacto: Se actualizan manifests, lockfiles, el modal Acerca de Kuota y `docs/releases/v1.0.8.md`; la publicacion queda pendiente de commit/tag/push.

Fecha: 2026-09-02
Decisión: Identificar Codex/ChatGPT Desktop por el bundle estable en macOS y por el proceso realmente activo en Windows.
Motivo: La app instalada actualmente se llama `ChatGPT.app`, pero conserva `com.openai.codex`; depender solo de `Codex.app` hacia que el cambio de cuenta funcionara y el relanzamiento apuntara a una app inexistente.
Alternativas descartadas: Reabrir siempre por el nombre `ChatGPT`, porque rompe instalaciones anteriores; cerrar todas las variantes, porque puede afectar procesos no seleccionados; modificar `codex-auth`, porque el cambio de credenciales ya funciona.
Impacto: Kuota mantiene compatibilidad con el nombre anterior y el actual, reabre la variante Desktop detectada y conserva intactas las sesiones CLI.

Fecha: 2026-08-31
Decisión: Reemplazar la seleccion automatica nativa de `codex-auth` por una seleccion estricta controlada por Kuota.
Motivo: `codex-auth 0.2.10` puede puntuar una candidata sin snapshot como `100%`; eso no garantiza que la cuenta destino tenga cuota real. Kuota debe cambiar solo cuando la semanal es valida y mayor que cero y, si existe la columna de 5 horas, tambien contiene un porcentaje valido mayor que cero.
Alternativas descartadas: Validar despues del cambio, porque la autenticacion ya habria cambiado; consultar la API cada segundo, porque aumenta carga y riesgo de limites; eliminar cuentas sin datos del registro, porque seria destructivo.
Impacto: Kuota deshabilita el auto-switch nativo al usar su opcion, observa localmente eventos `rate_limits`, valida candidatas en el backend antes del switch y conserva la cuenta activa si ninguna cumple. El polling de 60 segundos sigue como respaldo y el switch queda apagado por defecto.

Fecha: 2026-08-31
Decisión: Publicar las mejoras de cuotas, localizacion, layout y auto-switch estricto como `v1.0.7`.
Motivo: Son cambios visibles y de comportamiento que deben llegar mediante el actualizador firmado con notas bilingues.
Alternativas descartadas: Mantener `1.0.6`, porque los clientes no detectarian una nueva release; publicar sin notas explicitas, porque la app filtra esos bloques por idioma.
Impacto: Se actualizan todos los manifests, el modal Acerca de y `docs/releases/v1.0.7.md`; el tag `v1.0.7` dispara builds oficiales macOS/Windows.

Fecha: 2026-08-31
Decisión: Aumentar el popover de 360x520 a 360x590 y localizar en frontend la actividad y las fechas de reinicio de `codex-auth`.
Motivo: Reañadir la ventana de 5 horas agrego un bloque vertical completo y provoco scroll en la pantalla principal. La actividad y fechas de reinicio llegan como texto ingles (`Now`, `ago`, `on 1 Sep`) aunque Kuota este configurado en español.
Alternativas descartadas: Comprimir las barras y textos, porque empeora legibilidad; ocultar detalles, porque elimina informacion util; traducir en Rust, porque el idioma es una preferencia de UI y el valor fuente debe conservarse.
Impacto: La vista normal muestra ambas cuotas sin scroll en el caso esperado; la actividad y fechas de reinicio respetan el idioma activo. Ajustes conserva su scroll independiente para contenido variable.

Fecha: 2026-08-31
Decisión: Limitar los modales a la altura disponible y desplazar solo su cuerpo, conservando el encabezado fijo.
Motivo: Ajustes puede crecer por la explicacion del auto-switch y la lista de cuentas; `overflow-hidden` sin altura maxima cortaba la informacion fuera del popup.
Alternativas descartadas: Aumentar la ventana principal, porque altera el popover completo; reducir u ocultar texto, porque perderia informacion util; agregar scrolls por cada seccion, porque produce desplazamiento anidado innecesario.
Impacto: Los modales cortos mantienen su tamaño y los largos quedan completamente accesibles mediante scroll vertical, sin cambiar el tamaño de Kuota.

Fecha: 2026-08-31
Decisión: Detectar la cuota de 5 horas por porcentaje valido y reaccionar a cambios locales de autenticacion mediante eventos Tauri.
Motivo: La documentacion oficial describe ventanas compartidas de cinco horas en planes ChatGPT, pero no publica una estimacion especifica para Free. En la prueba real, la cuenta Free aparecio en la columna `5H USAGE` con `401`; por eso ni el plan ni la mera presencia de la columna prueban una cuota util. Un porcentaje parseable si es evidencia operativa. Consultar la API cada segundo seria costoso y podria generar 401/429, mientras `auth.json` cambia localmente cuando el motor conmuta la cuenta.
Alternativas descartadas: Restringir 5 horas a Plus, porque excluiria una Free que reporte una ventana valida; mostrar cualquier texto de la columna, porque errores parecerian cuotas; reducir el polling API a segundos, porque aumenta carga sin crear streaming real; reiniciar o matar sesiones CLI, porque puede interrumpir trabajo activo.
Impacto: Cualquier plan muestra 5 horas cuando recibe `%`; errores o ausencia fallan de forma segura. El watcher de archivos emite un evento en menos de un segundo despues del cambio local. La seleccion automatica estricta queda definida por la decision posterior de Kuota. No se agregan dependencias.

Fecha: 2026-07-19
Decisión: Publicar la eliminacion visual de la cuota de 5 horas como `v1.0.6` y mantener el aviso humoristico solicitado en español e ingles en las notas de esta y futuras versiones.
Motivo: El cambio es visible para el usuario y debe distribuirse mediante autoupdate; las notas bilingues son la fuente mostrada por GitHub Releases y por el dialogo localizado de Kuota.
Alternativas descartadas: Publicar sin incrementar version, porque los clientes no recibirian el cambio; incluir el aviso en un solo idioma, porque romperia la experiencia bilingue vigente.
Impacto: Cada nueva nota en `docs/releases/v<version>.md` debe resumir los cambios reales e incluir "Se añadieron nuevos bugs para solucionar en proximas versiones." y "New bugs were added to be fixed in future versions." en sus bloques respectivos.

Fecha: 2026-07-19
Decisión: Mostrar solo la cuota semanal de Codex y conservar temporalmente el parseo interno de la columna de 5 horas.
Motivo: Codex concentro el limite visible en la cuota semanal, por lo que mostrar una ventana de 5 horas agrega ruido; el parser posicional de `codex-auth list` aun debe tolerar salidas que incluyan ambas columnas.
Alternativas descartadas: Eliminar de inmediato el campo y el parseo de 5 horas, porque podria desplazar la lectura de `weekly_usage` con versiones anteriores de `codex-auth`; mantener ambas cuotas visibles, porque ya no representa el producto actual.
Impacto: La UI y el README hablan solo de cuota semanal; no cambian autenticacion, switching de cuentas ni frecuencia de refresco.

Fecha: 2026-07-01
Decisión: Hacer obligatorias las release notes bilingues y mostrar en app solo el idioma activo.
Motivo: GitHub Releases debe comunicar cambios a usuarios en español e ingles, pero dentro de Kuota la experiencia debe respetar el idioma configurado por el usuario.
Alternativas descartadas: Publicar notas solo en un idioma, porque rompe soporte bilingue; mostrar ambos idiomas dentro de la app, porque aumenta ruido visual en un popup compacto.
Impacto: Cada release debe tener `docs/releases/<tag>.md` con bloques `<!-- kuota:es -->` y `<!-- kuota:en -->`; el workflow publica ambos y la app filtra el bloque local.

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
