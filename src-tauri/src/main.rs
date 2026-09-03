#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(target_os = "macos")]
mod mac_codex_login;
mod tray;
#[cfg(target_os = "windows")]
mod windows_codex_login;

use serde::Serialize;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant, SystemTime};
use tauri::Emitter;

const CODEX_AUTH_PACKAGE: &str = "@loongphy/codex-auth";
#[cfg(target_os = "windows")]
const CODEX_CLI_PACKAGE: &str = "@openai/codex";
#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

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

            start_codex_auth_file_watcher(app.handle().clone());
            start_codex_usage_watcher(app.handle().clone());
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
            cancel_codex_auth_login,
            codex_auth_open_authorization_url,
            switch_codex_account,
            configure_codex_auto_switch,
            auto_switch_codex_account,
            open_codex,
            quit_app,
        ])
        .run(tauri::generate_context!())
        .expect("error running Codex Account Router");
}

fn start_codex_auth_file_watcher(app: tauri::AppHandle) {
    let Some(auth_file) = codex_home_dir().map(|directory| directory.join("auth.json")) else {
        return;
    };

    thread::spawn(move || {
        let mut previous = file_stamp(&auth_file);

        loop {
            thread::sleep(Duration::from_millis(500));
            let current = file_stamp(&auth_file);
            if current == previous {
                continue;
            }

            thread::sleep(Duration::from_millis(250));
            previous = file_stamp(&auth_file);
            let _ = app.emit("codex-auth-changed", ());
        }
    });
}

fn start_codex_usage_watcher(app: tauri::AppHandle) {
    let Some(sessions_dir) = codex_home_dir().map(|directory| directory.join("sessions")) else {
        return;
    };

    thread::spawn(move || {
        let mut rollout = newest_rollout_file(&sessions_dir);
        let mut previous_stamp = rollout.as_deref().and_then(file_stamp);
        let mut exhausted = rollout.as_deref().is_some_and(rollout_reports_exhaustion);
        let mut last_scan = Instant::now();

        loop {
            thread::sleep(Duration::from_millis(500));

            if last_scan.elapsed() >= Duration::from_secs(5) {
                let latest = newest_rollout_file(&sessions_dir);
                if latest != rollout {
                    rollout = latest;
                    previous_stamp = None;
                    exhausted = false;
                }
                last_scan = Instant::now();
            }

            let Some(path) = rollout.as_deref() else {
                continue;
            };
            let current_stamp = file_stamp(path);
            if current_stamp == previous_stamp {
                continue;
            }
            previous_stamp = current_stamp;

            let is_exhausted = rollout_reports_exhaustion(path);
            if is_exhausted && !exhausted {
                let _ = app.emit("codex-usage-exhausted", ());
            }
            exhausted = is_exhausted;
        }
    });
}

fn codex_home_dir() -> Option<PathBuf> {
    std::env::var_os("CODEX_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .or_else(|| std::env::var_os("USERPROFILE"))
                .map(|home| PathBuf::from(home).join(".codex"))
        })
}

fn file_stamp(path: &std::path::Path) -> Option<(SystemTime, u64)> {
    let metadata = std::fs::metadata(path).ok()?;
    Some((metadata.modified().ok()?, metadata.len()))
}

fn newest_rollout_file(root: &std::path::Path) -> Option<PathBuf> {
    let mut newest: Option<(SystemTime, PathBuf)> = None;
    let mut pending = vec![root.to_path_buf()];

    while let Some(directory) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_dir() {
                pending.push(entry.path());
                continue;
            }
            if !file_type.is_file()
                || entry.path().extension().and_then(|value| value.to_str()) != Some("jsonl")
            {
                continue;
            }
            let Ok(modified) = entry.metadata().and_then(|metadata| metadata.modified()) else {
                continue;
            };
            if newest
                .as_ref()
                .is_none_or(|(current, _)| modified > *current)
            {
                newest = Some((modified, entry.path()));
            }
        }
    }

    newest.map(|(_, path)| path)
}

