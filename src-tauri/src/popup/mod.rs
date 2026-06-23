use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, PhysicalPosition, Runtime, WebviewUrl, WebviewWindowBuilder};

const POPUP_LABEL_PREFIX: &str = "popup-";
const MAX_POPUPS_HARD: usize = 5;
const HORIZONTAL_GAP: i32 = 10;
const VERTICAL_GAP: i32 = 10;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopupMessage {
    pub message_id: String,
    pub account_id: String,
    pub subject: String,
    pub sender_name: String,
    pub sender_address: String,
    pub snippet: String,
    pub body_text: String,
    pub received_at: i64,
    pub is_starred: bool,
    pub is_read: bool,
    pub thread_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopupConfig {
    pub width: u32,
    pub height: u32,
    pub position_x: Option<i32>,
    pub position_y: Option<i32>,
    pub duration_ms: u64,
    pub show_sender: bool,
    pub show_subject: bool,
    pub show_snippet: bool,
    pub show_time: bool,
    pub max_popups: usize,
    pub font_size_sender: u32,
    pub font_size_subject: u32,
    pub font_size_snippet: u32,
    pub font_size_time: u32,
    #[serde(default = "default_auto_launch")]
    pub auto_launch: bool,
    #[serde(default = "default_minimize_on_startup")]
    pub minimize_on_startup: bool,
}

fn default_auto_launch() -> bool {
    true
}

fn default_minimize_on_startup() -> bool {
    false
}

impl Default for PopupConfig {
    fn default() -> Self {
        Self {
            width: 380,
            height: 200,
            position_x: None,
            position_y: None,
            duration_ms: 8000,
            show_sender: true,
            show_subject: true,
            show_snippet: true,
            show_time: true,
            max_popups: 1,
            font_size_sender: 13,
            font_size_subject: 13,
            font_size_snippet: 12,
            font_size_time: 11,
            auto_launch: true,
            minimize_on_startup: false,
        }
    }
}

pub struct PopupManager {
    pub active_popups: Mutex<HashMap<String, u64>>,
    config: Mutex<PopupConfig>,
    pending_messages: Mutex<HashMap<String, PopupMessage>>,
    next_order: Mutex<u64>,
}

impl PopupManager {
    pub fn new() -> Self {
        Self {
            active_popups: Mutex::new(HashMap::new()),
            config: Mutex::new(PopupConfig::default()),
            pending_messages: Mutex::new(HashMap::new()),
            next_order: Mutex::new(0),
        }
    }

    pub fn get_pending_message(&self, message_id: &str) -> Option<PopupMessage> {
        self.pending_messages
            .lock()
            .ok()
            .and_then(|mut map| map.remove(message_id))
    }

    pub fn update_config(&self, config: PopupConfig) {
        if let Ok(mut c) = self.config.lock() {
            *c = config;
        }
    }

    pub fn get_config(&self) -> PopupConfig {
        self.config.lock().map(|c| c.clone()).unwrap_or_default()
    }

    fn find_oldest(&self, active: &HashMap<String, u64>) -> Option<String> {
        active
            .iter()
            .min_by_key(|(_, order)| **order)
            .map(|(label, _)| label.clone())
    }

    fn calculate_position(
        &self,
        _screen_w: i32,
        _screen_h: i32,
        popup_w: i32,
        popup_h: i32,
        base_x: i32,
        base_y: i32,
        slot: usize,
    ) -> (i32, i32) {
        let slot = slot as i32;
        let max_rows = ((base_y + popup_h + VERTICAL_GAP) / (popup_h + VERTICAL_GAP)).max(1);
        let row = slot % max_rows;
        let col = slot / max_rows;
        let x = base_x - col * (popup_w + HORIZONTAL_GAP);
        let y = base_y - row * (popup_h + VERTICAL_GAP);
        (x.max(0), y.max(0))
    }

    fn reposition_all<R: Runtime>(&self, app: &AppHandle<R>) {
        if let Ok(active) = self.active_popups.lock() {
            if active.is_empty() {
                return;
            }

            let config = self.get_config();
            let mut slots: Vec<(String, u64)> = active.iter().map(|(k, v)| (k.clone(), *v)).collect();
            slots.sort_by_key(|(_, order)| *order);
            drop(active);

            let popup_w = config.width as i32;
            let popup_h = config.height as i32;

            let (screen_w, screen_h, base_x, base_y) = {
                if let Some(window) = app.get_webview_window(&slots[0].0) {
                    if let Some(monitor) = window.current_monitor().ok().flatten() {
                        let sz = monitor.size();
                        let sw = sz.width as i32;
                        let sh = sz.height as i32;
                        let bx = config.position_x.unwrap_or(sw - popup_w - 20);
                        let by = config.position_y.unwrap_or(sh - popup_h - 60);
                        (sw, sh, bx, by)
                    } else {
                        return;
                    }
                } else {
                    return;
                }
            };

            for (i, (label, _)) in slots.iter().enumerate() {
                if let Some(window) = app.get_webview_window(label) {
                    let (x, y) = self.calculate_position(screen_w, screen_h, popup_w, popup_h, base_x, base_y, i);
                    let _ = window.set_position(tauri::Position::Physical(
                        PhysicalPosition::new(x, y),
                    ));
                }
            }
        }
    }

    pub fn show_popup<R: Runtime>(
        &self,
        app: &AppHandle<R>,
        msg: PopupMessage,
    ) -> Result<(), String> {
        let config = self.get_config();
        let max = config.max_popups.clamp(1, MAX_POPUPS_HARD);

        let label = format!("{POPUP_LABEL_PREFIX}{}", msg.message_id);

        let mut active = self
            .active_popups
            .lock()
            .map_err(|e| format!("popup lock error: {e}"))?;

        if active.contains_key(&label) {
            return Ok(());
        }

        if active.len() >= max {
            let oldest_label = self.find_oldest(&active);
            if let Some(ref old) = oldest_label {
                active.remove(old);
                drop(active);
                if let Some(window) = app.get_webview_window(old) {
                    let _ = window.close();
                }
            } else {
                return Ok(());
            }
        } else {
            drop(active);
        }

        let mut active = self.active_popups.lock().map_err(|e| format!("popup lock error: {e}"))?;
        let order = {
            let mut next = self.next_order.lock().map_err(|e| format!("order lock error: {e}"))?;
            let o = *next;
            *next += 1;
            o
        };
        active.insert(label.clone(), order);
        let slot = active.len() - 1;
        drop(active);

        let label_clone = label.clone();
        let app_handle = app.clone();
        let message_id = msg.message_id.clone();

        {
            let mut messages = self.pending_messages.lock().map_err(|e| format!("popup lock error: {e}"))?;
            messages.insert(message_id.clone(), msg);
        }

        let config_json = serde_json::to_string(&config).unwrap_or_default();
        let theme = app
            .try_state::<AppState>()
            .and_then(|s| s.current_theme.lock().ok().map(|t| t.clone()))
            .unwrap_or_else(|| "light".to_string());

        match WebviewWindowBuilder::new(app, &label, WebviewUrl::App("popup.html".into()))
            .title("PebbleMsg")
            .inner_size(config.width as f64, config.height as f64)
            .decorations(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .resizable(false)
            .visible(false)
            .initialization_script(&format!(
                "window.__PEBBLE_MSG_ID__='{msg_id}';window.__PEBBLE_CONFIG__={cfg};window.__PEBBLE_THEME__='{theme}';",
                msg_id = message_id.replace('\'', "\\'"),
                cfg = config_json,
                theme = theme.replace('\'', "\\'"),
            ))
            .build()
        {
            Ok(window) => {
                let popup_w = config.width as i32;
                let popup_h = config.height as i32;

                #[allow(clippy::cast_possible_truncation)]
                if let Some(monitor) = window.current_monitor().ok().flatten() {
                    let monitor_size = monitor.size();
                    let screen_w = monitor_size.width as i32;
                    let screen_h = monitor_size.height as i32;

                    let base_x = config.position_x.unwrap_or(screen_w - popup_w - 20);
                    let base_y = config.position_y.unwrap_or(screen_h - popup_h - 60);

                    let (x, y) = self.calculate_position(screen_w, screen_h, popup_w, popup_h, base_x, base_y, slot);
                    let _ = window.set_position(tauri::Position::Physical(
                        PhysicalPosition::new(x, y),
                    ));
                }

                let _ = window.show();

                if config.duration_ms > 0 {
                    let label_for_close = label_clone.clone();
                    let app_for_close = app_handle.clone();
                    std::thread::spawn(move || {
                        std::thread::sleep(std::time::Duration::from_millis(config.duration_ms));
                        if let Some(w) = app_for_close.get_webview_window(&label_for_close) {
                            let _ = w.close();
                        }
                    });
                }

                let app_for_cleanup = app_handle;
                let label_for_cleanup = label_clone;
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::Destroyed = event {
                        if let Some(state) = app_for_cleanup.try_state::<AppState>() {
                            if let Ok(mut active) = state.popup_manager.active_popups.lock() {
                                active.remove(&label_for_cleanup);
                            }
                            state.popup_manager.reposition_all(&app_for_cleanup);
                        }
                    }
                });
            }
            Err(e) => {
                if let Ok(mut active) = self.active_popups.lock() {
                    active.remove(&label);
                }
                return Err(format!("Failed to create popup window: {e}"));
            }
        }

        Ok(())
    }

    pub fn close_all_popups<R: Runtime>(&self, app: &AppHandle<R>) {
        if let Ok(mut active) = self.active_popups.lock() {
            let labels: Vec<String> = active.keys().cloned().collect();
            active.clear();
            drop(active);
            for label in labels {
                if let Some(window) = app.get_webview_window(&label) {
                    let _ = window.close();
                }
            }
        }
    }
}
