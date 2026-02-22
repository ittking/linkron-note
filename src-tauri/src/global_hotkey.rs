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

/// 解析组合键字符串（如 "Alt + Space", "Command + Enter"）
fn parse_hotkey_combo(hotkey_str: &str) -> Option<(Option<Modifiers>, Code)> {
    let parts: Vec<&str> = hotkey_str
        .split('+')
        .map(|s| s.trim())
        .collect();

    if parts.is_empty() {
        return None;
    }

    let mut modifiers = Vec::new();
    let mut trigger_key = None;

    // 解析修饰键
    for part in &parts {
        let part_lower = part.to_lowercase();
        match part_lower.as_str() {
            "alt" | "option" => modifiers.push(Modifiers::ALT),
            "control" | "ctrl" => modifiers.push(Modifiers::CONTROL),
            "command" | "cmd" | "meta" => modifiers.push(Modifiers::SUPER),
            "shift" => modifiers.push(Modifiers::SHIFT),
            _ => {
                // 不是修饰键，可能是触发键
                trigger_key = Some(*part);
            }
        }
    }

    // 解析触发键
    let key_code = match trigger_key {
        Some(key) => {
            let key_lower = key.to_lowercase();
            match key_lower.as_str() {
                "space" => Some(Code::Space),
                "enter" | "return" => Some(Code::Enter),
                "tab" => Some(Code::Tab),
                "escape" | "esc" => Some(Code::Escape),
                "backspace" => Some(Code::Backspace),
                "delete" | "del" => Some(Code::Delete),
                // 字母键
                "a" => Some(Code::KeyA),
                "b" => Some(Code::KeyB),
                "c" => Some(Code::KeyC),
                "d" => Some(Code::KeyD),
                "e" => Some(Code::KeyE),
                "f" => Some(Code::KeyF),
                "g" => Some(Code::KeyG),
                "h" => Some(Code::KeyH),
                "i" => Some(Code::KeyI),
                "j" => Some(Code::KeyJ),
                "k" => Some(Code::KeyK),
                "l" => Some(Code::KeyL),
                "m" => Some(Code::KeyM),
                "n" => Some(Code::KeyN),
                "o" => Some(Code::KeyO),
                "p" => Some(Code::KeyP),
                "q" => Some(Code::KeyQ),
                "r" => Some(Code::KeyR),
                "s" => Some(Code::KeyS),
                "t" => Some(Code::KeyT),
                "u" => Some(Code::KeyU),
                "v" => Some(Code::KeyV),
                "w" => Some(Code::KeyW),
                "x" => Some(Code::KeyX),
                "y" => Some(Code::KeyY),
                "z" => Some(Code::KeyZ),
                // 数字键
                "0" => Some(Code::Digit0),
                "1" => Some(Code::Digit1),
                "2" => Some(Code::Digit2),
                "3" => Some(Code::Digit3),
                "4" => Some(Code::Digit4),
                "5" => Some(Code::Digit5),
                "6" => Some(Code::Digit6),
                "7" => Some(Code::Digit7),
                "8" => Some(Code::Digit8),
                "9" => Some(Code::Digit9),
                _ => None,
            }
        }
        None => None,
    };

    let code = key_code?;

    // 合并所有修饰键
    let combined_modifiers = if modifiers.is_empty() {
        None
    } else {
        Some(modifiers.into_iter().fold(
            Modifiers::empty(),
            |acc, m| acc | m
        ))
    };

    Some((combined_modifiers, code))
}

/// 将按键名称转换为 GlobalHotKey（兼容旧版单键格式）
fn parse_hotkey(key_name: &str) -> Option<GlobalHotKey> {
    // 检查是否是组合键格式（包含 "+"）
    if key_name.contains('+') {
        if let Some((modifiers, code)) = parse_hotkey_combo(key_name) {
            return Some(GlobalHotKey::new(modifiers, code));
        }
        return None;
    }

    // 单键格式（仅修饰键 + 空格）
    let key_name = key_name.to_lowercase();

    let modifiers = match key_name.as_str() {
        "alt" | "option" => Some(Modifiers::ALT),
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
    // 先保存旧快捷键，以便在注册失败时恢复
    let old_hotkey: Option<GlobalHotKey> = {
        if let Ok(current_guard) = CURRENT_HOTKEY.lock() {
            *current_guard
        } else {
            None
        }
    };

    // 注销旧快捷键
    if let Some(old) = old_hotkey {
        if let Ok(manager_guard) = HOTKEY_MANAGER.lock() {
            if let Some(manager) = manager_guard.as_ref() {
                let _: Result<_, _> = manager.unregister(old);
            }
        }
    }

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
        .map_err(|e| {
            let error_msg = format!("注册快捷键失败: {}", e);

            // 尝试恢复旧快捷键
            if let Some(old) = old_hotkey {
                if let Some(manager) = manager_guard.as_ref() {
                    let _: Result<_, _> = manager.register(old);
                }
            }

            error_msg
        })?;

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
