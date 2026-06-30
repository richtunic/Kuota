#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod tray;

use serde::Serialize;
use std::path::PathBuf;
use std::process::Command;

const CODEX_AUTH_PACKAGE: &str = "@loongphy/codex-auth";
#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;
#[cfg(target_os = "windows")]
const CREATE_NEW_CONSOLE: u32 = 0x00000010;

#[derive(Debug, Clone, Copy)]
enum CodexTarget {
    Desktop,
    Cli,
    None,
}

#[derive(Debug, Clone, Serialize)]
struct CodexAuthAccount {
    email: String,
    active: bool,
    plan: Option<String>,
    five_hour_usage: Option<String>,
    weekly_usage: Option<String>,
    last_activity: Option<String>,
    raw: String,
}

#[derive(Debug, Clone, Serialize)]
struct CodexAuthStatus {
    installed: bool,
    path: Option<String>,
    version: Option<String>,
    latest_version: Option<String>,
    update_available: bool,
    status: Option<String>,
    accounts: Vec<CodexAuthAccount>,
    error: Option<String>,
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            #[cfg(target_os = "macos")]
            let _ = disable_system_proxy();

            tray::setup_tray(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if matches!(event, tauri::WindowEvent::Destroyed) {
                #[cfg(target_os = "macos")]
                let _ = disable_system_proxy();
            }
            if matches!(event, tauri::WindowEvent::Focused(false)) && window.label() == "popover" {
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            codex_auth_status,
            ensure_codex_auth,
            codex_auth_login,
            switch_codex_account,
            open_codex,
            quit_app,
        ])
        .run(tauri::generate_context!())
        .expect("error running Codex Account Router");
}

#[tauri::command]
async fn ensure_codex_auth() -> Result<CodexAuthStatus, String> {
    let latest = npm_latest_codex_auth_version().ok();

    if locate_codex_auth().is_none() {
        install_or_update_codex_auth()?;
    } else if let (Some(current), Some(latest_version)) =
        (installed_codex_auth_version(), latest.as_deref())
    {
        if normalize_version(&current) != normalize_version(latest_version) {
            install_or_update_codex_auth()?;
        }
    }

    Ok(read_codex_auth_status(latest))
}

#[tauri::command]
async fn codex_auth_status() -> CodexAuthStatus {
    read_codex_auth_status(npm_latest_codex_auth_version().ok())
}

#[tauri::command]
async fn switch_codex_account(selector: String) -> Result<CodexAuthStatus, String> {
    let path = locate_codex_auth().ok_or_else(|| "codex-auth no esta instalado".to_string())?;
    let target = detect_codex_target();
    run_command(path, &["switch", selector.as_str()])?;
    restart_codex_target(target);
    Ok(read_codex_auth_status(npm_latest_codex_auth_version().ok()))
}

#[tauri::command]
async fn codex_auth_login(device_auth: bool) -> Result<(), String> {
    let path = locate_codex_auth().ok_or_else(|| "codex-auth no esta instalado".to_string())?;

    #[cfg(target_os = "macos")]
    {
        let mut command = format!("{} login", shell_quote(&path.to_string_lossy()));
        if device_auth {
            command.push_str(" --device-auth");
        }
        command.push_str("; echo ''; echo 'Cuando termine el login, vuelve a Codex Account Router y presiona Actualizar cuentas.'");

        let script = format!(
            "tell application \"Terminal\"\ndo script {}\nactivate\nend tell",
            apple_script_string(&command)
        );

        std::process::Command::new("osascript")
            .args(["-e", &script])
            .status()
            .map_err(|error| {
                format!("No se pudo abrir Terminal para codex-auth login: {}", error)
            })?;
    }

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;

        let mut login_command =
            format!("call {} login", windows_cmd_quote(&path.to_string_lossy()));
        if device_auth {
            login_command.push_str(" --device-auth");
        }
        login_command.push_str(" & echo. & echo Cuando termine el login, vuelve a Kuota y presiona Actualizar cuentas.");

        Command::new("cmd.exe")
            .args(["/K", &login_command])
            .env("PATH", extended_path())
            .creation_flags(CREATE_NEW_CONSOLE)
            .spawn()
            .map_err(|error| format!("No se pudo abrir cmd para codex-auth login: {}", error))?;
    }

    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    {
        let mut command = std::process::Command::new(path);
        command.arg("login");
        if device_auth {
            command.arg("--device-auth");
        }
        command.spawn().map_err(|error| error.to_string())?;
    }

    Ok(())
}

#[tauri::command]
async fn open_codex() -> Result<(), String> {
    restart_codex_target(CodexTarget::Desktop);
    Ok(())
}

