use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

use super::error::PtyError;
use super::event::{PtyEvent, PtyEventHandler};

struct PtySession {
    writer: Box<dyn Write + Send>,
    master: Box<dyn MasterPty + Send>,
}

pub struct PtyManager {
    sessions: Mutex<HashMap<String, PtySession>>,
    handler: Arc<dyn PtyEventHandler>,
}

impl PtyManager {
    pub fn new(handler: Arc<dyn PtyEventHandler>) -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
            handler,
        }
    }

    pub fn spawn(&self, id: String, rows: u16, cols: u16) -> Result<(), PtyError> {
        let pty_system = native_pty_system();

        let pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| PtyError::SpawnFailed(e.to_string()))?;

        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
        let mut cmd = CommandBuilder::new(&shell);
        cmd.arg("-l");
        cmd.cwd(std::env::var("HOME").unwrap_or_else(|_| "/".to_string()));

        pair.slave
            .spawn_command(cmd)
            .map_err(|e| PtyError::SpawnFailed(e.to_string()))?;

        let writer = pair
            .master
            .take_writer()
            .map_err(|e| PtyError::SpawnFailed(e.to_string()))?;
        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| PtyError::SpawnFailed(e.to_string()))?;

        let handler = Arc::clone(&self.handler);
        let event_id = id.clone();
        std::thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) | Err(_) => {
                        handler.on_event(&event_id, PtyEvent::Exit);
                        break;
                    }
                    Ok(n) => {
                        handler.on_event(&event_id, PtyEvent::Data(buf[..n].to_vec()));
                    }
                }
            }
        });

        let session = PtySession {
            writer,
            master: pair.master,
        };

        self.sessions
            .lock()
            .map_err(|e| PtyError::SpawnFailed(e.to_string()))?
            .insert(id, session);

        Ok(())
    }

    pub fn write(&self, id: &str, data: &[u8]) -> Result<(), PtyError> {
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|e| PtyError::IoError(std::io::Error::other(e.to_string())))?;
        let session = sessions
            .get_mut(id)
            .ok_or_else(|| PtyError::SessionNotFound(id.to_string()))?;
        session.writer.write_all(data)?;
        session.writer.flush()?;
        Ok(())
    }

    pub fn resize(&self, id: &str, rows: u16, cols: u16) -> Result<(), PtyError> {
        let sessions = self
            .sessions
            .lock()
            .map_err(|e| PtyError::IoError(std::io::Error::other(e.to_string())))?;
        let session = sessions
            .get(id)
            .ok_or_else(|| PtyError::SessionNotFound(id.to_string()))?;
        session
            .master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| PtyError::IoError(std::io::Error::other(e.to_string())))?;
        Ok(())
    }

    pub fn kill(&self, id: &str) -> Result<(), PtyError> {
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|e| PtyError::IoError(std::io::Error::other(e.to_string())))?;
        sessions.remove(id);
        Ok(())
    }
}
