use serde::{Deserialize, Serialize};

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_MODEL: &str = "claude-3-5-sonnet-20240620";

#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<AnthropicMessage>,
}

#[derive(Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct AnthropicSuccessfulResponse {
    content: Vec<AnthropicContent>,
}

#[derive(Deserialize)]
struct AnthropicContent {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

#[derive(Deserialize)]
struct AnthropicError {
    message: String,
    #[serde(rename = "type")]
    error_type: String,
}

#[derive(Deserialize)]
struct AnthropicErrorRequest {
    error: AnthropicError,
    #[serde(rename = "type")]
    error_type: String,
}

pub async fn generate_commit_message(
    api_key: &str,
    prompt: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let anthropic_request = AnthropicRequest {
        model: ANTHROPIC_MODEL.to_string(),
        max_tokens: 1000,
        messages: vec![AnthropicMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
        }],
    };

    let response = client
        .post(ANTHROPIC_API_URL)
        .header("x-api-key", api_key)
        .header("anthropic-version", "2024-06-20")
        .header("content-type", "application/json")
        .json(&anthropic_request)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_response: AnthropicErrorRequest = response.json().await?;
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Anthropic API error: {}", error_response.error.message),
        )));
    }

    let successful_response: AnthropicSuccessfulResponse = response.json().await?;
    let content = successful_response.content.first().ok_or_else(|| {
        Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Anthropic API returned no content",
        ))
    })?;
    if content.content_type != "text" {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Anthropic API returned non-text content",
        )));
    }
    Ok(content.text.clone())
}
