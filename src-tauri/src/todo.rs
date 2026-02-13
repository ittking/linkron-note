use rusqlite::{Connection, Result as SqliteResult, params};
use serde::{Deserialize, Serialize};

/// 待办事项提醒类型
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ReminderType {
    None,
    OneTime,
    Repeat,
}

/// 待办事项重复规则
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum RepeatRule {
    Day,
    Weekday,
    Month,
    Year,
}

/// 待办事项提醒配置
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TodoReminder {
    #[serde(rename = "type")]
    pub reminder_type: ReminderType,
    #[serde(rename = "repeatRule")]
    pub repeat_rule: Option<RepeatRule>,
    #[serde(rename = "repeatInterval")]
    pub repeat_interval: Option<u32>,
    #[serde(rename = "repeatTime")]
    pub repeat_time: Option<String>,
    #[serde(rename = "repeatDayOfWeek")]
    pub repeat_day_of_week: Option<Vec<u32>>,
    #[serde(rename = "repeatDayOfMonth")]
    pub repeat_day_of_month: Option<Vec<u32>>,
    #[serde(rename = "repeatMonth")]
    pub repeat_month: Option<u32>,
    #[serde(rename = "lastNotified")]
    pub last_notified: Option<String>,
}

/// 待办事项
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Todo {
    pub id: i64,
    pub date: String,
    pub text: String,
    pub status: String,
    pub reminder: Option<TodoReminder>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

/// 初始化待办事项表
pub fn init_tables(conn: &Connection) -> SqliteResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS todos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            date TEXT NOT NULL,
            text TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'todo',
            reminder TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )",
        [],
    )?;

    // 创建索引
    conn.execute("CREATE INDEX IF NOT EXISTS idx_todos_date ON todos(date)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_todos_status ON todos(status)", [])?;

    Ok(())
}

/// 创建待办事项
pub fn create_todo(
    conn: &Connection,
    date: &str,
    text: &str,
    status: &str,
    reminder: Option<String>,
) -> SqliteResult<i64> {
    let now = chrono::Utc::now().to_rfc3339();
    let reminder_json = reminder.unwrap_or_else(|| serde_json::to_string(&TodoReminder {
        reminder_type: ReminderType::None,
        repeat_rule: None,
        repeat_interval: None,
        repeat_time: None,
        repeat_day_of_week: None,
        repeat_day_of_month: None,
        repeat_month: None,
        last_notified: None,
    }).unwrap_or_default());

    conn.execute(
        "INSERT INTO todos (date, text, status, reminder, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![date, text, status, reminder_json, &now, &now],
    )?;

    Ok(conn.last_insert_rowid())
}

/// 更新待办事项
pub fn update_todo(
    conn: &Connection,
    id: i64,
    text: &str,
    status: &str,
    reminder: Option<String>,
) -> SqliteResult<()> {
    let now = chrono::Utc::now().to_rfc3339();
    let reminder_json = reminder.unwrap_or_else(|| serde_json::to_string(&TodoReminder {
        reminder_type: ReminderType::None,
        repeat_rule: None,
        repeat_interval: None,
        repeat_time: None,
        repeat_day_of_week: None,
        repeat_day_of_month: None,
        repeat_month: None,
        last_notified: None,
    }).unwrap_or_default());

    conn.execute(
        "UPDATE todos SET text = ?1, status = ?2, reminder = ?3, updated_at = ?4 WHERE id = ?5",
        params![text, status, reminder_json, &now, id],
    )?;

    Ok(())
}

/// 删除待办事项
pub fn delete_todo(conn: &Connection, id: i64) -> SqliteResult<()> {
    conn.execute("DELETE FROM todos WHERE id = ?", params![id])?;
    Ok(())
}

