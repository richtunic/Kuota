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
                ..
            } = event
            {
                toggle_popover(&tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}

fn toggle_popover(app: &tauri::AppHandle) {
    let Some(window) = app.get_webview_window("popover") else {
        return;
    };

    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
        return;
    }

    position_near_tray(&window);
    let _ = window.show();
    let _ = window.set_focus();
}

fn position_near_tray(window: &tauri::WebviewWindow) {
    let Ok(Some(monitor)) = window.primary_monitor() else {
        return;
    };

    let screen_size = monitor.size();
    let win_size = window
        .outer_size()
        .unwrap_or_else(|_| tauri::PhysicalSize::new(360, 520));

    #[cfg(target_os = "macos")]
    let position =
        tauri::PhysicalPosition::new(screen_size.width as i32 - win_size.width as i32 - 10, 28);

    #[cfg(not(target_os = "macos"))]
    let position = tauri::PhysicalPosition::new(
        screen_size.width as i32 - win_size.width as i32 - 10,
        screen_size.height as i32 - win_size.height as i32 - 48,
    );

    let _ = window.set_position(position);
}
