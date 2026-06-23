use crate::events;
use crate::popup::{PopupConfig, PopupMessage};
use crate::state::AppState;
use pebble_core::{now_timestamp, PebbleError};
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter, Manager, State};

/// Key for persisting popup config in secure_user_data (plain JSON).
const POPUP_CONFIG_KEY: &str = "popup_config";

pub(crate) fn apply_auto_launch_registry(enabled: bool) -> std::result::Result<(), PebbleError> {
    #[cfg(target_os = "windows")]
    {
        use windows_registry::CURRENT_USER;
        let key = CURRENT_USER
            .create("Software\\Microsoft\\Windows\\CurrentVersion\\Run")
            .map_err(|e| PebbleError::Internal(format!("registry: {e}")))?;
        if enabled {
            let exe = std::env::current_exe()
                .map_err(|e| PebbleError::Internal(format!("current_exe: {e}")))?;
            let path = exe.to_string_lossy().to_string();
            key.set_string("PebbleMsg", &format!("\"{path}\""))
                .map_err(|e| PebbleError::Internal(format!("registry set: {e}")))?;
        } else {
            key.remove_value("PebbleMsg")
                .map_err(|e| PebbleError::Internal(format!("registry remove: {e}")))?;
        }
    }
    Ok(())
}

fn load_popup_config_from_store(
    store: &pebble_store::Store,
) -> std::result::Result<Option<PopupConfig>, PebbleError> {
    match store.get_secure_user_data(POPUP_CONFIG_KEY)? {
        Some(bytes) => {
            match serde_json::from_slice::<PopupConfig>(&bytes) {
                Ok(cfg) => Ok(Some(cfg)),
                Err(_) => Ok(None), // corrupted data → fallback to default
            }
        }
        None => Ok(None),
    }
}

fn save_popup_config_to_store(
    store: &pebble_store::Store,
    config: &PopupConfig,
) -> std::result::Result<(), PebbleError> {
    let json = serde_json::to_vec(config)
        .map_err(|e| PebbleError::Internal(format!("json serialize: {e}")))?;
    store.set_secure_user_data(POPUP_CONFIG_KEY, &json)
}

#[tauri::command]
pub async fn set_ui_theme(
    state: State<'_, AppState>,
    theme: String,
) -> std::result::Result<(), PebbleError> {
    if let Ok(mut t) = state.current_theme.lock() {
        *t = theme;
    }
    Ok(())
}

#[tauri::command]
pub async fn get_popup_data(
    state: State<'_, AppState>,
    message_id: String,
) -> std::result::Result<Option<PopupMessage>, PebbleError> {
    Ok(state.popup_manager.get_pending_message(&message_id))
}

#[tauri::command]
pub async fn update_popup_config(
    state: State<'_, AppState>,
    config: PopupConfig,
) -> std::result::Result<(), PebbleError> {
    // Persist to store first, then update in-memory
    save_popup_config_to_store(&state.store, &config)?;
    apply_auto_launch_registry(config.auto_launch)?;
    state.popup_manager.update_config(config);
    Ok(())
}

#[tauri::command]
pub async fn get_popup_config(
    state: State<'_, AppState>,
) -> std::result::Result<PopupConfig, PebbleError> {
    // Load from store; fall back to in-memory defaults
    if let Some(config) = load_popup_config_from_store(&state.store)? {
        return Ok(config);
    }
    Ok(state.popup_manager.get_config())
}

#[tauri::command]
pub async fn show_test_popup(
    app: AppHandle,
    state: State<'_, AppState>,
) -> std::result::Result<(), PebbleError> {
    if !state.notifications_enabled.load(Ordering::SeqCst) {
        return Err(PebbleError::Validation(
            "Popup notifications are disabled".to_string(),
        ));
    }

    let msg = PopupMessage {
        message_id: format!(
            "test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        ),
        account_id: String::new(),
        subject: "Pebble - Test Popup".to_string(),
        sender_name: "PebbleMsg".to_string(),
        sender_address: "pebblemsg@local".to_string(),
        snippet: "This is a preview of the popup notification.".to_string(),
        body_text: "Hello from PebbleMsg!\n\nThis is a test popup.\nYou can customize font size,\nlayout and display content\nin Settings.".to_string(),
        received_at: now_timestamp(),
        is_starred: false,
        is_read: false,
        thread_id: None,
    };

    state
        .popup_manager
        .show_popup(&app, msg)
        .map_err(|e| PebbleError::Internal(e))
}

#[tauri::command]
pub async fn close_popup(
    app: AppHandle,
    message_id: String,
) -> std::result::Result<(), PebbleError> {
    let label = format!("popup-{message_id}");
    if let Some(window) = app.get_webview_window(&label) {
        let _ = window.close();
    }
    if let Some(state) = app.try_state::<AppState>() {
        if let Ok(mut active) = state.popup_manager.active_popups.lock() {
            active.remove(&label);
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn popup_open_message(
    app: AppHandle,
    message_id: String,
    account_id: String,
) -> std::result::Result<(), PebbleError> {
    crate::commands::notifications::clear_attention_indicator(&app);

    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }

    let _ = app.emit(
        events::MAIL_NOTIFICATION_OPEN,
        serde_json::json!({
            "account_id": account_id,
            "message_id": message_id,
        }),
    );

    let label = format!("popup-{message_id}");
    if let Some(window) = app.get_webview_window(&label) {
        let _ = window.close();
    }
    Ok(())
}
