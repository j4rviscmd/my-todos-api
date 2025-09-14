use openai::chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole};
use openai::Credentials;

use crate::error::{AppError, Result};

pub struct OpenAiClient {
    creds: Credentials,
    model: String,
}

impl OpenAiClient {
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("OPENAI_KEY").or_else(|_| std::env::var("OPENAI_API_KEY"))
            .map_err(|_| AppError::External("Missing OPENAI_KEY".into()))?;
        let base = std::env::var("OPENAI_BASE_URL").map_err(|_| AppError::External("Missing OPENAI_BASE_URL".into()))?;
        let model = std::env::var("OPENAI_MODEL").map_err(|_| AppError::External("Missing OPENAI_MODEL".into()))?;
        Ok(Self { creds: Credentials::new(api_key, base), model })
    }

    pub async fn complete(&self, system: String, user: String) -> Result<String> {
        debug_log_preview(&system, &user);
        let messages = vec![
            ChatCompletionMessage {
                role: ChatCompletionMessageRole::System,
                content: Some(system),
                name: None,
                function_call: None,
                tool_calls: None,
                tool_call_id: None,
            },
            ChatCompletionMessage {
                role: ChatCompletionMessageRole::User,
                content: Some(user),
                name: None,
                function_call: None,
                tool_calls: None,
                tool_call_id: None,
            },
        ];

        let completion_res = ChatCompletion::builder(&self.model, messages)
            .temperature(0.0)
            .credentials(self.creds.clone())
            .create()
            .await;

        match completion_res {
            Ok(resp) => {
                let assistant = resp.choices.first()
                    .and_then(|c| c.message.content.as_ref())
                    .map(|c| c.to_string())
                    .unwrap_or_default();
                Ok(assistant)
            }
            Err(e) => Err(AppError::External(e.to_string()))
        }
    }
}

fn debug_log_preview(system: &str, user: &str) {
    if std::env::var("DEBUG_OPENAI_RAW").ok().as_deref() == Some("1") {
        eprintln!("[debug] system.len={} user.len={}", system.len(), user.len());
        eprintln!("[debug] system.preview={}", system.chars().take(80).collect::<String>());
        eprintln!("[debug] user.preview={}", user.chars().take(80).collect::<String>());
    }
}
