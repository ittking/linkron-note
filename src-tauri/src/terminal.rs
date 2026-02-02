use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize, Child};
use std::collections::HashMap;
use std::env;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use lazy_static::lazy_static;

// PTY 会话结构，保存所有必要的资源
struct PtySession {
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    master: Box<dyn MasterPty + Send>,
    child: Box<dyn Child + Send>, // 保存子进程句柄
}

struct TerminalManager {
    sessions: Mutex<HashMap<String, PtySession>>,
}

impl TerminalManager {
    fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
        }
    }

    fn create_session(
        &self,
        app_handle: AppHandle,
        session_id: String,
        shell: &str,
        cols: u16,
        rows: u16,
        _working_dir: Option<String>,
    ) -> Result<(), String> {
        let pty_system = native_pty_system();

        let pty_pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("Failed to open PTY: {}", e))?;

        let mut cmd = CommandBuilder::new(shell);
        cmd.env("TERM", "xterm-256color");

        // 暂时不设置工作目录，先让终端能正常启动
        // TODO: 后续添加工作目录支持

        let child = pty_pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| format!("Failed to spawn shell: {}", e))?;

        let reader = pty_pair.master.try_clone_reader().unwrap();
        let writer = pty_pair.master.take_writer().unwrap();

        let session = PtySession {
            writer: Arc::new(Mutex::new(Box::new(writer))),
            master: pty_pair.master,
            child, // 保存子进程句柄
        };

        {
            let mut sessions = self.sessions.lock().unwrap();
            sessions.insert(session_id.clone(), session);
        }

        // 启动输出读取线程
        let session_id_clone = session_id.clone();
        std::thread::spawn(move || {
            let mut reader = reader;
            let mut buffer = [0u8; 8192];

            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        let data = buffer[..n].to_vec();
                        let _ = app_handle.emit(
                            format!("terminal-output-{}", session_id_clone).as_str(),
                            String::from_utf8_lossy(&data).to_string(),
                        );
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(())
    }

    fn write_to_session(&self, session_id: &str, data: &str) -> Result<(), String> {
        let sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get(session_id) {
            let mut writer = session.writer.lock().unwrap();
            writer
                .write_all(data.as_bytes())
                .map_err(|e| format!("Failed to write to PTY: {}", e))?;
            writer
                .flush()
                .map_err(|e| format!("Failed to flush PTY: {}", e))?;
            Ok(())
        } else {
            Err(format!("Session {} not found", session_id))
        }
    }

    fn resize_session(&self, session_id: &str, cols: u16, rows: u16) -> Result<(), String> {
        let sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get(session_id) {
            session
                .master
                .resize(PtySize {
                    rows,
                    cols,
                    pixel_width: 0,
                    pixel_height: 0,
                })
                .map_err(|e| format!("Failed to resize PTY: {}", e))?;
            Ok(())
        } else {
            Err(format!("Session {} not found", session_id))
        }
    }

    fn close_session(&self, session_id: &str) -> Result<(), String> {
        let mut sessions = self.sessions.lock().unwrap();

        // 先移除会话，这样会获取 session 的所有权
        if let Some(mut session) = sessions.remove(session_id) {
            // 杀死子进程，这会导致 PTY 读取线程收到 EOF 并退出
            let _ = session.child.kill();
        }

        Ok(())
    }
}

// 使用 lazy_static 创建全局单例
lazy_static! {
    static ref TERMINAL_MANAGER: TerminalManager = TerminalManager::new();
}

#[tauri::command]
pub async fn create_pty_session(
    app_handle: AppHandle,
    session_id: String,
    shell: String,
    cols: u16,
    rows: u16,
    working_dir: Option<String>,
) -> Result<(), String> {
    TERMINAL_MANAGER.create_session(app_handle, session_id, &shell, cols, rows, working_dir)
}

#[tauri::command]
pub async fn write_to_pty(session_id: String, data: String) -> Result<(), String> {
    TERMINAL_MANAGER.write_to_session(&session_id, &data)
}

#[tauri::command]
pub async fn resize_pty(session_id: String, cols: u16, rows: u16) -> Result<(), String> {
    TERMINAL_MANAGER.resize_session(&session_id, cols, rows)
}

#[tauri::command]
pub async fn close_pty_session(session_id: String) -> Result<(), String> {
    TERMINAL_MANAGER.close_session(&session_id)
}

#[tauri::command]
pub async fn get_current_directory() -> Result<String, String> {
    env::current_dir()
        .map(|path| path.to_string_lossy().to_string())
        .map_err(|e| format!("Failed to get current directory: {}", e))
}