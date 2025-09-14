use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct PromptItem {
    pub system: String,
    pub user: String,
}

#[derive(Deserialize, Debug)]
pub struct PromptsRequest {
    pub prompts: PromptItem,
}

impl PromptsRequest {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut issues = Vec::new();
        if self.prompts.system.trim().is_empty() { issues.push("prompts.system is empty".into()); }
        if self.prompts.user.trim().is_empty() { issues.push("prompts.user is empty".into()); }
        if issues.is_empty() { Ok(()) } else { Err(issues) }
    }
}
