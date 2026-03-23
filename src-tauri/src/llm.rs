use anyhow::Context;
use reqwest::Client;
use serde_json::json;

#[derive(Clone)]
pub struct LlmClient {
    http: Client,
    api_key: Option<String>,
    model: String,
}

impl LlmClient {
    pub fn new() -> Self {
        let api_key = std::env::var("OPENAI_API_KEY").ok();
        let model = std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string());
        Self {
            http: Client::new(),
            api_key,
            model,
        }
    }

    pub fn mode(&self) -> &'static str {
        if self.api_key.is_some() {
            "cloud"
        } else {
            "fallback"
        }
    }

    pub async fn chat(
        &self,
        persona_name: &str,
        tone: &str,
        memory_summary: &str,
        user_message: &str,
    ) -> anyhow::Result<String> {
        if let Some(api_key) = &self.api_key {
            let body = json!({
                "model": self.model,
                "messages": [
                    {
                        "role": "system",
                        "content": format!(
                            "You are {}. Tone: {}. Use concise Chinese unless user asks otherwise. Memory: {}",
                            persona_name, tone, memory_summary
                        )
                    },
                    {
                        "role": "user",
                        "content": user_message
                    }
                ],
                "temperature": 0.7
            });

            let res = self
                .http
                .post("https://api.openai.com/v1/chat/completions")
                .bearer_auth(api_key)
                .json(&body)
                .timeout(std::time::Duration::from_secs(30))
                .send()
                .await
                .context("llm request failed")?;

            if !res.status().is_success() {
                return Ok(Self::fallback_reply(user_message));
            }

            let val: serde_json::Value = res.json().await.context("llm parse failed")?;
            let reply = val
                .get("choices")
                .and_then(|v| v.get(0))
                .and_then(|v| v.get("message"))
                .and_then(|v| v.get("content"))
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| "我听到了，我们一步步来。")
                .to_string();
            return Ok(reply);
        }

        Ok(Self::fallback_reply(user_message))
    }

    fn fallback_reply(user_message: &str) -> String {
        if user_message.contains("待办") || user_message.contains("提醒") {
            "我记下了。你可以在待办面板里确认和调整时间。".to_string()
        } else if user_message.contains("你好") {
            "你好，我在。告诉我你现在最想推进的一件事。".to_string()
        } else {
            "收到。这个问题我可以继续帮你拆解成下一步行动。".to_string()
        }
    }
}
