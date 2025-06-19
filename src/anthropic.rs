use serde::{Deserialize, Serialize};

const ANTHROPIC_VERSION: &str = "2023-06-01";
const BASE_URL: &str = "https://api.anthropic.com";

#[async_trait::async_trait]
pub trait HttpClient {
    async fn post_json<T: Serialize + Send + Sync>(
        &self,
        url: &str,
        headers: Vec<(String, String)>,
        body: &T,
    ) -> Result<String, Box<dyn std::error::Error>>;
}

pub struct ReqwestHttpClient {
    client: reqwest::Client,
}

impl ReqwestHttpClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

impl Default for ReqwestHttpClient {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Client<H: HttpClient> {
    http_client: H,
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

impl Client<ReqwestHttpClient> {
    pub fn new_default(api_key: String) -> Self {
        Self::new(ReqwestHttpClient::new(), api_key)
    }
}

impl<H: HttpClient> Client<H> {
    pub fn new(http_client: H, api_key: String) -> Self {
        Self {
            http_client,
            api_key,
            api_base_url: BASE_URL.to_string(),
        }
    }

    pub async fn new_message(
        &self,
        message_new_params: MessageNewParams,
    ) -> Result<Message, Box<dyn std::error::Error>> {
        let api_url = format!("{}/v1/messages", self.api_base_url);
        let headers = vec![
            ("x-api-key".to_string(), self.api_key.to_string()),
            (
                "anthropic-version".to_string(),
                ANTHROPIC_VERSION.to_string(),
            ),
            ("content-type".to_string(), "application/json".to_string()),
        ];
        let response = self
            .http_client
            .post_json(&api_url, headers, &message_new_params)
            .await?;

        let message: Message = serde_json::from_str(&response)?;
        Ok(message)
    }
}

#[async_trait::async_trait]
impl HttpClient for ReqwestHttpClient {
    async fn post_json<T: Serialize + Send + Sync>(
        &self,
        url: &str,
        headers: Vec<(String, String)>,
        body: &T,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut header_map = reqwest::header::HeaderMap::new();
        for (key, value) in headers {
            header_map.insert(
                reqwest::header::HeaderName::from_bytes(key.as_bytes())?,
                reqwest::header::HeaderValue::from_str(&value)?,
            );
        }

        let response = self
            .client
            .post(url)
            .headers(header_map)
            .json(body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: ErrorWrapper = response.json().await?;
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Anthropic API error: {}", error.error.message),
            )));
        }

        let response_text = response.text().await?;
        Ok(response_text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use serde_json::json;

    struct MockHttpClient {
        expected_url: String,
        expected_headers: Vec<(String, String)>,
        response: String,
        should_fail: bool,
    }

    #[async_trait]
    impl HttpClient for MockHttpClient {
        async fn post_json<T: Serialize + Send + Sync>(
            &self,
            url: &str,
            headers: Vec<(String, String)>,
            body: &T,
        ) -> Result<String, Box<dyn std::error::Error>> {
            assert_eq!(url, self.expected_url);

            for (key, value) in &self.expected_headers {
                assert!(
                    headers.iter().any(|(k, v)| k == key && v == value),
                    "Missing header: {:?}={:?}",
                    key,
                    value
                )
            }

            let body_json = serde_json::to_value(body)?;
            assert!(body_json.get("model").is_some());
            assert!(body_json.get("messages").is_some());

            if self.should_fail {
                Err("Mock error".into())
            } else {
                Ok(self.response.clone())
            }
        }
    }

    #[tokio::test]
    async fn test_new_message_success() {
        let mock_response = json!({
            "content": [{
                "type": "text",
                "text": "feat: Add new authentication system"
            }]
        })
        .to_string();

        let mock_client = MockHttpClient {
            expected_url: "https://api.anthropic.com/v1/messages".to_string(),
            expected_headers: vec![
                ("x-api-key".to_string(), "test_key".to_string()),
                ("anthropic-version".to_string(), "2023-06-01".to_string()),
            ],
            response: mock_response,
            should_fail: false,
        };

        let client = Client::new(mock_client, "test_key".to_string());

        let params = MessageNewParams::new(
            "claude-3-5-sonnet-20240620".to_string(),
            1024,
            vec![MessageParam::new("Test prompt".to_string())],
        );

        let result = client.new_message(params).await;
        assert!(result.is_ok());

        let message = result.unwrap();
        assert_eq!(message.content.len(), 1);
        assert_eq!(
            message.content[0].text,
            "feat: Add new authentication system"
        );
    }

    #[tokio::test]
    async fn test_new_message_empty_response() {
        let mock_response = json!({
            "content": []
        })
        .to_string();

        let mock_client = MockHttpClient {
            expected_url: "https://api.anthropic.com/v1/messages".to_string(),
            expected_headers: vec![],
            response: mock_response,
            should_fail: false,
        };

        let client = Client::new(mock_client, "test_key".to_string());
        let params = MessageNewParams::new(
            "claude-3-5-sonnet-20240620".to_string(),
            1024,
            vec![MessageParam::new("Test prompt".to_string())],
        );

        let result = client.new_message(params).await;
        assert!(result.is_ok());

        let message = result.unwrap();
        assert_eq!(message.content.len(), 0);
    }

    #[tokio::test]
    async fn test_new_message_http_error() {
        let mock_client = MockHttpClient {
            expected_url: "https://api.anthropic.com/v1/messages".to_string(),
            expected_headers: vec![],
            response: String::new(),
            should_fail: true,
        };

        let client = Client::new(mock_client, "test_key".to_string());
        let params = MessageNewParams::new(
            "claude-3-5-sonnet-20240620".to_string(),
            1024,
            vec![MessageParam::new("Test prompt".to_string())],
        );

        let result = client.new_message(params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_new_message_invalid_json_response() {
        let mock_client = MockHttpClient {
            expected_url: "https://api.anthropic.com/v1/messages".to_string(),
            expected_headers: vec![],
            response: "invalid json".to_string(),
            should_fail: false,
        };

        let client = Client::new(mock_client, "test_key".to_string());
        let params = MessageNewParams::new(
            "claude-3-5-sonnet-20240620".to_string(),
            1024,
            vec![MessageParam::new("Test prompt".to_string())],
        );

        let result = client.new_message(params).await;
        assert!(result.is_err());
        assert!(
            result
                .err()
                .unwrap()
                .to_string()
                .contains("expected value at line")
        );
    }
}
