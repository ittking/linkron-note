use rusqlite::{Connection, Result as SqliteResult, params};
use serde::{Deserialize, Serialize};
use chrono::Datelike;

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
    pub repeat_day_of_week: Option<u32>,
    #[serde(rename = "repeatDayOfMonth")]
    pub repeat_day_of_month: Option<u32>,
    #[serde(rename = "repeatMonth")]
    pub repeat_month: Option<u32>,
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

    todos.collect()
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

    todos.collect()
}

/// 获取需要提醒的待办事项

pub fn get_reminders(conn: &Connection) -> SqliteResult<Vec<Todo>> {

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



    let todos: Vec<Todo> = todos.collect::<Result<Vec<_>, _>>()?;



    // 过滤需要提醒的待办事项

    let mut filtered_todos = Vec::new();

    let current_time = chrono::Utc::now();



    for todo in todos {

        if let Some(reminder) = &todo.reminder {

            match reminder.reminder_type {

                ReminderType::OneTime => {

                    if let Some(ref repeat_time) = reminder.repeat_time {

                        if let Ok(reminder_time) = chrono::NaiveDateTime::parse_from_str(

                            &format!("{} {}", todo.date, repeat_time),

                            "%Y-%m-%d %H:%M"

                        ) {

                            let reminder_datetime = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(

                                reminder_time,

                                chrono::Utc

                            );



                            if reminder_datetime <= current_time && reminder_datetime > current_time - chrono::Duration::minutes(5) {

                                filtered_todos.push(todo);

                            }

                        }

                    }

                }

                ReminderType::Repeat => {

                    // 检查重复提醒逻辑

                    let should_remind = should_send_remind(&todo.date, reminder, &current_time);

                    if should_remind {

                        filtered_todos.push(todo);

                    }

                }

                ReminderType::None => {}

            }

        }

    }



    Ok(filtered_todos)

}

/// 判断是否应该发送提醒
fn should_send_remind(date: &str, reminder: &TodoReminder, current_time: &chrono::DateTime<chrono::Utc>) -> bool {
    let todo_date = match chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d") {
        Ok(d) => d,
        Err(_) => return false,
    };

    let current_date = current_time.date_naive();

    match reminder.repeat_rule {
        Some(RepeatRule::Day) => {
            if let Some(interval) = reminder.repeat_interval {
                let days_diff = (current_date - todo_date).num_days();
                days_diff >= 0 && days_diff % (interval as i64) == 0
            } else {
                current_date == todo_date
            }
        }
        Some(RepeatRule::Weekday) => {
            if let Some(day_of_week) = reminder.repeat_day_of_week {
                current_date.weekday().num_days_from_monday() as u32 == day_of_week - 1
            } else {
                false
            }
        }
        Some(RepeatRule::Month) => {
            if let Some(day_of_month) = reminder.repeat_day_of_month {
                if current_date.day() == day_of_month {
                    if let Some(month) = reminder.repeat_month {
                        current_date.month() as u32 == month
                    } else {
                        true
                    }
                } else {
                    false
                }
            } else {
                false
            }
        }
        Some(RepeatRule::Year) => {
            if let Some(day_of_month) = reminder.repeat_day_of_month {
                if let Some(month) = reminder.repeat_month {
                    current_date.month() as u32 == month && current_date.day() == day_of_month
                } else {
                    false
                }
            } else {
                false
            }
        }
        None => false,
    }
}

/// 统计待办事项数量（排除已完成、已取消、已暂停）
pub fn count_todos(conn: &Connection) -> SqliteResult<i64> {
    conn.query_row(
        "SELECT COUNT(*) FROM todos WHERE status NOT IN ('completed', 'cancelled', 'pending')",
        [],
        |row| row.get(0)
    )
}

/// 获取今日相关的待办事项
/// 包括：
/// 1. 今日创建的待办事项
/// 2. 提醒日期是今天的非重复提醒（一次性提醒）的待办事项
pub fn get_today_todos(conn: &Connection, today_date: &str) -> SqliteResult<Vec<Todo>> {
    // 获取今日创建的待办事项
    let mut stmt = conn.prepare(
        "SELECT id, date, text, status, reminder, created_at, updated_at
         FROM todos WHERE date = ?1 ORDER BY created_at ASC"
    )?;

    let today_todos = stmt.query_map(params![today_date], |row| {
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

    // 获取所有待办事项，筛选出提醒日期是今天的非重复提醒待办
    let mut stmt_all = conn.prepare(
        "SELECT id, date, text, status, reminder, created_at, updated_at
         FROM todos WHERE reminder IS NOT NULL ORDER BY created_at ASC"
    )?;

    let all_todos = stmt_all.query_map([], |row| {
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

    // 筛选出一次性提醒且提醒日期是今天的待办
    let mut reminder_today_todos = Vec::new();
    for todo in all_todos {
        if let Some(reminder) = &todo.reminder {
            if reminder.reminder_type == ReminderType::OneTime {
                if let Some(ref repeat_time) = reminder.repeat_time {
                    // 解析提醒时间，检查日期是否是今天
                    if let Ok(reminder_dt) = chrono::NaiveDateTime::parse_from_str(
                        &format!("{} {}", todo.date, repeat_time),
                        "%Y-%m-%d %H:%M"
                    ) {
                        if reminder_dt.format("%Y-%m-%d").to_string() == today_date {
                            reminder_today_todos.push(todo);
                        }
                    }
                }
            }
        }
    }

    // 合并并去重
    let mut result = today_todos;
    for rt in reminder_today_todos {
        if !result.iter().any(|t| t.id == rt.id) {
            result.push(rt);
        }
    }

    // 排序：未完成的在前，已完成的在后；同状态按创建时间排序（新的在前）
    result.sort_by(|a, b| {
        // 先按状态排序：已完成（completed、cancelled）在后
        let a_completed = a.status == "completed" || a.status == "cancelled";
        let b_completed = b.status == "completed" || b.status == "cancelled";
        
        match (a_completed, b_completed) {
            (true, false) => std::cmp::Ordering::Greater,  // a完成，b未完成，b在前
            (false, true) => std::cmp::Ordering::Less,     // a未完成，b完成，a在前
            _ => {
                // 同状态按创建时间排序（新的在前）
                let a_time = chrono::DateTime::parse_from_rfc3339(&a.created_at)
                    .unwrap_or_else(|_| chrono::Utc::now().into())
                    .timestamp();
                let b_time = chrono::DateTime::parse_from_rfc3339(&b.created_at)
                    .unwrap_or_else(|_| chrono::Utc::now().into())
                    .timestamp();
                b_time.cmp(&a_time)
            }
        }
    });

    Ok(result)
}