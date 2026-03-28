use orc_core::pty::{PtyError, PtyEvent, PtyEventHandler, PtyManager};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};

struct TauriPtyHandler {
    app: AppHandle,
}

impl PtyEventHandler for TauriPtyHandler {
    fn on_event(&self, session_id: &str, event: PtyEvent) {
        match event {
            PtyEvent::Data(data) => {
                let text = String::from_utf8_lossy(&data).to_string();
                let _ = self.app.emit(&format!("pty-data-{session_id}"), text);
            }
            PtyEvent::Exit => {
                let _ = self.app.emit(&format!("pty-exit-{session_id}"), ());
            }
        }
    }
}

pub struct PtyState {
    manager: PtyManager,
}

impl PtyState {
    pub fn new(app: AppHandle) -> Self {
        let handler = Arc::new(TauriPtyHandler { app });
        Self {
            manager: PtyManager::new(handler),
        }
    }
}

fn map_err(e: PtyError) -> String {
    e.to_string()
}

#[tauri::command]
pub fn pty_spawn(
    state: tauri::State<'_, PtyState>,
    id: String,
    rows: u16,
    cols: u16,
) -> Result<(), String> {
    state.manager.spawn(id, rows, cols).map_err(map_err)
}

#[tauri::command]
pub fn pty_write(state: tauri::State<'_, PtyState>, id: String, data: String) -> Result<(), String> {
    state.manager.write(&id, data.as_bytes()).map_err(map_err)
}

#[tauri::command]
pub fn pty_resize(
    state: tauri::State<'_, PtyState>,
    id: String,
    rows: u16,
    cols: u16,
) -> Result<(), String> {
    state.manager.resize(&id, rows, cols).map_err(map_err)
}

#[tauri::command]
pub fn pty_kill(state: tauri::State<'_, PtyState>, id: String) -> Result<(), String> {
    state.manager.kill(&id).map_err(map_err)
}
