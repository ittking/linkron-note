// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod mouse;

#[cfg(any(windows, target_os = "macos"))]
mod window_manager;

use tauri::Manager;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn check_directory_exists(path: String) -> Result<bool, String> {
    std::fs::metadata(&path)
        .map(|m| m.is_dir())
        .map_err(|e| format!("Failed to check directory: {}", e))
}

#[tauri::command]
async fn create_directory(path: String) -> Result<(), String> {
    std::fs::create_dir_all(&path)
        .map_err(|e| format!("Failed to create directory: {}", e))
}

#[tauri::command]
async fn set_autostart(enable: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let path = r"Software\Microsoft\Windows\CurrentVersion\Run";
        
        if enable {
            let key = hkcu.open_subkey_with_flags(path, KEY_WRITE)
                .map_err(|e| format!("Failed to open registry key: {}", e))?;
            
            let exe_path = std::env::current_exe()
                .map_err(|e| format!("Failed to get executable path: {}", e))?;
            
            let exe_path_str = exe_path.to_str()
                .ok_or("Failed to convert path to string")?;
            
            key.set_value("iterm", &exe_path_str)
                .map_err(|e| format!("Failed to set registry value: {}", e))?;
        } else {
            let key = hkcu.open_subkey_with_flags(path, KEY_WRITE)
                .map_err(|e| format!("Failed to open registry key: {}", e))?;
            
            key.delete_value("iterm")
                .map_err(|e| format!("Failed to delete registry value: {}", e))?;
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        use std::fs;
        use std::path::PathBuf;
        
        let home = std::env::var("HOME").map_err(|e| format!("Failed to get HOME: {}", e))?;
        let launch_agents = PathBuf::from(home).join("Library/LaunchAgents");
        let plist_path = launch_agents.join("com.administrator.iterm.plist");
        
        if enable {
            fs::create_dir_all(&launch_agents)
                .map_err(|e| format!("Failed to create LaunchAgents directory: {}", e))?;
            
            let exe_path = std::env::current_exe()
                .map_err(|e| format!("Failed to get executable path: {}", e))?;
            
            let exe_path_str = exe_path.to_str()
                .ok_or("Failed to convert path to string")?;
            
            let plist_content = format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.administrator.iterm</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
</dict>
</plist>
"#,
                exe_path_str
            );
            
            fs::write(&plist_path, plist_content)
                .map_err(|e| format!("Failed to write plist file: {}", e))?;
        } else {
            if plist_path.exists() {
                fs::remove_file(&plist_path)
                    .map_err(|e| format!("Failed to remove plist file: {}", e))?;
            }
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        use std::path::PathBuf;
        
        let home = std::env::var("HOME").map_err(|e| format!("Failed to get HOME: {}", e))?;
        let autostart_dir = PathBuf::from(home).join(".config/autostart");
        let desktop_path = autostart_dir.join("iterm.desktop");
        
        if enable {
            fs::create_dir_all(&autostart_dir)
                .map_err(|e| format!("Failed to create autostart directory: {}", e))?;
            
            let exe_path = std::env::current_exe()
                .map_err(|e| format!("Failed to get executable path: {}", e))?;
            
            let exe_path_str = exe_path.to_str()
                .ok_or("Failed to convert path to string")?;
            
            let desktop_content = format!(
                r#"[Desktop Entry]
Type=Application
Name=iterm
Exec={}
Terminal=false
X-GNOME-Autostart-enabled=true
"#,
                exe_path_str
            );
            
            fs::write(&desktop_path, desktop_content)
                .map_err(|e| format!("Failed to write desktop file: {}", e))?;
        } else {
            if desktop_path.exists() {
                fs::remove_file(&desktop_path)
                    .map_err(|e| format!("Failed to remove desktop file: {}", e))?;
            }
        }
    }
    
    Ok(())
}

#[tauri::command]
async fn is_autostart_enabled() -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let path = r"Software\Microsoft\Windows\CurrentVersion\Run";
        
        match hkcu.open_subkey_with_flags(path, KEY_READ) {
            Ok(key) => {
                match key.get_value::<String, _>("iterm") {
                    Ok(_) => Ok(true),
                    Err(_) => Ok(false),
                }
            }
            Err(_) => Ok(false),
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        use std::path::PathBuf;
        
        let home = std::env::var("HOME").map_err(|e| format!("Failed to get HOME: {}", e))?;
        let plist_path = PathBuf::from(home).join("Library/LaunchAgents/com.administrator.iterm.plist");
        
        Ok(plist_path.exists())
    }
    
    #[cfg(target_os = "linux")]
    {
        use std::path::PathBuf;
        
        let home = std::env::var("HOME").map_err(|e| format!("Failed to get HOME: {}", e))?;
        let desktop_path = PathBuf::from(home).join(".config/autostart/iterm.desktop");
        
        Ok(desktop_path.exists())
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Ok(false)
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            #[cfg(any(windows, target_os = "macos"))]
            {
                if let Some(window) = app.get_webview_window("main") {
                    window_manager::set_window_on_all_desktops(&window);
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            mouse::start_mouse_listener,
            mouse::stop_mouse_listener,
            mouse::is_mouse_listener_running,
            set_autostart,
            is_autostart_enabled,
            check_directory_exists,
            create_directory
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
