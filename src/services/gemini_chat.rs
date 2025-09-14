use crate::{
    error::{AppError, Result},
    models::prompt::PromptsRequest,
};

// GEMINI_MODEL is provided via environment (.env). No hard-coded default here.

pub async fn create_answer(req: PromptsRequest) -> Result<String> {
    if let Err(issues) = req.validate() {
        return Err(AppError::Validation(issues.join(", ")));
    }

    // Combine system + user prompts. If you prefer structured multi-turn, adjust accordingly.
    let system = req.prompts.system.trim();
    let user = req.prompts.user.trim();
    let combined = if system.is_empty() {
        user.to_string()
    } else {
        format!("{system}\n\nUser: {user}")
    };

    // gemini-rs expects an API key via env var: GEMINI_API_KEY (as per crate docs).
    // We let missing key bubble up as External error with readable message.
    std::env::var("GEMINI_API_KEY").map_err(|_| AppError::Unauthorized)?; // treat missing key as Unauthorized

    // Fetch model name from environment
    let model = std::env::var("GEMINI_MODEL")
        .map_err(|_| AppError::Validation("GEMINI_MODEL env var not set".into()))?;

    // Perform the chat request
    let resp = gemini_rs::chat(&model)
        .send_message(&combined)
        .await
        .map_err(|e| AppError::External(e.to_string()))?;
    Ok(resp.to_string())
}
