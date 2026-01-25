use rdev::{Event, EventType, listen};
use std::sync::{Arc, Mutex};
use tauri::Emitter;

// 鼠标事件类型
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum MouseEventType {
    Move,
    ButtonPress,
    ButtonRelease,
    Wheel,
}

// 鼠标事件数据
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct MouseEvent {
    pub event_type: MouseEventType,
    pub x: i32,
    pub y: i32,
    pub button: Option<String>,
    pub wheel_delta: Option<i32>,
}

// 全局监听器状态
struct MouseListener {
    is_running: Arc<Mutex<bool>>,
    last_position: Arc<Mutex<(f64, f64)>>,
}

impl MouseListener {
    fn new() -> Self {
        MouseListener {
            is_running: Arc::new(Mutex::new(false)),
            last_position: Arc::new(Mutex::new((0.0, 0.0))),
        }
    }
}

lazy_static::lazy_static! {
    static ref MOUSE_LISTENER: MouseListener = MouseListener::new();
}

// 开始监听鼠标事件
#[tauri::command]
pub fn start_mouse_listener(app_handle: tauri::AppHandle) -> Result<String, String> {
    let mut is_running = MOUSE_LISTENER.is_running.lock().map_err(|e| e.to_string())?;
    
    if *is_running {
        return Ok("鼠标监听器已经在运行".to_string());
    }
    
    *is_running = true;
    drop(is_running);
    
    let is_running_clone = MOUSE_LISTENER.is_running.clone();
    let last_position_clone = MOUSE_LISTENER.last_position.clone();
    
    std::thread::spawn(move || {
        let callback = move |event: Event| {
            if !*is_running_clone.lock().unwrap() {
                return;
            }
            
            let mouse_event = match event.event_type {
                EventType::MouseMove { x, y } => {
                    // 更新最后已知位置
                    *last_position_clone.lock().unwrap() = (x, y);
                    Some(MouseEvent {
                        event_type: MouseEventType::Move,
                        x: x as i32,
                        y: y as i32,
                        button: None,
                        wheel_delta: None,
                    })
                }
                EventType::ButtonPress(button) => {
                    let button_str = format!("{:?}", button);
                    let (x, y) = *last_position_clone.lock().unwrap();
                    Some(MouseEvent {
                        event_type: MouseEventType::ButtonPress,
                        x: x as i32,
                        y: y as i32,
                        button: Some(button_str),
                        wheel_delta: None,
                    })
                }
                EventType::ButtonRelease(button) => {
                    let button_str = format!("{:?}", button);
                    let (x, y) = *last_position_clone.lock().unwrap();
                    Some(MouseEvent {
                        event_type: MouseEventType::ButtonRelease,
                        x: x as i32,
                        y: y as i32,
                        button: Some(button_str),
                        wheel_delta: None,
                    })
                }
                EventType::Wheel { delta_x: _, delta_y } => {
                    let (x, y) = *last_position_clone.lock().unwrap();
                    Some(MouseEvent {
                        event_type: MouseEventType::Wheel,
                        x: x as i32,
                        y: y as i32,
                        button: None,
                        wheel_delta: Some(delta_y as i32),
                    })
                }
                _ => None,
            };
            
            if let Some(evt) = mouse_event {
                // 发送事件到前端
                if let Err(e) = app_handle.emit("mouse-event", evt) {
                    eprintln!("发送鼠标事件失败: {:?}", e);
                }
            }
        };
        
        if let Err(error) = listen(callback) {
            eprintln!("鼠标监听错误: {:?}", error);
        }
    });
    
    Ok("鼠标监听器已启动".to_string())
}

// 停止监听鼠标事件
#[tauri::command]
pub fn stop_mouse_listener() -> Result<String, String> {
    let mut is_running = MOUSE_LISTENER.is_running.lock().map_err(|e| e.to_string())?;
    
    if !*is_running {
        return Ok("鼠标监听器未在运行".to_string());
    }
    
    *is_running = false;
    
    Ok("鼠标监听器已停止".to_string())
}

// 获取鼠标监听器状态
#[tauri::command]
pub fn is_mouse_listener_running() -> Result<bool, String> {
    let is_running = MOUSE_LISTENER.is_running.lock().map_err(|e| e.to_string())?;
    Ok(*is_running)
}