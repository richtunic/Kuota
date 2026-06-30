use tauri::{
    image::Image,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, Manager,
};

pub fn setup_tray(app: &mut App) -> tauri::Result<()> {
    let tray_icon = Image::from_bytes(include_bytes!("../icons/tray.png"))?;

    let _tray = TrayIconBuilder::with_id("main-tray")
        .tooltip("Codex Account Router")
        .icon(tray_icon)
        .icon_as_template(true)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                position,
                ..
            } = event
            {
                toggle_popover(&tray.app_handle(), Some((position.x, position.y)));
            }
        })
        .build(app)?;

    Ok(())
}

fn toggle_popover(app: &tauri::AppHandle, tray_position: Option<(f64, f64)>) {
    let Some(window) = app.get_webview_window("popover") else {
        return;
    };

    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
        return;
    }

    position_near_tray(&window, tray_position);
    let _ = window.show();
    let _ = window.set_focus();
}

fn position_near_tray(window: &tauri::WebviewWindow, tray_position: Option<(f64, f64)>) {
    let monitor = match tray_position
        .and_then(|(x, y)| window.monitor_from_point(x, y).ok().flatten())
        .or_else(|| window.primary_monitor().ok().flatten())
    {
        Some(monitor) => monitor,
        None => return,
    };

    let work_area = monitor.work_area();
    let win_size = window
        .outer_size()
        .unwrap_or_else(|_| tauri::PhysicalSize::new(360, 520));
    let margin = 8;

    let Some((tray_x, _)) = tray_position else {
        position_with_fallback(window, work_area, win_size, margin);
        return;
    };

    let min_x = work_area.position.x + margin;
    let max_x = work_area.position.x + work_area.size.width as i32 - win_size.width as i32 - margin;
    let x = (tray_x.round() as i32 - win_size.width as i32 / 2).clamp(min_x, max_x.max(min_x));

    #[cfg(target_os = "macos")]
    let y = work_area.position.y + margin;

    #[cfg(not(target_os = "macos"))]
    let y = work_area.position.y + work_area.size.height as i32 - win_size.height as i32 - margin;

    let _ = window.set_position(tauri::PhysicalPosition::new(x, y));
}

fn position_with_fallback(
    window: &tauri::WebviewWindow,
    work_area: &tauri::PhysicalRect<i32, u32>,
    win_size: tauri::PhysicalSize<u32>,
    margin: i32,
) {
    let x = work_area.position.x + work_area.size.width as i32 - win_size.width as i32 - margin;

    #[cfg(target_os = "macos")]
    let y = work_area.position.y + margin;

    #[cfg(not(target_os = "macos"))]
    let y = work_area.position.y + work_area.size.height as i32 - win_size.height as i32 - margin;

    let _ = window.set_position(tauri::PhysicalPosition::new(x, y));
}
