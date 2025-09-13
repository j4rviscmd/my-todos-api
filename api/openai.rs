use openai::chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole};
use openai::Credentials;
use serde::Deserialize;
use serde_json::json;
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};

fn corsify(mut resp: Response<Body>) -> Response<Body> {
    let headers = resp.headers_mut();
    headers.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
    headers.insert(
        "Access-Control-Allow-Methods",
        "POST,OPTIONS".parse().unwrap(),
    );
    headers.insert(
        "Access-Control-Allow-Headers",
        "Content-Type,x-api-key".parse().unwrap(),
    );
    headers.insert("Access-Control-Max-Age", "86400".parse().unwrap());
    resp
}

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
    // Preflight (OPTIONS)
    if req.method().as_str() == "OPTIONS" {
        let resp = Response::builder()
            .status(StatusCode::NO_CONTENT)
            .body(Body::Empty)?;
        return Ok(corsify(resp));
    }

    // Method check (POSTのみ)
    if req.method().as_str() != "POST" {
        let resp = Response::builder()
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .header("Allow", "POST,OPTIONS")
            .header("Content-Type", "application/json")
            .body(json!({"error": "Use POST with JSON body { 'prompts': { 'system': '...', 'user': '...' } }"}).to_string().into())?;
        return Ok(corsify(resp));
    }

    // Header.x-api-key チェック
    let expected_api_key = std::env::var("X_API_KEY").ok();
    if let Some(expected) = expected_api_key.as_ref() {
        // 環境変数が設定されている場合のみ検証 (未設定ならスキップ)
        let provided = req.headers().get("x-api-key").and_then(|h| h.to_str().ok());
        match provided {
            Some(got) if got == expected => { /* OK */ }
            _ => {
                let resp = Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .header("Content-Type", "application/json")
                    .body(json!({"error": "Unauthorized"}).to_string().into())?;
                return Ok(corsify(resp));
            }
        }
    }

    let body_bytes = req.body();
    if body_bytes.is_empty() {
        let resp = Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("Content-Type", "application/json")
            .body(json!({"error": "Empty body"}).to_string().into())?;
        return Ok(corsify(resp));
    }

    let parsed: PromptsRequest = match serde_json::from_slice(body_bytes) {
        Ok(v) => v,
        Err(e) => {
            let resp = Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header("Content-Type", "application/json")
                .body(
                    json!({"error": "Invalid JSON", "detail": e.to_string()})
                        .to_string()
                        .into(),
                )?;
            return Ok(corsify(resp));
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
        let resp = Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("Content-Type", "application/json")
            .body(
                json!({ "error": "Validation failed", "issues": issues })
                    .to_string()
                    .into(),
            )?;
        return Ok(corsify(resp));
    }

    // OpenAI (GitHub Models) 呼び出し
    // crate は OPENAI_KEY / OPENAI_BASE_URL を参照するので名称に注意。
    let api_key = std::env::var("OPENAI_KEY")
        .ok()
        .filter(|v| !v.trim().is_empty());
    let api_key = match api_key {
        Some(v) => v,
        None => {
            let resp = Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header("Content-Type", "application/json")
                .body(
                    json!({"error": "Missing OPENAI_KEY (or OPENAI_API_KEY)"})
                        .to_string()
                        .into(),
                )?;
            return Ok(corsify(resp));
        }
    };

    let base_url = std::env::var("OPENAI_BASE_URL").unwrap();

    // Credentials を直接生成 (from_env は unwrap で panic するため使わない)
    let creds = Credentials::new(api_key, base_url);

    let system_content = parsed.prompts.system.clone();
    let user_content = parsed.prompts.user.clone();

    let messages = vec![
        ChatCompletionMessage {
            role: ChatCompletionMessageRole::System,
            content: Some(system_content),
            name: None,
            function_call: None,
            tool_calls: None,
            tool_call_id: None,
        },
        ChatCompletionMessage {
            role: ChatCompletionMessageRole::User,
            content: Some(user_content),
            name: None,
            function_call: None,
            tool_calls: None,
            tool_call_id: None,
        },
    ];
    let model = std::env::var("OPENAI_MODEL").unwrap();

    let completion_res = ChatCompletion::builder(&model, messages)
        .temperature(0.0)
        .credentials(creds)
        .create()
        .await;

    match completion_res {
        Ok(resp) => {
            // 最初のアシスタントメッセージを取り出す
            let assistant = resp
                .choices
                .first()
                .and_then(|c| c.message.content.as_ref())
                .map(|c| c.to_string())
                .unwrap_or_default();

            let resp = Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(json!({ "answer": assistant }).to_string().into())?;
            Ok(corsify(resp))
        }
        Err(e) => {
            let resp = Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .header("Content-Type", "application/json")
                .body(
                    json!({"error": "Upstream OpenAI error", "detail": e.to_string()})
                        .to_string()
                        .into(),
                )?;
            Ok(corsify(resp))
        }
    }
}