/// 获取指定日期的待办事项
pub fn get_todos_by_date(conn: &Connection, date: &str) -> SqliteResult<Vec<Todo>> {
    let mut stmt = conn.prepare(
        "SELECT id, date, text, status, reminder, created_at, updated_at
         FROM todos WHERE date = ?1 ORDER BY created_at ASC"
    )?;

    let todos = stmt.query_map(params![date], |row| {
        let reminder_json: String = row.get(4)?;
        let reminder: Option<TodoReminder> = serde_json::from_str(&reminder_json).ok();
        Ok(Todo {
            id: row.get(0)?,
            date: row.get(1)?,
            text: row.get(2)?,
            status: row.get(3)?,
            reminder,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    })?;

    let mut result: Vec<Todo> = todos.collect::<SqliteResult<Vec<_>>>()?;

    // 排序：未完成 > 进行中 > 暂停 > 已取消 > 已完成；同状态按创建时间排序（新的在前）
    result.sort_by(|a, b| {
        // 定义状态优先级：数字越小优先级越高
        fn status_priority(status: &str) -> i32 {
            match status {
                "todo" => 1,           // 未完成 - 最高优先级
                "in-progress" => 2,    // 进行中
                "pending" => 3,        // 暂停
                "cancelled" => 4,      // 已取消
                "completed" => 5,      // 已完成 - 最低优先级
                _ => 6,                // 其他状态
            }
        }

        let a_priority = status_priority(&a.status);
        let b_priority = status_priority(&b.status);

        if a_priority != b_priority {
            // 优先级不同，按优先级排序（数字小的在前）
            a_priority.cmp(&b_priority)
        } else {
            // 同状态按创建时间排序（新的在前）
            let a_time = chrono::DateTime::parse_from_rfc3339(&a.created_at)
                .unwrap_or_else(|_| chrono::Utc::now().into())
                .timestamp();
            let b_time = chrono::DateTime::parse_from_rfc3339(&b.created_at)
                .unwrap_or_else(|_| chrono::Utc::now().into())
                .timestamp();
            b_time.cmp(&a_time)
        }
    });

    Ok(result)
}

/// 获取指定月份的待办事项
pub fn get_todos_by_month(conn: &Connection, year: i32, month: i32) -> SqliteResult<Vec<Todo>> {
    let month_str = format!("{:02}", month);
    let pattern = format!("{}-{}%", year, month_str);

    let mut stmt = conn.prepare(
        "SELECT id, date, text, status, reminder, created_at, updated_at
         FROM todos WHERE date LIKE ?1 ORDER BY date, created_at ASC"
    )?;

    let todos = stmt.query_map(params![pattern], |row| {
        let reminder_json: String = row.get(4)?;
        let reminder: Option<TodoReminder> = serde_json::from_str(&reminder_json).ok();
        Ok(Todo {
            id: row.get(0)?,
            date: row.get(1)?,
            text: row.get(2)?,
            status: row.get(3)?,
            reminder,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    })?;

    let mut result: Vec<Todo> = todos.collect::<SqliteResult<Vec<_>>>()?;

    // 排序：未完成 > 进行中 > 暂停 > 已取消 > 已完成；同状态按创建时间排序（新的在前）
    result.sort_by(|a, b| {
        // 定义状态优先级：数字越小优先级越高
        fn status_priority(status: &str) -> i32 {
            match status {
                "todo" => 1,           // 未完成 - 最高优先级
                "in-progress" => 2,    // 进行中
                "pending" => 3,        // 暂停
                "cancelled" => 4,      // 已取消
                "completed" => 5,      // 已完成 - 最低优先级
                _ => 6,                // 其他状态
            }
        }

        let a_priority = status_priority(&a.status);
        let b_priority = status_priority(&b.status);

        if a_priority != b_priority {
            // 优先级不同，按优先级排序（数字小的在前）
            a_priority.cmp(&b_priority)
        } else {
            // 同状态按创建时间排序（新的在前）
            let a_time = chrono::DateTime::parse_from_rfc3339(&a.created_at)
                .unwrap_or_else(|_| chrono::Utc::now().into())
                .timestamp();
            let b_time = chrono::DateTime::parse_from_rfc3339(&b.created_at)
                .unwrap_or_else(|_| chrono::Utc::now().into())
                .timestamp();
            b_time.cmp(&a_time)
        }
    });

    Ok(result)
}

/// 获取需要提醒的待办事项



pub fn get_reminders(conn: &Connection) -> SqliteResult<Vec<Todo>> {



    let current_time = chrono::Local::now();







    let mut stmt = conn.prepare(



        "SELECT id, date, text, status, reminder, created_at, updated_at



         FROM todos



         WHERE status NOT IN ('completed', 'cancelled')



         AND reminder IS NOT NULL



         ORDER BY date, created_at ASC"



    )?;







    let todos = stmt.query_map([], |row| {



        let reminder_json: String = row.get(4)?;



        let reminder: Option<TodoReminder> = serde_json::from_str(&reminder_json).ok();



        Ok(Todo {



            id: row.get(0)?,



            date: row.get(1)?,



            text: row.get(2)?,



            status: row.get(3)?,



            reminder,



            created_at: row.get(5)?,



            updated_at: row.get(6)?,



        })



    })?;







    let todos: Vec<Todo> = todos.collect::<SqliteResult<Vec<_>>>()?;







    let mut filtered_todos = Vec::new();







    for todo in todos {



        if let Some(reminder) = &todo.reminder {



            match reminder.reminder_type {



                ReminderType::OneTime => {



                    if let Some(ref repeat_time) = reminder.repeat_time {



                        if let Ok(reminder_time) = chrono::NaiveDateTime::parse_from_str(



                            repeat_time,



                            "%Y-%m-%dT%H:%M"



                        ) {



                            // 转换为本地时间进行比较



                            let reminder_local = reminder_time.and_local_timezone(chrono::Local).unwrap();



                            



                            // 计算时间差



                            let time_diff = current_time.signed_duration_since(reminder_local);



                            



                            // 只有在当前时间的前后10秒内才返回



                            // 这样可以避免边界情况下的过期判断问题



                            if time_diff.num_seconds() >= -10 && time_diff.num_seconds() <= 10 {



                                filtered_todos.push(todo);



                            }



                        }



                    }



                }



                ReminderType::Repeat => {



                    // 重复提醒不做时间限制，直接返回



                    filtered_todos.push(todo);



                }



                ReminderType::None => {}



            }



        }



    }







    Ok(filtered_todos)



}

/// 统计待办事项数量（排除已完成、已取消、已暂停）
pub fn count_todos(conn: &Connection) -> SqliteResult<i64> {
    conn.query_row(
        "SELECT COUNT(*) FROM todos WHERE status NOT IN ('completed', 'cancelled', 'pending')",
        [],
        |row| row.get(0)
    )
}

/// 获取今日待办事项
/// 获取所有日期为今天的待办事项，按状态排序：未完成的在前，已完成的在后
pub fn get_today_todos(conn: &Connection, today_date: &str) -> SqliteResult<Vec<Todo>> {
    // 获取今日的所有待办事项
    let mut stmt = conn.prepare(
        "SELECT id, date, text, status, reminder, created_at, updated_at
         FROM todos WHERE date = ?1 ORDER BY created_at ASC"
    )?;

    let mut result = stmt.query_map(params![today_date], |row| {
        let reminder_json: String = row.get(4)?;
        let reminder: Option<TodoReminder> = serde_json::from_str(&reminder_json).ok();
        Ok(Todo {
            id: row.get(0)?,
            date: row.get(1)?,
            text: row.get(2)?,
            status: row.get(3)?,
            reminder,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    })?.collect::<SqliteResult<Vec<_>>>()?;

    // 排序：未完成 > 进行中 > 暂停 > 已取消 > 已完成；同状态按创建时间排序（新的在前）
    result.sort_by(|a, b| {
        // 定义状态优先级：数字越小优先级越高
        fn status_priority(status: &str) -> i32 {
            match status {
                "todo" => 1,           // 未完成 - 最高优先级
                "in-progress" => 2,    // 进行中
                "pending" => 3,        // 暂停
                "cancelled" => 4,      // 已取消
                "completed" => 5,      // 已完成 - 最低优先级
                _ => 6,                // 其他状态
            }
        }

        let a_priority = status_priority(&a.status);
        let b_priority = status_priority(&b.status);

        if a_priority != b_priority {
            // 优先级不同，按优先级排序（数字小的在前）
            a_priority.cmp(&b_priority)
        } else {
            // 同状态按创建时间排序（新的在前）
            let a_time = chrono::DateTime::parse_from_rfc3339(&a.created_at)
                .unwrap_or_else(|_| chrono::Utc::now().into())
                .timestamp();
            let b_time = chrono::DateTime::parse_from_rfc3339(&b.created_at)
                .unwrap_or_else(|_| chrono::Utc::now().into())
                .timestamp();
            b_time.cmp(&a_time)
        }
    });

    Ok(result)
}