fn rollout_reports_exhaustion(path: &std::path::Path) -> bool {
    const TAIL_BYTES: u64 = 128 * 1024;

    let Ok(mut file) = File::open(path) else {
        return false;
    };
    let Ok(length) = file.metadata().map(|metadata| metadata.len()) else {
        return false;
    };
    let start = length.saturating_sub(TAIL_BYTES);
    if file.seek(SeekFrom::Start(start)).is_err() {
        return false;
    }

    let mut tail = String::new();
    if file.read_to_string(&mut tail).is_err() {
        return false;
    }
    if start > 0 {
        let Some(first_newline) = tail.find('\n') else {
            return false;
        };
        tail = tail[first_newline + 1..].to_string();
    }

    for line in tail.lines().rev() {
        let Ok(event) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        let Some(rate_limits) = event
            .get("payload")
            .and_then(|payload| payload.get("rate_limits"))
        else {
            continue;
        };
        if rate_limits.is_null() {
            continue;
        }

        return ["primary", "secondary"].iter().any(|window| {
            rate_limits
                .get(window)
                .and_then(|value| value.get("used_percent"))
                .and_then(|value| value.as_f64())
                .is_some_and(|used| used >= 100.0)
        });
    }

    false
}

#[tauri::command]
async fn ensure_codex_auth() -> Result<CodexAuthStatus, String> {
    let latest = npm_latest_codex_auth_version().ok();

    #[cfg(target_os = "windows")]
    ensure_windows_codex_runtime()?;

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
    let target = detect_codex_target();
    run_codex_auth_command(&["switch", selector.as_str()])?;
    restart_codex_target(target);
    Ok(read_codex_auth_status(npm_latest_codex_auth_version().ok()))
}

#[tauri::command]
async fn configure_codex_auto_switch(enabled: bool) -> Result<CodexAuthStatus, String> {
    let status = run_codex_auth_command(&["status"]).unwrap_or_default();
    if enabled || status.lines().any(|line| line.trim() == "auto-switch: ON") {
        run_codex_auth_command(&["config", "auto", "disable"])?;
    }

    Ok(read_codex_auth_status(npm_latest_codex_auth_version().ok()))
}

#[tauri::command]
async fn auto_switch_codex_account() -> Result<Option<CodexAuthStatus>, String> {
    let status = read_codex_auth_status(None);
    let Some(active) = status.accounts.iter().find(|account| account.active) else {
        return Ok(None);
    };
    if !has_exhausted_applicable_quota(active) {
        return Ok(None);
    }

    let Some(candidate) = status
        .accounts
        .iter()
        .filter(|account| !account.active)
        .filter_map(|account| available_quota_score(account).map(|score| (score, account)))
        .max_by_key(|(score, _)| *score)
        .map(|(_, account)| account)
    else {
        return Ok(None);
    };

    let target = detect_codex_target();
    run_codex_auth_command(&["switch", candidate.email.as_str()])?;
    if matches!(target, CodexTarget::Desktop) {
        restart_codex_target(CodexTarget::Desktop);
    }

    Ok(Some(read_codex_auth_status(None)))
}

fn usage_percent(value: Option<&str>) -> Option<u8> {
    let value = value?;
    let percent_index = value.find('%')?;
    let digits = value[..percent_index]
        .split_whitespace()
        .last()?
        .parse::<u8>()
        .ok()?;
    (digits <= 100).then_some(digits)
}

fn has_exhausted_applicable_quota(account: &CodexAuthAccount) -> bool {
    usage_percent(account.weekly_usage.as_deref()) == Some(0)
        || usage_percent(account.five_hour_usage.as_deref()) == Some(0)
}

