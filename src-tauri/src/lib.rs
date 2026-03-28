use tauri::Manager;

mod agent;
mod pty;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(pty::PtyState::new(app.handle().clone()));
            app.manage(agent::AgentState::new(app.handle().clone()));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            pty::pty_spawn,
            pty::pty_write,
            pty::pty_resize,
            pty::pty_kill,
            agent::agent_spawn,
            agent::agent_send,
            agent::agent_kill,
            agent::agent_status,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run orc");
}
