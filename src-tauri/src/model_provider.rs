use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelListResponse {
    pub object: String,
    pub data: Vec<ModelInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelInfo {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub owned_by: String,
}

/// 从供应商加载模型列表
#[tauri::command]
pub async fn load_provider_models(
    provider: String,
    api_key: String,
    api_url: String,
) -> Result<Vec<String>, String> {
    let client = reqwest::Client::new();
    
    let base_url = if api_url.is_empty() {
        get_default_api_url(&provider)
    } else {
        api_url
    };
    
    let url = format!("{}/models", base_url.trim_end_matches('/'));
    
    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "无法读取错误信息".to_string());
        return Err(format!("请求失败: {} - {}", status, error_text));
    }
    
    let model_response: ModelListResponse = response
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;
    
    let models: Vec<String> = model_response
        .data
        .into_iter()
        .map(|m| m.id)
        .collect();
    
    Ok(models)
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