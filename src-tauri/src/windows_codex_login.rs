#![cfg(target_os = "windows")]

use serde::Serialize;
use serde_json::Value;
use std::{
    fs,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use tauri::{AppHandle, Emitter};

use crate::{
    apply_extended_path, hidden_command, locate_codex_auth, locate_command, CREATE_NO_WINDOW,
};

const LOGIN_EVENT: &str = "codex-login-event";

lazy_static::lazy_static! {
    static ref LOGIN_CHILD: Mutex<Option<Child>> = Mutex::new(None);
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowsCodexLoginEvent {
    event: &'static str,
    code: Option<String>,
    url: Option<String>,
    message: Option<String>,
}

#[derive(Default)]
struct DetectionState {
    code: Option<String>,
    url: Option<String>,
    browser_opened: bool,
    code_copied: bool,
    awaiting_code_lines: u8,
    last_error: Option<String>,
}

pub struct WindowsCodexLoginService;

impl WindowsCodexLoginService {
    pub fn start(app: AppHandle, device_auth: bool) -> Result<(), String> {
        let mut active_child = LOGIN_CHILD
            .lock()
            .map_err(|_| "No se pudo iniciar el login de Codex.".to_string())?;
        if active_child.is_some() {
            return Err("Ya hay un login de Codex en curso.".to_string());
        }

        let launcher = resolve_codex_auth_launcher()?;
        let mut command = Command::new(&launcher.program);
        command.args(&launcher.args);
        command.arg("login");
        if device_auth {
            command.arg("--device-auth");
        }
        apply_extended_path(&mut command);
        command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        use std::os::windows::process::CommandExt;
        command.creation_flags(CREATE_NO_WINDOW);

        let mut child = command
            .spawn()
            .map_err(|_| "No se pudo iniciar el login de Codex.".to_string())?;

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        *active_child = Some(child);
        drop(active_child);

        emit(
            &app,
            WindowsCodexLoginEvent {
                event: "LoginStarted",
                code: None,
                url: None,
                message: None,
            },
        );
        emit(
            &app,
            WindowsCodexLoginEvent {
                event: "WaitingForAuthorization",
                code: None,
                url: None,
                message: None,
            },
        );

        let detection = Arc::new(Mutex::new(DetectionState::default()));
        if let Some(stdout) = stdout {
            read_output(app.clone(), detection.clone(), stdout);
        }
        if let Some(stderr) = stderr {
            read_output(app.clone(), detection.clone(), stderr);
        }

        thread::spawn(move || loop {
            let status = {
                let mut guard = match LOGIN_CHILD.lock() {
                    Ok(guard) => guard,
                    Err(_) => return,
                };
                let Some(child) = guard.as_mut() else {
                    return;
                };

                match child.try_wait() {
                    Ok(Some(status)) => {
                        let _ = guard.take();
                        Some(status)
                    }
                    Ok(None) => None,
                    Err(_) => {
                        let _ = guard.take();
                        emit(
                            &app,
                            WindowsCodexLoginEvent {
                                event: "LoginFailed",
                                code: None,
                                url: None,
                                message: Some(
                                    "No se pudo completar la autorizacion de Codex.".to_string(),
                                ),
                            },
                        );
                        return;
                    }
                }
            };

            if let Some(status) = status {
                if status.success() {
                    emit(
                        &app,
                        WindowsCodexLoginEvent {
                            event: "LoginSucceeded",
                            code: None,
                            url: None,
                            message: Some("Cuenta agregada correctamente.".to_string()),
                        },
                    );
                } else {
                    emit(
                        &app,
                        WindowsCodexLoginEvent {
                            event: "LoginFailed",
                            code: None,
                            url: None,
                            message: Some(login_error_message(&detection)),
                        },
                    );
                }
                return;
            }

            thread::sleep(Duration::from_millis(250));
        });

        Ok(())
    }

    pub fn cancel(app: AppHandle) -> Result<(), String> {
        let child = LOGIN_CHILD
            .lock()
            .map_err(|_| "No se pudo cancelar el login de Codex.".to_string())?
            .take();

        if let Some(mut child) = child {
            let _ = child.kill();
            let _ = child.wait();
            emit(
                &app,
                WindowsCodexLoginEvent {
                    event: "LoginCancelled",
                    code: None,
                    url: None,
                    message: Some("Login cancelado.".to_string()),
                },
            );
        }

        Ok(())
    }

    pub fn open_authorization_url(url: String) -> Result<(), String> {
        open_url(&url)
    }
}

struct CodexAuthLauncher {
    program: PathBuf,
    args: Vec<PathBuf>,
}

fn resolve_codex_auth_launcher() -> Result<CodexAuthLauncher, String> {
    let path = locate_codex_auth().ok_or_else(|| "codex-auth no esta instalado".to_string())?;

    if path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.eq_ignore_ascii_case("exe"))
        .unwrap_or(false)
    {
        return Ok(CodexAuthLauncher {
            program: path,
            args: vec![],
        });
    }

    let node = locate_node_exe()
        .ok_or_else(|| "Node.js no esta instalado o no esta en PATH.".to_string())?;
    let script = locate_codex_auth_script(&path)
        .ok_or_else(|| "No se pudo resolver codex-auth sin abrir consola.".to_string())?;

    Ok(CodexAuthLauncher {
        program: node,
        args: vec![script],
    })
}

fn locate_node_exe() -> Option<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(path) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path) {
            candidates.push(dir.join("node.exe"));
        }
    }

    if let Ok(program_files) = std::env::var("ProgramFiles") {
        candidates.push(PathBuf::from(program_files).join("nodejs").join("node.exe"));
    }

    if let Ok(program_files_x86) = std::env::var("ProgramFiles(x86)") {
        candidates.push(
            PathBuf::from(program_files_x86)
                .join("nodejs")
                .join("node.exe"),
        );
    }

    candidates.into_iter().find(|path| path.exists())
}

