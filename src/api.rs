use crate::anthropic;

const ANTHROPIC_MODEL: &str = "claude-3-5-sonnet-20240620";

pub async fn generate_commit_message(
    api_key: &str,
    prompt: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = anthropic::Client::new(api_key.to_string());
    let message = anthropic::MessageParam::new(prompt.to_string());
    let message_new_params =
        anthropic::MessageNewParams::new(ANTHROPIC_MODEL.to_string(), 1000, vec![message]);
    let message = client.new_message(message_new_params).await?;
    Ok(message.content.first().unwrap().text.clone())
}
