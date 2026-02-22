// 全局快捷键模块
// 使用 global-hotkey 库实现真正的全局快捷键
use global_hotkey::{
    hotkey::{Code, HotKey as GlobalHotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager,
};
use once_cell::sync::Lazy;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};

// 全局快捷键管理器（静态）
static HOTKEY_MANAGER: Lazy<Mutex<Option<GlobalHotKeyManager>>> = Lazy::new(|| Mutex::new(None));

// 当前注册的快捷键（静态）
static CURRENT_HOTKEY: Lazy<Mutex<Option<GlobalHotKey>>> = Lazy::new(|| Mutex::new(None));

// AppHandle（用于发送事件到前端）
static APP_HANDLE: Lazy<Mutex<Option<AppHandle>>> = Lazy::new(|| Mutex::new(None));

/// 将按键名称转换为 GlobalHotKey（使用组合键：修饰键 + 空格）
fn parse_hotkey(key_name: &str) -> Option<GlobalHotKey> {
    let key_name = key_name.to_lowercase();
    
    let modifiers = match key_name.as_str() {
        "alt" => Some(Modifiers::ALT),
        "option" => Some(Modifiers::ALT), // macOS 上的 Option 对应 Alt
        "control" | "ctrl" => Some(Modifiers::CONTROL),
        "command" | "cmd" | "meta" => Some(Modifiers::SUPER),
        "shift" => Some(Modifiers::SHIFT),
        _ => return None,
    };

    // 使用空格键作为触发键，形成组合键
    Some(GlobalHotKey::new(modifiers, Code::Space))
}

/// 注册全局快捷键
#[tauri::command]
pub fn register_hotkey(app_handle: AppHandle, key_name: String) -> Result<(), String> {
    // 先注销之前的快捷键
    unregister_hotkey();

    // 解析新的快捷键
    let hotkey = parse_hotkey(&key_name)
        .ok_or_else(|| format!("不支持的按键: {}", key_name))?;

    // 保存 AppHandle
    {
        let mut handle_guard = APP_HANDLE.lock()
            .map_err(|e| format!("获取 AppHandle 失败: {}", e))?;
        *handle_guard = Some(app_handle);
    }

    // 获取或创建快捷键管理器
    let mut manager_guard = HOTKEY_MANAGER.lock()
        .map_err(|e| format!("获取管理器失败: {}", e))?;

    if manager_guard.is_none() {
        *manager_guard = Some(
            GlobalHotKeyManager::new()
                .map_err(|e| format!("创建快捷键管理器失败: {}", e))?
        );
    }

    let manager = manager_guard.as_ref()
        .ok_or_else(|| "快捷键管理器未初始化".to_string())?;

    // 注册快捷键
    manager
        .register(hotkey)
        .map_err(|e| format!("注册快捷键失败: {}", e))?;

    // 保存当前快捷键
    {
        let mut current_guard = CURRENT_HOTKEY.lock()
            .map_err(|e| format!("获取当前快捷键失败: {}", e))?;
        *current_guard = Some(hotkey);
    }

    Ok(())
}

/// 注销当前的全局快捷键
#[tauri::command]
pub fn unregister_hotkey() {
    // 先获取要注销的快捷键
    let hotkey_to_unregister: Option<GlobalHotKey> = {
        if let Ok(current_guard) = CURRENT_HOTKEY.lock() {
            *current_guard
        } else {
            None
        }
    };

    // 如果有快捷键需要注销
    if let Some(hotkey) = hotkey_to_unregister {
        if let Ok(manager_guard) = HOTKEY_MANAGER.lock() {
            if let Some(manager) = manager_guard.as_ref() {
                let _: Result<_, _> = manager.unregister(hotkey);
            }
        }
    }
    
    // 清空当前快捷键
    if let Ok(mut current_guard) = CURRENT_HOTKEY.lock() {
        *current_guard = None;
    }
}

/// 初始化快捷键事件监听
pub fn init_hotkey_listener(app_handle: AppHandle) {
    std::thread::spawn(move || {
        let rx = GlobalHotKeyEvent::receiver();
        
        while let Ok(event) = rx.recv() {
            // 检查事件类型
            // event.state 是 HotKeyState 类型，可以直接比较
            if event.state == global_hotkey::HotKeyState::Pressed {
                // 发送事件到前端
                let _ = app_handle.emit("global-hotkey-triggered", ());
            }
        }
    });
}

/// 处理按键按下事件（已弃用，保留用于兼容）
#[tauri::command]
pub fn handle_key_press_event(_key_name: String) -> Result<bool, String> {
    // 不再需要，因为使用全局快捷键
    Ok(false)
}

/// 获取支持的按键列表
#[tauri::command]
pub fn get_supported_keys() -> Vec<String> {
    #[cfg(windows)]
    {
        vec![
            "Alt".to_string(),
            "Control".to_string(),
            "Shift".to_string(),
        ]
    }

    #[cfg(target_os = "macos")]
    {
        vec![
            "Option".to_string(),
            "Control".to_string(),
            "Command".to_string(),
            "Shift".to_string(),
        ]
    }

    #[cfg(not(any(windows, target_os = "macos")))]
    {
        vec![
            "Alt".to_string(),
            "Control".to_string(),
            "Shift".to_string(),
        ]
    }
}