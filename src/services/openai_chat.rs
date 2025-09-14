use crate::{clients::openai::OpenAiClient, error::{Result, AppError}, models::prompt::PromptsRequest};

pub async fn create_answer(req: PromptsRequest) -> Result<String> {
    if let Err(issues) = req.validate() {
        return Err(AppError::Validation(issues.join(", ")));
    }
    let client = OpenAiClient::from_env()?;
    let system = req.prompts.system;
    let user = req.prompts.user;
    client.complete(system, user).await
}
