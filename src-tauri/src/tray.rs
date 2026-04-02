use tauri::{AppHandle, Manager};

/// 创建系统托盘（Windows）或 Dock 菜单（macOS）
pub fn create_system_menu(app_handle: &AppHandle) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    create_windows_tray(app_handle)?;

    #[cfg(target_os = "macos")]
    create_macos_dock_menu(app_handle)?;

    #[cfg(target_os = "linux")]
    create_linux_tray(app_handle)?;

    Ok(())
}

/// Windows 系统托盘
#[cfg(target_os = "windows")]
fn create_windows_tray(app_handle: &AppHandle) -> Result<(), String> {
    use tray_icon::menu::{Menu, MenuEvent, MenuItem};
    use tray_icon::TrayIcon;

    // 创建菜单项 - 使用自定义 ID
    let show_main_item = MenuItem::with_id("show_main", "打开主页", true, None);
    let quit_item = MenuItem::with_id("quit_app", "退出应用", true, None);

    let menu = Menu::with_items(&[&show_main_item, &quit_item])
        .map_err(|e| format!("Failed to create menu: {}", e))?;

    // 从资源加载图标
    let icon = load_default_icon(app_handle)?;

    // 创建托盘图标并 leak 以保持其生命周期
    let _tray = Box::leak(Box::new(
        TrayIcon::new(tray_icon::TrayIconAttributes {
            menu: Some(Box::new(menu)),
            tooltip: Some("LINKRON".to_string()),
            icon: Some(icon),
            ..Default::default()
        })
        .map_err(|e| format!("Failed to create tray icon: {}", e))?,
    ));

    // 处理菜单点击事件
    let app_handle = app_handle.clone();
    std::thread::spawn(move || {
        let menu_channel = MenuEvent::receiver();
        for event in menu_channel {
            match event.id().0.as_str() {
                "show_main" => {
                    if let Some(window) = app_handle.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "quit_app" => {
                    app_handle.exit(0);
                }
                _ => {}
            }
        }
    });

    Ok(())
}

/// macOS Dock 菜单
#[cfg(target_os = "macos")]
fn create_macos_dock_menu(app_handle: &AppHandle) -> Result<(), String> {
    use cocoa::appkit::{NSApp, NSMenu, NSMenuItem};
    use cocoa::base::{id, nil};
    use cocoa::foundation::{NSAutoreleasePool, NSString};
    use objc::runtime::Object;
    use objc::{class, msg_send, sel, sel_impl};

    unsafe {
        // 获取共享应用实例
        let app = NSApp();
        if app == nil {
            return Err("Failed to get NSApp".to_string());
        }

        // 创建菜单
        let menu = NSMenu::new(nil).autorelease();

        // 创建空字符串用于 keyEquivalent
        let empty_key = NSString::alloc(nil).init_str("");

        // "打开主页" 菜单项
        let show_item_title = NSString::alloc(nil).init_str("打开主页");
        let show_item: id = msg_send![class!(NSMenuItem), newItemWithTitle: show_item_title
                                    action: sel!(showMainWindow:)
                                     keyEquivalent: empty_key];
        show_item.setTarget_(app as *mut Object);

        // "退出应用" 菜单项
        let quit_item_title = NSString::alloc(nil).init_str("退出应用");
        let quit_item: id = msg_send![class!(NSMenuItem), newItemWithTitle: quit_item_title
                                    action: sel!(quitApp:)
                                     keyEquivalent: empty_key];
        quit_item.setTarget_(app as *mut Object);

        // 添加到菜单
        menu.addItem_(show_item);
        menu.addItem_(quit_item);

        // 设置为 Dock 菜单
        let _: () = msg_send![app, setDockMenu: menu];
    }

    Ok(())
}

/// Linux 系统托盘
#[cfg(target_os = "linux")]
fn create_linux_tray(app_handle: &AppHandle) -> Result<(), String> {
    use tray_icon::menu::{Menu, MenuEvent, MenuItem};
    use tray_icon::TrayIcon;

    let show_main_item = MenuItem::with_id("show_main", "打开主页", true, None);
    let quit_item = MenuItem::with_id("quit_app", "退出应用", true, None);

    let menu = Menu::with_items(&[&show_main_item, &quit_item])
        .map_err(|e| format!("Failed to create menu: {}", e))?;

    let icon = load_default_icon(app_handle)?;

    let _tray = Box::leak(Box::new(
        TrayIcon::new(tray_icon::TrayIconAttributes {
            menu: Some(Box::new(menu)),
            tooltip: Some("LINKRON".to_string()),
            icon: Some(icon),
            ..Default::default()
        })
        .map_err(|e| format!("Failed to create tray icon: {}", e))?,
    ));

    let app_handle = app_handle.clone();
    std::thread::spawn(move || {
        let menu_channel = MenuEvent::receiver();
        for event in menu_channel {
            match event.id().0.as_str() {
                "show_main" => {
                    if let Some(window) = app_handle.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "quit_app" => {
                    app_handle.exit(0);
                }
                _ => {}
            }
        }
    });

    Ok(())
}

/// 加载应用图标（仅 Windows 和 Linux）
#[cfg(any(target_os = "windows", target_os = "linux"))]
fn load_default_icon(app_handle: &AppHandle) -> Result<tray_icon::Icon, String> {
    // 优先使用嵌入的图标数据（更可靠）
    if let Ok(icon) = try_embedded_icon() {
        return Ok(icon);
    }

    // 尝试多个路径获取图标
    let resource_paths = vec![
        // 相对于资源目录的路径
        "icons/32x32.png",
    ];

    let mut icon_data = None;
    let mut tried_paths = Vec::new();

    for path in resource_paths {
        tried_paths.push(path.to_string());

        // 尝试使用 Tauri 的资源解析器
        if let Ok(resolved_path) = app_handle.path().resolve(path, tauri::path::BaseDirectory::Resource) {
            if resolved_path.exists() {
                if let Ok(data) = std::fs::read(&resolved_path) {
                    icon_data = Some(data);
                    break;
                }
            }
        }

        // 尝试直接读取（开发环境）
        if std::path::Path::new(path).exists() {
            if let Ok(data) = std::fs::read(path) {
                icon_data = Some(data);
                break;
            }
        }
    }

    let icon_data = icon_data.ok_or_else(|| {
        format!(
            "Failed to find icon. Tried paths: {:?}",
            tried_paths
        )
    })?;

    // 使用 image crate 解析 PNG
    let image = image::load_from_memory(&icon_data)
        .map_err(|e| format!("Failed to parse icon image: {}", e))?
        .to_rgba8();

    let icon = tray_icon::Icon::from_rgba(image.as_raw().to_vec(), image.width(), image.height())
        .map_err(|e| format!("Failed to create icon: {}", e))?;

    Ok(icon)
}

/// 尝试创建简单的内置图标（备用方案）
#[cfg(any(target_os = "windows", target_os = "linux"))]
fn try_embedded_icon() -> Result<tray_icon::Icon, String> {
    use std::io::Cursor;

    // 创建一个简单的 32x32 RGBA 图像
    let mut img = image::RgbaImage::new(32, 32);

    // 绘制一个简单的圆形图标
    let center_x = 16;
    let center_y = 16;
    let radius = 14;

    for y in 0..32 {
        for x in 0..32 {
            let dx = x as i32 - center_x;
            let dy = y as i32 - center_y;
            let dist = ((dx * dx + dy * dy) as f32).sqrt();

            if dist <= radius as f32 {
                // 渐变效果
                let alpha = if dist < radius as f32 - 2.0 {
                    255
                } else {
                    ((radius as f32 - dist) * 127.0).clamp(0.0, 255.0) as u8
                };
                // 使用蓝色作为图标颜色
                img.put_pixel(x, y, image::Rgba([59, 130, 246, alpha]));
            } else {
                img.put_pixel(x, y, image::Rgba([0, 0, 0, 0]));
            }
        }
    }

    tray_icon::Icon::from_rgba(img.as_raw().to_vec(), img.width(), img.height())
        .map_err(|e| format!("Failed to create embedded icon: {}", e))
}

/// 显示主窗口
#[tauri::command]
pub fn show_main_window(app_handle: AppHandle) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("找不到主窗口".to_string())
    }
}

/// 退出应用
#[tauri::command]
pub fn quit_app(app_handle: AppHandle) -> Result<(), String> {
    app_handle.exit(0);
    Ok(())
}