#[tauri::command]
fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn apple_script_string(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

#[cfg(target_os = "windows")]
fn windows_cmd_quote(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

#[cfg(target_os = "windows")]
fn hidden_command(program: impl AsRef<std::ffi::OsStr>) -> Command {
    let mut command = Command::new(program);
    use std::os::windows::process::CommandExt;
    command.creation_flags(CREATE_NO_WINDOW);
    command
}

#[cfg(not(target_os = "windows"))]
fn hidden_command(program: impl AsRef<std::ffi::OsStr>) -> Command {
    let command = Command::new(program);
    command
}

fn read_codex_auth_status(latest_version: Option<String>) -> CodexAuthStatus {
    let Some(path) = locate_codex_auth() else {
        return CodexAuthStatus {
            installed: false,
            path: None,
            version: None,
            latest_version,
            update_available: false,
            status: None,
            accounts: vec![],
            error: Some("codex-auth no esta instalado".to_string()),
        };
    };

    let version = run_command(path.clone(), &["--version"]).ok();
    let status = run_command(path.clone(), &["status"]).ok();
    let accounts = run_command(path.clone(), &["list"])
        .map(|output| parse_codex_auth_accounts(&output))
        .unwrap_or_default();

    let update_available = match (&version, &latest_version) {
        (Some(current), Some(latest)) => normalize_version(current) != normalize_version(latest),
        _ => false,
    };

    CodexAuthStatus {
        installed: true,
        path: Some(path.to_string_lossy().to_string()),
        version,
        latest_version,
        update_available,
        status,
        accounts,
        error: None,
    }
}

fn install_or_update_codex_auth() -> Result<(), String> {
    let npm = locate_command("npm")
        .ok_or_else(|| "npm no esta instalado o no esta en PATH".to_string())?;
    let output = hidden_command(npm)
        .args(["install", "-g", "@loongphy/codex-auth@latest"])
        .env("PATH", extended_path())
        .output()
        .map_err(|error| format!("No se pudo instalar {}: {}", CODEX_AUTH_PACKAGE, error))?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Err(if stderr.is_empty() { stdout } else { stderr })
}

fn npm_latest_codex_auth_version() -> Result<String, String> {
    let npm = locate_command("npm")
        .ok_or_else(|| "npm no esta instalado o no esta en PATH".to_string())?;
    run_command(npm, &["view", CODEX_AUTH_PACKAGE, "version"])
}

fn installed_codex_auth_version() -> Option<String> {
    let path = locate_codex_auth()?;
    run_command(path, &["--version"]).ok()
}

fn locate_codex_auth() -> Option<PathBuf> {
    locate_command("codex-auth")
}

fn locate_command(command: &str) -> Option<PathBuf> {
    let mut candidates = Vec::new();
    let command_names = command_variants(command);

    if let Ok(path) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path) {
            for command_name in &command_names {
                candidates.push(dir.join(command_name));
            }
        }
    }

    if let Ok(home) = std::env::var("HOME") {
        for command_name in &command_names {
            candidates.push(
                PathBuf::from(&home)
                    .join(".npm-global/bin")
                    .join(command_name),
            );
            candidates.push(PathBuf::from(&home).join(".local/bin").join(command_name));
            candidates.push(
                PathBuf::from(&home)
                    .join(".nvm/current/bin")
                    .join(command_name),
            );
        }
    }

    if let Ok(appdata) = std::env::var("APPDATA") {
        for command_name in &command_names {
            candidates.push(PathBuf::from(&appdata).join("npm").join(command_name));
        }
    }

    for command_name in &command_names {
        candidates.push(PathBuf::from("/opt/homebrew/bin").join(command_name));
        candidates.push(PathBuf::from("/usr/local/bin").join(command_name));
        candidates.push(PathBuf::from("/usr/bin").join(command_name));
    }

    candidates.into_iter().find(|path| path.exists())
}

fn command_variants(command: &str) -> Vec<String> {
    #[cfg(target_os = "windows")]
    {
        vec![
            format!("{}.cmd", command),
            format!("{}.exe", command),
            command.to_string(),
        ]
    }

    #[cfg(not(target_os = "windows"))]
    {
        vec![command.to_string()]
    }
}

fn run_command(path: PathBuf, args: &[&str]) -> Result<String, String> {
    let output = hidden_command(path)
        .args(args)
        .env("PATH", extended_path())
        .output()
        .map_err(|error| format!("No se pudo ejecutar comando: {}", error))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if output.status.success() {
        Ok(if stdout.is_empty() { stderr } else { stdout })
    } else {
        Err(if stderr.is_empty() { stdout } else { stderr })
    }
}