fn locate_codex_auth_script(command_path: &Path) -> Option<PathBuf> {
    let mut package_roots = Vec::new();

    if let Some(prefix) = command_path.parent() {
        package_roots.push(
            prefix
                .join("node_modules")
                .join("@loongphy")
                .join("codex-auth"),
        );
    }

    if let Ok(appdata) = std::env::var("APPDATA") {
        package_roots.push(
            PathBuf::from(appdata)
                .join("npm")
                .join("node_modules")
                .join("@loongphy")
                .join("codex-auth"),
        );
    }

    let mut npm_root_command = hidden_command(locate_command("npm")?);
    npm_root_command.args(["root", "-g"]);
    apply_extended_path(&mut npm_root_command);
    if let Ok(output) = npm_root_command.output() {
        if output.status.success() {
            let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !root.is_empty() {
                package_roots.push(PathBuf::from(root).join("@loongphy").join("codex-auth"));
            }
        }
    }

    package_roots
        .into_iter()
        .find_map(|root| read_codex_auth_bin(&root))
}

fn read_codex_auth_bin(package_root: &Path) -> Option<PathBuf> {
    let package_json = package_root.join("package.json");
    let content = fs::read_to_string(package_json).ok()?;
    let json = serde_json::from_str::<Value>(&content).ok()?;
    let bin = json.get("bin")?;

    let bin_path = if let Some(path) = bin.as_str() {
        path
    } else {
        bin.get("codex-auth")
            .or_else(|| bin.as_object().and_then(|object| object.values().next()))?
            .as_str()?
    };

    let script = package_root.join(bin_path);
    script.exists().then_some(script)
}

fn read_output<R>(app: AppHandle, detection: Arc<Mutex<DetectionState>>, reader: R)
where
    R: std::io::Read + Send + 'static,
{
    thread::spawn(move || {
        let reader = BufReader::new(reader);
        for line in reader.lines().map_while(Result::ok) {
            inspect_line(&app, &detection, &strip_ansi(&line));
        }
    });
}