fn available_quota_score(account: &CodexAuthAccount) -> Option<u8> {
    let weekly = usage_percent(account.weekly_usage.as_deref())?;
    if weekly == 0 {
        return None;
    }

    match account.five_hour_usage.as_deref() {
        Some(value) => {
            let five_hours = usage_percent(Some(value))?;
            (five_hours > 0).then_some(weekly.min(five_hours))
        }
        None => Some(weekly),
    }
}

#[tauri::command]
async fn codex_auth_login(app: tauri::AppHandle, device_auth: bool) -> Result<(), String> {
    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    let _ = &app;

    #[cfg(target_os = "windows")]
    ensure_windows_codex_runtime()?;

    if locate_codex_auth().is_none() {
        install_or_update_codex_auth()?;
    }
    let path = locate_codex_auth().ok_or_else(|| "codex-auth no esta instalado".to_string())?;

    #[cfg(target_os = "macos")]
    {
        let _ = path;
        mac_codex_login::MacCodexLoginService::start(app, device_auth)?;
    }

    #[cfg(target_os = "windows")]
    {
        let _ = path;
        windows_codex_login::WindowsCodexLoginService::start(app, device_auth)?;
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
async fn cancel_codex_auth_login(app: tauri::AppHandle) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        return mac_codex_login::MacCodexLoginService::cancel(app);
    }

    #[cfg(target_os = "windows")]
    {
        return windows_codex_login::WindowsCodexLoginService::cancel(app);
    }

    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    {
        let _ = app;
        Ok(())
    }
}

#[tauri::command]
async fn codex_auth_open_authorization_url(url: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        return mac_codex_login::MacCodexLoginService::open_authorization_url(url);
    }

    #[cfg(target_os = "windows")]
    {
        return windows_codex_login::WindowsCodexLoginService::open_authorization_url(url);
    }

    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    {
        let _ = url;
        Ok(())
    }
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

