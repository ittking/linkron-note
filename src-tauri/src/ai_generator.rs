use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Choice {
    message: ChatMessage,
}

/// 使用 AI 生成正则表达式
#[tauri::command]
pub async fn generate_regex(
    prompt: String,
    provider: String,
    api_key: String,
    api_url: String,
    model: String,
) -> Result<String, String> {
    let client = reqwest::Client::new();
    
    let base_url = if api_url.is_empty() {
        get_default_api_url(&provider)
    } else {
        api_url
    };
    
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
    
    let messages = vec![ChatMessage {
        role: "user".to_string(),
        content: prompt,
    }];
    
    let request_body = ChatRequest {
        model: model,
        messages,
    };
    
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        "Authorization",
        format!("Bearer {}", api_key).parse().unwrap(),
    );
    headers.insert("Content-Type", "application/json".parse().unwrap());
    
    let response = client
        .post(&url)
        .headers(headers)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "无法读取错误信息".to_string());
        return Err(format!("请求失败: {} - {}", status, error_text));
    }
    
    let chat_response: ChatResponse = response
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;
    
    if let Some(choice) = chat_response.choices.first() {
        Ok(choice.message.content.trim().to_string())
    } else {
        Err("未收到有效的响应".to_string())
    }
}

fn get_default_api_url(provider: &str) -> String {
    match provider {
        "deepseek" => "https://api.deepseek.com/v1".to_string(),
        "siliconflow" => "https://api.siliconflow.cn/v1".to_string(),
        "kimi" => "https://api.moonshot.cn/v1".to_string(),
        "zhipu" => "https://open.bigmodel.cn/api/paas/v4".to_string(),
        _ => "https://api.openai.com/v1".to_string(),
    }
}