fn inspect_line(app: &AppHandle, detection: &Arc<Mutex<DetectionState>>, line: &str) {
    if let Some(message) = classify_error(line) {
        if let Ok(mut state) = detection.lock() {
            state.last_error = Some(message);
        }
    }

    let code_context = update_code_context(detection, line);

    if let Some(url) = detect_url(line) {
        let mut url_detected = false;
        let mut url_to_open = None;

        let mut state = match detection.lock() {
            Ok(state) => state,
            Err(_) => return,
        };
        if state.url.as_deref() != Some(&url) {
            state.url = Some(url.clone());
            url_detected = true;
        }

        if !state.browser_opened && state.code_copied {
            state.browser_opened = true;
            url_to_open = Some(url.clone());
        }
        drop(state);

        if url_detected {
            emit(
                app,
                WindowsCodexLoginEvent {
                    event: "AuthorizationUrlDetected",
                    code: None,
                    url: Some(url.clone()),
                    message: None,
                },
            );
        }

        if let Some(url) = url_to_open {
            open_url_and_emit(app, url);
        }
    }

    if let Some(code) = detect_code(line, code_context) {
        let mut code_detected = false;
        let mut copied = false;
        let mut url_to_open = None;

        let mut state = match detection.lock() {
            Ok(state) => state,
            Err(_) => return,
        };
        if state.code.as_deref() != Some(&code) {
            state.code = Some(code.clone());
            code_detected = true;
            state.awaiting_code_lines = 0;
            copied = copy_code_to_clipboard(&code).is_ok();
            state.code_copied = copied;
            if copied && !state.browser_opened {
                url_to_open = state.url.clone();
                state.browser_opened = url_to_open.is_some();
            }
        }
        drop(state);

        if code_detected {
            emit(
                app,
                WindowsCodexLoginEvent {
                    event: "AuthorizationCodeDetected",
                    code: Some(code),
                    url: None,
                    message: if copied {
                        Some("Codigo copiado. Abriendo OpenAI...".to_string())
                    } else {
                        Some("Copia el codigo para continuar.".to_string())
                    },
                },
            );
        }

        if let Some(url) = url_to_open {
            open_url_and_emit(app, url);
        }
    }
}

fn update_code_context(detection: &Arc<Mutex<DetectionState>>, line: &str) -> bool {
    let has_hint = line_suggests_authorization_code(line);
    let mut state = match detection.lock() {
        Ok(state) => state,
        Err(_) => return has_hint,
    };

    let in_context = has_hint || state.awaiting_code_lines > 0;
    if has_hint {
        state.awaiting_code_lines = 3;
    } else if state.awaiting_code_lines > 0 && !line.trim().is_empty() {
        state.awaiting_code_lines -= 1;
    }

    in_context
}

fn login_error_message(detection: &Arc<Mutex<DetectionState>>) -> String {
    detection
        .lock()
        .ok()
        .and_then(|state| state.last_error.clone())
        .unwrap_or_else(|| "No se pudo completar la autorizacion de Codex.".to_string())
}

fn classify_error(line: &str) -> Option<String> {
    let lower = line.to_lowercase();

    if lower.contains("access_token")
        || lower.contains("refresh_token")
        || lower.contains("auth.json")
        || lower.contains("cookie")
    {
        return None;
    }

    if lower.contains("codex-auth")
        && (lower.contains("not found") || lower.contains("no se reconoce"))
    {
        return Some(
            "codex-auth no esta instalado. Kuota intentara instalarlo automaticamente.".to_string(),
        );
    }

    if (lower.contains("`codex`") || lower.contains("codex executable"))
        && (lower.contains("not found") || lower.contains("was not found"))
    {
        return Some(
            "Codex CLI no esta instalado. Kuota intentara instalarlo automaticamente.".to_string(),
        );
    }

    if lower.contains("node") && (lower.contains("not found") || lower.contains("no se reconoce")) {
        return Some("Node.js no esta disponible para ejecutar codex-auth.".to_string());
    }

    if lower.contains("enotfound")
        || lower.contains("econn")
        || lower.contains("network")
        || lower.contains("fetch failed")
    {
        return Some("No se pudo conectar con el servicio de autorizacion.".to_string());
    }

    if lower.contains("429") || lower.contains("too many requests") {
        return Some(
            "OpenAI limito temporalmente los intentos de generar codigo. Espera unos minutos antes de reintentar.".to_string(),
        );
    }

    if lower.contains("eacces") || lower.contains("permission") || lower.contains("denied") {
        return Some("Windows bloqueo el proceso de autorizacion de Codex.".to_string());
    }

    if lower.contains("error") || lower.contains("failed") {
        return Some("codex-auth reporto un error durante el login.".to_string());
    }

    None
}

fn detect_url(line: &str) -> Option<String> {
    line.split_whitespace()
        .map(clean_token)
        .find(|token| token.starts_with("https://") || token.starts_with("http://"))
}

fn detect_code(line: &str, code_context: bool) -> Option<String> {
    let tokens = line.split_whitespace().map(clean_token).collect::<Vec<_>>();

    tokens
        .iter()
        .find(|token| is_strong_code_candidate(token))
        .cloned()
        .or_else(|| {
            code_context.then(|| {
                tokens
                    .iter()
                    .rev()
                    .find(|token| is_context_code_candidate(token))
                    .cloned()
            })?
        })
}