fn apple_script_string(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

#[cfg(target_os = "windows")]
pub(crate) fn hidden_command(program: impl AsRef<std::ffi::OsStr>) -> Command {
    let mut command = Command::new(program);
    use std::os::windows::process::CommandExt;
    command.creation_flags(CREATE_NO_WINDOW);
    command
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn hidden_command(program: impl AsRef<std::ffi::OsStr>) -> Command {
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

    let version = run_codex_auth_command(&["--version"]).ok();
    let status = run_codex_auth_command(&["status"]).ok();
    let accounts = run_codex_auth_command(&["list"])
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
    let output = npm_output(&["install", "-g", &format!("{}@latest", CODEX_AUTH_PACKAGE)])
        .map_err(|error| format!("No se pudo instalar {}: {}", CODEX_AUTH_PACKAGE, error))?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Err(if stderr.is_empty() { stdout } else { stderr })
}

#[cfg(target_os = "windows")]
fn ensure_windows_codex_runtime() -> Result<(), String> {
    if locate_command("npm").is_none() {
        return Err("npm no esta instalado o no esta en PATH".to_string());
    }

    if locate_codex_cli().is_none() {
        install_or_update_codex_cli()?;
    }
    if windows_codex_native_bin_dir().is_none() {
        return Err("Codex CLI se instalo, pero no se encontro el ejecutable nativo.".to_string());
    }

    if locate_codex_auth().is_none() {
        install_or_update_codex_auth()?;
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn install_or_update_codex_cli() -> Result<(), String> {
    let output = npm_output(&["install", "-g", &format!("{}@latest", CODEX_CLI_PACKAGE)])
        .map_err(|error| format!("No se pudo instalar {}: {}", CODEX_CLI_PACKAGE, error))?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Err(if stderr.is_empty() { stdout } else { stderr })
}

fn npm_latest_codex_auth_version() -> Result<String, String> {
    let output = npm_output(&["view", CODEX_AUTH_PACKAGE, "version"])?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if output.status.success() {
        Ok(if stdout.is_empty() { stderr } else { stdout })
    } else {
        Err(if stderr.is_empty() { stdout } else { stderr })
    }
}

fn installed_codex_auth_version() -> Option<String> {
    run_codex_auth_command(&["--version"]).ok()
}

pub(crate) fn locate_codex_auth() -> Option<PathBuf> {
    locate_command("codex-auth")
}

#[cfg(target_os = "windows")]
pub(crate) struct CodexAuthLauncher {
    pub(crate) program: PathBuf,
    pub(crate) args: Vec<PathBuf>,
}

#[cfg(target_os = "windows")]
pub(crate) fn resolve_codex_auth_launcher() -> Result<CodexAuthLauncher, String> {
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

#[cfg(target_os = "windows")]
fn locate_codex_auth_script(command_path: &std::path::Path) -> Option<PathBuf> {
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

    let mut npm_root_command = npm_command(locate_command("npm")?).ok()?;
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

#[cfg(target_os = "windows")]
fn read_codex_auth_bin(package_root: &std::path::Path) -> Option<PathBuf> {
    let package_json = package_root.join("package.json");
    let content = std::fs::read_to_string(package_json).ok()?;
    let json = serde_json::from_str::<serde_json::Value>(&content).ok()?;
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

#[cfg(target_os = "windows")]
fn locate_codex_cli() -> Option<PathBuf> {
    locate_command("codex")
}

#[cfg(target_os = "windows")]
fn windows_codex_native_bin_dir() -> Option<PathBuf> {
    let (platform_package, target_triple) = windows_codex_platform_package()?;
    let mut roots = Vec::new();

    if let Some(path) = locate_codex_cli() {
        if let Some(prefix) = path.parent() {
            roots.push(prefix.to_path_buf());
        }
    }

    if let Ok(appdata) = std::env::var("APPDATA") {
        roots.push(PathBuf::from(appdata).join("npm"));
    }

    if let Ok(program_files) = std::env::var("ProgramFiles") {
        roots.push(PathBuf::from(program_files).join("nodejs"));
    }

    roots
        .into_iter()
        .map(|root| {
            root.join("node_modules")
                .join("@openai")
                .join("codex")
                .join("node_modules")
                .join("@openai")
                .join(platform_package)
                .join("vendor")
                .join(target_triple)
                .join("bin")
        })
        .find(|bin_dir| bin_dir.join("codex.exe").exists())
}

#[cfg(target_os = "windows")]
fn windows_codex_platform_package() -> Option<(&'static str, &'static str)> {
    match std::env::consts::ARCH {
        "x86_64" => Some(("codex-win32-x64", "x86_64-pc-windows-msvc")),
        "aarch64" => Some(("codex-win32-arm64", "aarch64-pc-windows-msvc")),
        _ => None,
    }
}

pub(crate) fn locate_command(command: &str) -> Option<PathBuf> {
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
            format!("{}.exe", command),
            format!("{}.cmd", command),
            command.to_string(),
        ]
    }

    #[cfg(not(target_os = "windows"))]
    {
        vec![command.to_string()]
    }
}

fn run_command(path: PathBuf, args: &[&str]) -> Result<String, String> {
    let mut command = hidden_command(path);
    command.args(args);
    run_prepared_command(command)
}

fn run_codex_auth_command(args: &[&str]) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        let launcher = resolve_codex_auth_launcher()?;
        let mut command = hidden_command(launcher.program);
        command.args(launcher.args);
        command.args(args);
        return run_prepared_command(command);
    }

    #[cfg(not(target_os = "windows"))]
    {
        let path = locate_codex_auth().ok_or_else(|| "codex-auth no esta instalado".to_string())?;
        run_command(path, args)
    }
}

fn run_prepared_command(mut command: Command) -> Result<String, String> {
    apply_extended_path(&mut command);
    let output = command
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

fn npm_output(args: &[&str]) -> Result<std::process::Output, String> {
    let npm = locate_command("npm")
        .ok_or_else(|| "npm no esta instalado o no esta en PATH".to_string())?;
    let mut command = npm_command(npm)?;
    command.args(args);
    apply_extended_path(&mut command);
    command
        .output()
        .map_err(|error| format!("No se pudo ejecutar npm: {}", error))
}

fn npm_command(npm: PathBuf) -> Result<Command, String> {
    #[cfg(target_os = "windows")]
    {
        if let Some((node, script)) = resolve_npm_cli(&npm) {
            let mut command = hidden_command(node);
            command.arg(script);
            return Ok(command);
        }
    }

    Ok(hidden_command(npm))
}

#[cfg(target_os = "windows")]
fn resolve_npm_cli(npm: &std::path::Path) -> Option<(PathBuf, PathBuf)> {
    let node = locate_node_exe()?;
    let mut candidates = Vec::new();

    if let Some(prefix) = npm.parent() {
        candidates.push(
            prefix
                .join("node_modules")
                .join("npm")
                .join("bin")
                .join("npm-cli.js"),
        );
    }

    if let Ok(program_files) = std::env::var("ProgramFiles") {
        candidates.push(
            PathBuf::from(program_files)
                .join("nodejs")
                .join("node_modules")
                .join("npm")
                .join("bin")
                .join("npm-cli.js"),
        );
    }

    candidates
        .into_iter()
        .find(|script| script.exists())
        .map(|script| (node, script))
}

#[cfg(target_os = "windows")]
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

fn parse_codex_auth_accounts(output: &str) -> Vec<CodexAuthAccount> {
    let has_five_hour_column = output
        .lines()
        .any(|line| line.to_ascii_uppercase().contains("5H USAGE"));

    output
        .lines()
        .filter_map(|line| {
            let tokens: Vec<&str> = line.split_whitespace().collect();
            let email_index = tokens
                .iter()
                .position(|part| part.contains('@') && part.contains('.'))?;
            let email = tokens[email_index].to_string();
            let plan = tokens.get(email_index + 1).map(|value| value.to_string());
            let usage_index = email_index + 2;
            let five_hour_usage = has_five_hour_column
                .then(|| read_usage_column(&tokens, usage_index))
                .flatten();
            let weekly_index = if has_five_hour_column {
                usage_index + usage_column_width(&tokens, usage_index)
            } else {
                usage_index
            };
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

#[cfg(test)]
mod tests {
    use super::{
        available_quota_score, file_stamp, has_exhausted_applicable_quota,
        macos_codex_desktop_running, parse_codex_auth_accounts, rollout_reports_exhaustion,
        CodexAuthAccount,
    };

    fn account(five_hours: Option<&str>, weekly: Option<&str>) -> CodexAuthAccount {
        CodexAuthAccount {
            email: "test@example.com".to_string(),
            active: false,
            plan: None,
            five_hour_usage: five_hours.map(str::to_string),
            weekly_usage: weekly.map(str::to_string),
            last_activity: None,
            raw: String::new(),
        }
    }

    #[test]
    fn parses_current_output_with_five_hour_column() {
        let output = "ACCOUNT PLAN 5H USAGE WEEKLY USAGE LAST ACTIVITY\n\
                      * 01 plus@example.com Plus 0% (2h 4m) 63% (3d) Now\n\
                        02 free@example.com Free N/A 81% (5d) 2h ago";

        let accounts = parse_codex_auth_accounts(output);

        assert_eq!(accounts.len(), 2);
        assert_eq!(accounts[0].five_hour_usage.as_deref(), Some("0% (2h 4m)"));
        assert_eq!(accounts[0].weekly_usage.as_deref(), Some("63% (3d)"));
        assert_eq!(accounts[1].five_hour_usage.as_deref(), Some("N/A"));
        assert_eq!(accounts[1].weekly_usage.as_deref(), Some("81% (5d)"));
    }

    #[test]
    fn parses_output_without_five_hour_column() {
        let output = "ACCOUNT PLAN WEEKLY USAGE LAST ACTIVITY\n\
                      * 01 free@example.com Free 81% (5d) Now";

        let accounts = parse_codex_auth_accounts(output);

        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].five_hour_usage, None);
        assert_eq!(accounts[0].weekly_usage.as_deref(), Some("81% (5d)"));
        assert_eq!(accounts[0].last_activity.as_deref(), Some("Now"));
    }

    #[test]
    fn detects_local_auth_file_changes() {
        let path = std::env::temp_dir().join(format!("kuota-auth-watch-{}", std::process::id()));
        std::fs::write(&path, b"first").expect("write initial auth fixture");
        let before = file_stamp(&path);

        std::fs::write(&path, b"second-value").expect("update auth fixture");
        let after = file_stamp(&path);
        let _ = std::fs::remove_file(path);

        assert_ne!(before, after);
    }

    #[test]
    fn detects_codex_desktop_before_and_after_chatgpt_rename() {
        assert!(macos_codex_desktop_running(
            "/Applications/ChatGPT.app/Contents/MacOS/ChatGPT"
        ));
        assert!(macos_codex_desktop_running(
            "/Applications/Codex.app/Contents/MacOS/Codex"
        ));
        assert!(!macos_codex_desktop_running(
            "/opt/homebrew/bin/codex --resume"
        ));
    }

    #[test]
    fn accepts_only_candidates_with_known_available_quotas() {
        assert_eq!(
            available_quota_score(&account(Some("40%"), Some("80%"))),
            Some(40)
        );
        assert_eq!(available_quota_score(&account(None, Some("80%"))), Some(80));
        assert_eq!(
            available_quota_score(&account(Some("0%"), Some("80%"))),
            None
        );
        assert_eq!(
            available_quota_score(&account(Some("401"), Some("80%"))),
            None
        );
        assert_eq!(
            available_quota_score(&account(Some("40%"), Some("N/A"))),
            None
        );
    }

    #[test]
    fn detects_exhausted_active_quota() {
        assert!(has_exhausted_applicable_quota(&account(
            Some("0%"),
            Some("80%")
        )));
        assert!(has_exhausted_applicable_quota(&account(
            Some("40%"),
            Some("0%")
        )));
        assert!(!has_exhausted_applicable_quota(&account(
            Some("401"),
            Some("80%")
        )));
    }

    #[test]
    fn detects_exhaustion_in_latest_rollout_rate_limits() {
        let path = std::env::temp_dir().join(format!("kuota-rollout-{}.jsonl", std::process::id()));
        std::fs::write(
            &path,
            concat!(
                "{\"payload\":{\"rate_limits\":{\"primary\":{\"used_percent\":99},\"secondary\":{\"used_percent\":20}}}}\n",
                "{\"payload\":{\"rate_limits\":{\"primary\":{\"used_percent\":100},\"secondary\":{\"used_percent\":20}}}}\n"
            ),
        )
        .expect("write rollout fixture");

        assert!(rollout_reports_exhaustion(&path));
        let _ = std::fs::remove_file(path);
    }
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

pub(crate) fn extended_path() -> String {
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

    #[cfg(target_os = "windows")]
    if let Some(codex_bin_dir) = windows_codex_native_bin_dir() {
        parts.push(codex_bin_dir);
    }

    parts.push(PathBuf::from("/opt/homebrew/bin"));
    parts.push(PathBuf::from("/usr/local/bin"));
    parts.push(PathBuf::from("/usr/bin"));

    std::env::join_paths(parts)
        .map(|path| path.to_string_lossy().to_string())
        .unwrap_or_default()
}

pub(crate) fn apply_extended_path(command: &mut Command) {
    let path = extended_path();
    command.env("PATH", &path);
    #[cfg(target_os = "windows")]
    command.env("Path", &path);
}

fn detect_codex_target() -> CodexTarget {
    #[cfg(target_os = "macos")]
    {
        let ps = hidden_command("ps").args(["aux"]).output();
        if let Ok(output) = ps {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if macos_codex_desktop_running(&stdout) {
                return CodexTarget::Desktop;
            }

            let cli_running = stdout.lines().any(|line| {
                let lower = line.to_lowercase();
                lower.contains(" codex ")
                    && !lower.contains("/applications/codex.app")
                    && !lower.contains("/applications/chatgpt.app")
                    && !lower.contains("gpt-router")
                    && !lower.contains("codex-auth")
            });

            if cli_running {
                return CodexTarget::Cli;
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        if windows_process_running("ChatGPT.exe") || windows_process_running("Codex.exe") {
            return CodexTarget::Desktop;
        }
        if windows_process_running("codex.exe") {
            return CodexTarget::Cli;
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

fn macos_codex_desktop_running(processes: &str) -> bool {
    processes.contains("/ChatGPT.app/Contents/MacOS/")
        || processes.contains("/Codex.app/Contents/MacOS/")
}

fn restart_codex_desktop() {
    #[cfg(target_os = "macos")]
    {
        let _ = hidden_command("osascript")
            .args(["-e", "tell application id \"com.openai.codex\" to quit"])
            .status();
        std::thread::sleep(std::time::Duration::from_millis(800));
        let opened = hidden_command("open")
            .args(["-b", "com.openai.codex"])
            .status()
            .map(|status| status.success())
            .unwrap_or(false);
        if !opened {
            let opened_chatgpt = hidden_command("open")
                .args(["-a", "ChatGPT"])
                .status()
                .map(|status| status.success())
                .unwrap_or(false);
            if !opened_chatgpt {
                let _ = hidden_command("open").args(["-a", "Codex"]).spawn();
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let running_image = ["ChatGPT.exe", "Codex.exe"]
            .into_iter()
            .find(|image_name| windows_process_running(image_name));
        let running_executable = running_image.and_then(running_windows_codex_desktop_path);
        if let Some(image_name) = running_image {
            let _ = hidden_command("taskkill")
                .args(["/IM", image_name, "/F"])
                .status();
            wait_for_windows_process_exit(image_name, 2400);
        }
        std::thread::sleep(std::time::Duration::from_millis(600));

        if running_executable
            .and_then(|path| open_windows_executable(path).ok())
            .is_none()
            && open_windows_codex_desktop().is_err()
        {
            let _ = open_windows_codex_cli();
        }
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

    #[cfg(target_os = "windows")]
    {
        let _ = open_windows_codex_cli();
    }
}

#[cfg(target_os = "windows")]
fn windows_process_running(image_name: &str) -> bool {
    hidden_command("tasklist")
        .args(["/FI", &format!("IMAGENAME eq {}", image_name), "/NH"])
        .output()
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .to_lowercase()
                .contains(&image_name.to_lowercase())
        })
        .unwrap_or(false)
}

#[cfg(target_os = "windows")]
fn wait_for_windows_process_exit(image_name: &str, timeout_ms: u64) {
    let started = std::time::Instant::now();
    while started.elapsed() < std::time::Duration::from_millis(timeout_ms) {
        if !windows_process_running(image_name) {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(150));
    }
}

#[cfg(target_os = "windows")]
fn wait_for_windows_process_start(image_name: &str, timeout_ms: u64) -> bool {
    let started = std::time::Instant::now();
    while started.elapsed() < std::time::Duration::from_millis(timeout_ms) {
        if windows_process_running(image_name) {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(150));
    }
    false
}

#[cfg(target_os = "windows")]
fn running_windows_codex_desktop_path(image_name: &str) -> Option<PathBuf> {
    let query = format!(
        "$p = Get-CimInstance Win32_Process -Filter \"Name = '{}'\" | Select-Object -First 1 -ExpandProperty ExecutablePath; if ($p) {{ Write-Output $p }}",
        image_name
    );
    let output = hidden_command("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", &query])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path.is_empty() {
        return None;
    }

    let path = PathBuf::from(path);
    path.exists().then_some(path)
}

#[cfg(target_os = "windows")]
fn open_windows_executable(executable: PathBuf) -> Result<(), String> {
    let image_name = executable
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Codex.exe");
    let parent = executable.parent().map(|path| path.to_path_buf());
    let executable_arg = executable.to_string_lossy().to_string();
    let working_dir_arg = parent
        .as_deref()
        .unwrap_or_else(|| std::path::Path::new("."))
        .to_string_lossy()
        .to_string();

    let mut direct = hidden_command(&executable);
    if let Some(parent) = &parent {
        direct.current_dir(parent);
    }
    let direct_result = direct.spawn();
    if direct_result.is_ok() && wait_for_windows_process_start(image_name, 3000) {
        return Ok(());
    }

    let mut powershell = hidden_command("powershell.exe");
    powershell.args([
        "-NoProfile",
        "-NonInteractive",
        "-Command",
        "Start-Process -FilePath $args[0] -WorkingDirectory $args[1]",
        &executable_arg,
        &working_dir_arg,
    ]);
    let powershell_result = powershell.status();
    if powershell_result
        .map(|status| status.success())
        .unwrap_or(false)
        && wait_for_windows_process_start(image_name, 4000)
    {
        return Ok(());
    }

    let explorer_result = hidden_command("explorer.exe").arg(&executable_arg).status();
    if explorer_result
        .map(|status| status.success())
        .unwrap_or(false)
        && wait_for_windows_process_start(image_name, 5000)
    {
        return Ok(());
    }

    Err(format!(
        "No se pudo abrir Codex/ChatGPT Desktop desde {}",
        executable.to_string_lossy()
    ))
}

#[cfg(target_os = "windows")]
fn open_windows_codex_desktop() -> Result<(), String> {
    let executable = locate_windows_codex_desktop()
        .ok_or_else(|| "No se encontro Codex Desktop para reiniciarlo.".to_string())?;
    open_windows_executable(executable)
}

#[cfg(target_os = "windows")]
fn locate_windows_codex_desktop() -> Option<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        let root = PathBuf::from(local_app_data);
        candidates.push(root.join("Programs").join("ChatGPT").join("ChatGPT.exe"));
        candidates.push(root.join("ChatGPT").join("ChatGPT.exe"));
        candidates.push(root.join("OpenAI").join("ChatGPT").join("ChatGPT.exe"));
        candidates.push(root.join("Programs").join("Codex").join("Codex.exe"));
        candidates.push(root.join("Programs").join("codex").join("Codex.exe"));
        candidates.push(root.join("Codex").join("Codex.exe"));
        candidates.push(root.join("OpenAI").join("Codex").join("Codex.exe"));
    }

    if let Ok(program_files) = std::env::var("ProgramFiles") {
        let root = PathBuf::from(program_files);
        candidates.push(root.join("ChatGPT").join("ChatGPT.exe"));
        candidates.push(root.join("OpenAI").join("ChatGPT").join("ChatGPT.exe"));
        candidates.push(root.join("Codex").join("Codex.exe"));
        candidates.push(root.join("OpenAI").join("Codex").join("Codex.exe"));
    }

    if let Ok(program_files_x86) = std::env::var("ProgramFiles(x86)") {
        let root = PathBuf::from(program_files_x86);
        candidates.push(root.join("ChatGPT").join("ChatGPT.exe"));
        candidates.push(root.join("OpenAI").join("ChatGPT").join("ChatGPT.exe"));
        candidates.push(root.join("Codex").join("Codex.exe"));
        candidates.push(root.join("OpenAI").join("Codex").join("Codex.exe"));
    }

    candidates.into_iter().find(|path| path.exists())
}

#[cfg(target_os = "windows")]
fn open_windows_codex_cli() -> Result<(), String> {
    let codex = locate_codex_cli().ok_or_else(|| "Codex CLI no esta instalado.".to_string())?;
    let mut command = hidden_command(codex);
    apply_extended_path(&mut command);
    command
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("No se pudo abrir Codex CLI: {}", error))
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
