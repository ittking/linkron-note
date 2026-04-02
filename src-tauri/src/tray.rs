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

    // 从资源加载图标，如果没有则使用默认图标
    let icon = load_default_icon()?;

    // 创建托盘图标并 leak 以保持其生命周期
    let _tray = Box::leak(Box::new(
        TrayIcon::new(
            tray_icon::TrayIconAttributes {
                menu: Some(Box::new(menu)),
                tooltip: Some("LINKRON".to_string()),
                icon: Some(icon),
                ..Default::default()
            }
        ).map_err(|e| format!("Failed to create tray icon: {}", e))?
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
    use cocoa::appkit::{NSApp, NSApplication, NSMenu, NSMenuItem};
    use cocoa::base::{id, nil};
    use cocoa::foundation::NSString;
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

        // "打开主页" 菜单项
        let show_item_title = NSString::alloc(nil).init_str("打开主页");
        let show_item: id = msg_send![class!(NSMenuItem), newItemWithTitle: show_item_title
                                    action: sel!(showMainWindow:)
                                     keyEquivalent::@""];
        show_item.setTarget_(app as *mut Object);

        // "退出应用" 菜单项
        let quit_item_title = NSString::alloc(nil).init_str("退出应用");
        let quit_item: id = msg_send![class!(NSMenuItem), newItemWithTitle: quit_item_title
                                    action: sel!(quitApp:)
                                     keyEquivalent::@""];
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

    let icon = load_default_icon()?;

    let _tray = Box::leak(Box::new(
        TrayIcon::new(
            tray_icon::TrayIconAttributes {
                menu: Some(Box::new(menu)),
                tooltip: Some("LINKRON".to_string()),
                icon: Some(icon),
                ..Default::default()
            }
        ).map_err(|e| format!("Failed to create tray icon: {}", e))?
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

/// 加载默认图标
fn load_default_icon() -> Result<tray_icon::Icon, String> {
    // 创建一个简单的默认图标 (16x16 像素)
    let mut pixels = vec![255u8; 16 * 16 * 4];

    // 创建一个简单的 L 形状
    for y in 0..16 {
        for x in 0..16 {
            let idx = (y * 16 + x) * 4;
            // 简单的 L 形状 - 第一列和最后一行
            if x == 0 || y == 15 {
                pixels[idx] = 100;     // R
                pixels[idx + 1] = 100; // G
                pixels[idx + 2] = 200; // B
                pixels[idx + 3] = 255; // A
            } else {
                pixels[idx] = 200;     // R
                pixels[idx + 1] = 200; // G
                pixels[idx + 2] = 200; // B
                pixels[idx + 3] = 255; // A
            }
        }
    }

    let icon = tray_icon::Icon::from_rgba(pixels, 16, 16)
        .map_err(|e| format!("Failed to create icon: {}", e))?;

    Ok(icon)
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
