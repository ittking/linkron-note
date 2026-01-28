use std::time::Duration;

/// 爬取网页 HTML 内容
#[tauri::command]
pub async fn fetch_webpage_html(url: String) -> Result<String, String> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("URL 必须以 http:// 或 https:// 开头".to_string());
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let response = client
        .get(&url)
        .header(
            "Accept",
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8",
        )
        .header("Accept-Language", "zh-CN,zh;q=0.9,en;q=0.8")
        .send()
        .await
        .map_err(|e| format!("请求网页失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("请求失败，状态码: {}", response.status()));
    }

    let html = response
        .text()
        .await
        .map_err(|e| format!("读取网页内容失败: {}", e))?;

    Ok(html)
}