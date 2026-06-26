use std::process::{Child, Command};
use std::sync::{Arc, Mutex};

lazy_static::lazy_static! {
    static ref PROXY_PROCESS: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));
}

fn find_mitmdump() -> Option<String> {
    let candidates = [
        "/opt/homebrew/bin/mitmdump",
        "/usr/local/bin/mitmdump",
        "/usr/bin/mitmdump",
    ];

    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").unwrap_or_default();
        let mac_candidates = [
            format!("{}/Library/Python/3.11/bin/mitmdump", home),
            format!("{}/Library/Python/3.12/bin/mitmdump", home),
            format!("{}/.local/bin/mitmdump", home),
        ];

        for path in mac_candidates {
            if std::path::Path::new(&path).exists() {
                return Some(path);
            }
        }
    }

    for path in candidates {
        if std::path::Path::new(path).exists() {
            return Some(path.to_string());
        }
    }

    if Command::new("which")
        .arg("mitmdump")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
    {
        return Some("mitmdump".to_string());
    }

    None
}

fn find_script() -> Option<String> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;
    let candidates = [
        dir.join("gpt_interceptor.py"),
        dir.join("../Resources/gpt_interceptor.py"),
        dir.join("../gpt_interceptor.py"),
        dir.join("../../../proxy/gpt_interceptor.py"),
    ];

    candidates
        .iter()
        .find(|path| path.exists())
        .map(|path| path.to_string_lossy().to_string())
}

pub fn is_proxy_running() -> bool {
    let Ok(mut guard) = PROXY_PROCESS.lock() else {
        return false;
    };

    if let Some(child) = guard.as_mut() {
        match child.try_wait() {
            Ok(Some(_)) => {
                *guard = None;
                false
            }
            Ok(None) => true,
            Err(_) => {
                *guard = None;
                false
            }
        }
    } else {
        false
    }
}

#[tauri::command]
pub async fn start_proxy(port: u16) -> Result<String, String> {
    let mut guard = PROXY_PROCESS.lock().map_err(|error| error.to_string())?;
    if guard.is_some() {
        return Ok("already_running".to_string());
    }

    let mitmdump = find_mitmdump()
        .ok_or_else(|| "mitmdump no encontrado. Ejecuta proxy/setup.sh o instala mitmproxy.".to_string())?;
    let script = find_script().ok_or_else(|| "proxy/gpt_interceptor.py no encontrado.".to_string())?;

    let child = Command::new(&mitmdump)
        .args([
            "--listen-host",
            "127.0.0.1",
            "--listen-port",
            &port.to_string(),
            "--quiet",
            "-s",
            &script,
        ])
        .spawn()
        .map_err(|error| format!("Error iniciando proxy: {}", error))?;

    *guard = Some(child);
    Ok("started".to_string())
}

pub fn stop_proxy_process() -> Result<(), String> {
    let mut guard = PROXY_PROCESS.lock().map_err(|error| error.to_string())?;
    if let Some(mut child) = guard.take() {
        let _ = child.kill();
        let _ = child.wait();
    }

    Ok(())
}

#[tauri::command]
pub async fn stop_proxy() -> Result<(), String> {
    stop_proxy_process()
}

#[tauri::command]
pub async fn proxy_status() -> bool {
    is_proxy_running()
}
