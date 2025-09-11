use serde::Deserialize;
use serde_json::json;
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

#[derive(Deserialize, Debug)]
struct PromptItem {
    id: u32,        // 1から始まる連番想定
    prompt: String, // プロンプト本文
}

#[derive(Deserialize, Debug)]
struct PromptsRequest {
    prompts: Vec<PromptItem>,
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    // Method check (POSTのみ)
    if req.method().as_str() != "POST" {
        return Ok(Response::builder()
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .header("Allow", "POST")
            .header("Content-Type", "application/json")
            .body(json!({"error": "Use POST with JSON body { 'prompts': [ { 'id': 1, 'prompt': '...'}, ... ] }"}).to_string().into())?);
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

    if parsed.prompts.is_empty() {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("Content-Type", "application/json")
            .body(
                json!({"error": "'prompts' must not be empty"})
                    .to_string()
                    .into(),
            )?);
    }

    // Validate sequential IDs starting from 1
    let mut invalid: Vec<String> = Vec::new();
    for (i, item) in parsed.prompts.iter().enumerate() {
        let expected = (i as u32) + 1;
        if item.id != expected {
            invalid.push(format!("expected id {} but got {}", expected, item.id));
        }
        if item.prompt.trim().is_empty() {
            invalid.push(format!("id {} has empty prompt", item.id));
        }
    }
    if !invalid.is_empty() {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("Content-Type", "application/json")
            .body(
                json!({
                    "error": "Validation failed",
                    "issues": invalid
                })
                .to_string()
                .into(),
            )?);
    }

    // ここで OpenAI API 呼び出しを行う想定 (未実装: TODO:)

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(json!({
            "count": parsed.prompts.len(),
            "prompts": parsed.prompts.iter().map(|p| json!({"id": p.id, "prompt": p.prompt})).collect::<Vec<_>>()
        }).to_string().into())?)
}
