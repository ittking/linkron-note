use std::time::Duration;
use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

/// 提醒任务管理器
/// 在后台定时检查待办提醒并发送系统通知
pub struct ReminderTask {
    app_handle: AppHandle,
    work_directory: Option<String>,
}

impl ReminderTask {
    pub fn new(app_handle: AppHandle, work_directory: Option<String>) -> Self {
        Self {
            app_handle,
            work_directory,
        }
    }

    /// 启动提醒检查任务
    /// 每秒检查一次需要提醒的待办事项
    pub fn start(self) {
        let app_handle = self.app_handle;
        let work_directory = self.work_directory;
        
        tauri::async_runtime::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(1)).await;
                
                // 检查并发送提醒
                if let Err(e) = Self::check_and_send_reminders(&app_handle, &work_directory).await {
                    eprintln!("Failed to check reminders: {}", e);
                }
            }
        });
    }

    /// 检查并发送提醒
    async fn check_and_send_reminders(
        app_handle: &AppHandle,
        work_directory: &Option<String>,
    ) -> Result<(), String> {
        use rusqlite::Connection;
        use chrono::Local;

        // 获取数据库路径
        let db_path = crate::database::get_database_path(work_directory.clone())?;

        // 直接打开数据库连接
        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open database: {}", e))?;

        // 获取需要提醒的待办事项
        let todos = crate::todo::get_reminders(&conn)
            .map_err(|e| format!("Failed to get reminders: {}", e))?;

        // 获取当前分钟
        let current_minute = chrono::Utc::now().format("%Y-%m-%d %H:%M").to_string();

        // 为每个待办发送系统通知并更新 last_notified
        for todo in todos {
            let title = "待办提醒";
            let body = format!("{} - {}", Local::now().format("%H:%M"), todo.text);

            // 使用 Tauri 通知插件发送通知
            app_handle.notification()
                .builder()
                .title(title)
                .body(body)
                .show()
                .map_err(|e| format!("Failed to send notification: {}", e))?;

            // 更新待办事项的 last_notified 字段
            if let Some(mut reminder) = todo.reminder {
                reminder.last_notified = Some(current_minute.clone());
                let reminder_json = serde_json::to_string(&reminder)
                    .map_err(|e| format!("Failed to serialize reminder: {}", e))?;

                conn.execute(
                    "UPDATE todos SET reminder = ?1 WHERE id = ?2",
                    rusqlite::params![reminder_json, todo.id]
                ).map_err(|e| format!("Failed to update reminder: {}", e))?;
            }
        }

        Ok(())
    }
}