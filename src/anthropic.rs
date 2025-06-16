use serde::{Deserialize, Serialize};

pub struct Client {
    http_client: reqwest::Client,
    api_key: String,
    api_base_url: String,
}

#[derive(Serialize)]
pub struct MessageParam {
    role: String,
    content: String,
}

#[derive(Serialize)]
pub struct MessageNewParams {
    model: String,
    max_tokens: u64,
    messages: Vec<MessageParam>,
}

#[derive(Deserialize)]
pub struct Message {
    pub content: Vec<MessageContent>,
}

#[derive(Deserialize)]
pub struct MessageContent {
    #[serde(rename = "type")]
    pub message_content_type: String,
    pub text: String,
}

#[derive(Deserialize)]
struct ErrorResponse {
    message: String,
    #[serde(rename = "type")]
    error_type: String,
}

#[derive(Deserialize)]
struct ErrorWrapper {
    error: ErrorResponse,
    #[serde(rename = "type")]
    response_type: String,
}

impl MessageNewParams {
    pub fn new(model: String, max_tokens: u64, messages: Vec<MessageParam>) -> Self {
        Self {
            model,
            max_tokens,
            messages,
        }
    }
}

impl MessageParam {
    pub fn new(content: String) -> Self {
        Self {
            role: "user".to_string(),
            content,
        }
    }
}

impl Client {
    pub fn new(api_key: String) -> Self {
        Self {
            http_client: reqwest::Client::new(),
            api_key,
            api_base_url: "https://api.anthropic.com".to_string(),
        }
    }

    pub async fn new_message(
        &self,
        message_new_params: MessageNewParams,
    ) -> Result<Message, Box<dyn std::error::Error>> {
        let api_url = format!("{}/v1/messages", self.api_base_url);
        let response = self
            .http_client
            .post(api_url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&message_new_params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: ErrorWrapper = response.json().await?;
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Anthropic API error: {}", error.error.message),
            )));
        }

        let message: Message = response.json().await?;
        Ok(message)
    }
}