fn line_suggests_authorization_code(line: &str) -> bool {
    let lower = line.to_lowercase();
    lower.contains("device code")
        || lower.contains("authorization code")
        || lower.contains("one-time code")
        || lower.contains("enter this code")
        || lower.contains("codigo de autorizacion")
        || lower.contains("codigo")
}

fn is_strong_code_candidate(token: &str) -> bool {
    if !is_basic_code_token(token) {
        return false;
    }

    let has_separator = token.contains('-') || token.contains('_');
    let has_uppercase = token.chars().any(|char| char.is_ascii_uppercase());
    let has_digit = token.chars().any(|char| char.is_ascii_digit());
    let has_multiple_groups = token
        .split(['-', '_'])
        .filter(|group| group.len() >= 2)
        .count()
        >= 2;

    (has_separator && has_multiple_groups && (has_uppercase || has_digit))
        || (has_uppercase && has_digit && token.len() >= 6)
}

fn is_context_code_candidate(token: &str) -> bool {
    if !is_basic_code_token(token) || is_code_stopword(token) {
        return false;
    }

    let has_separator = token.contains('-') || token.contains('_');
    let has_uppercase = token.chars().any(|char| char.is_ascii_uppercase());
    let has_lowercase = token.chars().any(|char| char.is_ascii_lowercase());
    let has_digit = token.chars().any(|char| char.is_ascii_digit());
    let has_multiple_groups = token
        .split(['-', '_'])
        .filter(|group| group.len() >= 2)
        .count()
        >= 2;

    (has_separator && has_multiple_groups)
        || (token.len() >= 6 && (has_digit || (has_uppercase && !has_lowercase)))
}

fn is_basic_code_token(token: &str) -> bool {
    if token.len() < 5
        || token.len() > 64
        || token.contains("://")
        || token.contains('.')
        || token.contains('@')
    {
        return false;
    }

    token
        .chars()
        .all(|char| char.is_ascii_alphanumeric() || char == '-' || char == '_')
}

fn is_code_stopword(token: &str) -> bool {
    matches!(
        token.to_lowercase().as_str(),
        "device"
            | "authorization"
            | "authorisation"
            | "chatgpt"
            | "openai"
            | "codex"
            | "sign"
            | "signin"
            | "login"
            | "codigo"
            | "code"
            | "codes"
            | "enter"
            | "expires"
            | "minutes"
            | "one-time"
            | "using"
            | "common"
            | "phishing"
            | "target"
            | "never"
            | "share"
            | "browser"
            | "account"
    )
}

fn clean_token(token: &str) -> String {
    token
        .trim_matches(|char: char| {
            matches!(
                char,
                '"' | '\''
                    | '`'
                    | '('
                    | ')'
                    | '['
                    | ']'
                    | '{'
                    | '}'
                    | '<'
                    | '>'
                    | ','
                    | ';'
                    | ':'
                    | '!'
                    | '?'
            )
        })
        .trim_end_matches('.')
        .to_string()
}

fn strip_ansi(value: &str) -> String {
    let mut output = String::new();
    let mut chars = value.chars().peekable();
    while let Some(char) = chars.next() {
        if char == '\u{1b}' {
            for next in chars.by_ref() {
                if next.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            output.push(char);
        }
    }
    output
}

fn open_url(url: &str) -> Result<(), String> {
    hidden_command("rundll32.exe")
        .args(["url.dll,FileProtocolHandler", url])
        .spawn()
        .map(|_| ())
        .map_err(|_| "No se pudo abrir el navegador automaticamente.".to_string())
}

fn open_url_and_emit(app: &AppHandle, url: String) {
    if open_url(&url).is_ok() {
        emit(
            app,
            WindowsCodexLoginEvent {
                event: "BrowserOpened",
                code: None,
                url: Some(url),
                message: None,
            },
        );
    }
}

fn copy_code_to_clipboard(code: &str) -> Result<(), String> {
    let status = hidden_command("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "Set-Clipboard -Value $args[0]",
            code,
        ])
        .status()
        .map_err(|_| "No se pudo copiar el codigo automaticamente.".to_string())?;

    status
        .success()
        .then_some(())
        .ok_or_else(|| "No se pudo copiar el codigo automaticamente.".to_string())
}

fn emit(app: &AppHandle, event: WindowsCodexLoginEvent) {
    let _ = app.emit(LOGIN_EVENT, event);
}