fn parse_codex_auth_accounts(output: &str) -> Vec<CodexAuthAccount> {
    output
        .lines()
        .filter_map(|line| {
            let tokens: Vec<&str> = line.split_whitespace().collect();
            let email_index = tokens
                .iter()
                .position(|part| part.contains('@') && part.contains('.'))?;
            let email = tokens[email_index].to_string();
            let plan = tokens.get(email_index + 1).map(|value| value.to_string());
            let five_hour_usage = read_usage_column(&tokens, email_index + 2);
            let weekly_index = email_index + 2 + usage_column_width(&tokens, email_index + 2);
            let weekly_usage = read_usage_column(&tokens, weekly_index);
            let last_index = weekly_index + usage_column_width(&tokens, weekly_index);
            let last_activity = if last_index < tokens.len() {
                Some(tokens[last_index..].join(" "))
            } else {
                None
            };

            Some(CodexAuthAccount {
                email,
                active: line.trim_start().starts_with('*'),
                plan,
                five_hour_usage,
                weekly_usage,
                last_activity,
                raw: line.trim().to_string(),
            })
        })
        .collect()
}

fn read_usage_column(tokens: &[&str], index: usize) -> Option<String> {
    tokens.get(index).map(|_| {
        let width = usage_column_width(tokens, index);
        tokens[index..index + width].join(" ")
    })
}

fn usage_column_width(tokens: &[&str], index: usize) -> usize {
    if index >= tokens.len() {
        return 0;
    }

    if tokens
        .get(index + 1)
        .map(|value| value.starts_with('('))
        .unwrap_or(false)
    {
        let mut width = 2;
        while index + width < tokens.len() && !tokens[index + width - 1].ends_with(')') && width < 6
        {
            width += 1;
        }
        width
    } else {
        1
    }
}

fn normalize_version(version: &str) -> String {
    version
        .split_whitespace()
        .last()
        .unwrap_or(version)
        .trim_start_matches('v')
        .to_string()
}

fn extended_path() -> String {
    let mut parts: Vec<PathBuf> = Vec::new();
    if let Ok(path) = std::env::var("PATH") {
        parts.extend(std::env::split_paths(&path));
    }

    if let Ok(home) = std::env::var("HOME") {
        parts.push(PathBuf::from(&home).join(".npm-global/bin"));
        parts.push(PathBuf::from(&home).join(".local/bin"));
    }

    if let Ok(appdata) = std::env::var("APPDATA") {
        parts.push(PathBuf::from(appdata).join("npm"));
    }

    parts.push(PathBuf::from("/opt/homebrew/bin"));
    parts.push(PathBuf::from("/usr/local/bin"));
    parts.push(PathBuf::from("/usr/bin"));

    std::env::join_paths(parts)
        .map(|path| path.to_string_lossy().to_string())
        .unwrap_or_default()
}

fn detect_codex_target() -> CodexTarget {
    #[cfg(target_os = "macos")]
    {
        if process_output_contains("ps", &["aux"], "/Applications/Codex.app") {
            return CodexTarget::Desktop;
        }

        let ps = hidden_command("ps").args(["aux"]).output();
        if let Ok(output) = ps {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let cli_running = stdout.lines().any(|line| {
                let lower = line.to_lowercase();
                lower.contains(" codex ")
                    && !lower.contains("/applications/codex.app")
                    && !lower.contains("gpt-router")
                    && !lower.contains("codex-auth")
            });

            if cli_running {
                return CodexTarget::Cli;
            }
        }
    }

    CodexTarget::None
}

fn restart_codex_target(target: CodexTarget) {
    match target {
        CodexTarget::Desktop | CodexTarget::None => restart_codex_desktop(),
        CodexTarget::Cli => open_codex_cli(),
    }
}

fn process_output_contains(command: &str, args: &[&str], needle: &str) -> bool {
    hidden_command(command)
        .args(args)
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).contains(needle))
        .unwrap_or(false)
}

fn restart_codex_desktop() {
    #[cfg(target_os = "macos")]
    {
        let _ = hidden_command("osascript")
            .args(["-e", "tell application \"Codex\" to quit"])
            .status();
        std::thread::sleep(std::time::Duration::from_millis(800));
        let _ = hidden_command("open").args(["-a", "Codex"]).spawn();
    }
}

fn open_codex_cli() {
    #[cfg(target_os = "macos")]
    {
        let command = "codex";
        let script = format!(
            "tell application \"Terminal\"\ndo script {}\nactivate\nend tell",
            apple_script_string(command)
        );
        let _ = hidden_command("osascript").args(["-e", &script]).status();
    }
}

#[cfg(target_os = "macos")]
fn disable_system_proxy() -> Result<(), String> {
    let services = ["Wi-Fi", "Ethernet", "USB 10/100/1000 LAN"];
    for service in services {
        let _ = hidden_command("networksetup")
            .args(["-setwebproxystate", service, "off"])
            .output();
        let _ = hidden_command("networksetup")
            .args(["-setsecurewebproxystate", service, "off"])
            .output();
    }

    Ok(())
}
