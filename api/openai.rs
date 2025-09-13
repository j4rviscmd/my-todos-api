use serde::Deserialize;
use serde_json::json;
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

#[derive(Deserialize, Debug)]
struct PromptItem {
    system: String, // システム指示
    user: String,   // ユーザー入力
}

#[derive(Deserialize, Debug)]
struct PromptsRequest {
    prompts: PromptItem, // 配列不要: 単一オブジェクト
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    // Method check (POSTのみ)
    if req.method().as_str() != "POST" {
        return Ok(Response::builder()
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .header("Allow", "POST")
            .header("Content-Type", "application/json")
    .body(json!({"error": "Use POST with JSON body { 'prompts': { 'system': '...', 'user': '...' } }"}).to_string().into())?);
    }

    // Header.x-api-key チェック
    let expected_api_key = std::env::var("X_API_KEY").ok();
    if let Some(expected) = expected_api_key.as_ref() {
        // 環境変数が設定されている場合のみ検証 (未設定ならスキップ)
        let provided = req.headers().get("x-api-key").and_then(|h| h.to_str().ok());
        match provided {
            Some(got) if got == expected => { /* OK */ }
            _ => {
                return Ok(Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .header("Content-Type", "application/json")
                    .body(json!({"error": "Unauthorized"}).to_string().into())?);
            }
        }
    }

    let body_bytes = req.body();
    if body_bytes.is_empty() {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("Content-Type", "application/json")
            .body(json!({"error": "Empty body"}).to_string().into())?);
    }

    let parsed: PromptsRequest = match serde_json::from_slice(body_bytes) {
        Ok(v) => v,
        Err(e) => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header("Content-Type", "application/json")
                .body(
                    json!({"error": "Invalid JSON", "detail": e.to_string()})
                        .to_string()
                        .into(),
                )?);
        }
    };

    // Validate single prompt item
    let mut issues: Vec<String> = Vec::new();
    if parsed.prompts.system.trim().is_empty() {
        issues.push("prompts.system is empty".to_string());
    }
    if parsed.prompts.user.trim().is_empty() {
        issues.push("prompts.user is empty".to_string());
    }
    if !issues.is_empty() {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("Content-Type", "application/json")
            .body(
                json!({
                    "error": "Validation failed",
                    "issues": issues
                })
                .to_string()
                .into(),
            )?);
    }

    // ここで OpenAI API 呼び出しを行う想定 (未実装: TODO:)

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(
            json!({
                "prompts": {
                    "system": parsed.prompts.system,
                    "user": parsed.prompts.user
                }
            })
            .to_string()
            .into(),
        )?)
}
