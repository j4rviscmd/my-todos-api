use my_vercel_api::{
    http::{
        cors::add_cors,
        response::{error_response, json_response},
    },
    models::prompt::PromptsRequest,
    services::gemini_chat::create_answer,
};
use serde_json::json;
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    // Preflight
    if req.method().as_str() == "OPTIONS" {
        return Ok(add_cors(
            Response::builder()
                .status(StatusCode::NO_CONTENT)
                .body(Body::Empty)?,
        ));
    }

    if req.method().as_str() != "POST" {
        return Ok(add_cors(json_response(
            StatusCode::METHOD_NOT_ALLOWED,
            &json!({"error":"Use POST with JSON body { 'prompts': { 'system': '...', 'user': '...' } }"}),
        )?));
    }

    // API Key ヘッダ検証 (任意)
    if let Ok(expected) = std::env::var("X_API_KEY") {
        let provided = req.headers().get("x-api-key").and_then(|h| h.to_str().ok());
        if provided != Some(expected.as_str()) {
            return Ok(add_cors(json_response(
                StatusCode::UNAUTHORIZED,
                &json!({"error":"Unauthorized"}),
            )?));
        }
    }

    let body = req.body();
    if body.is_empty() {
        return Ok(add_cors(json_response(
            StatusCode::BAD_REQUEST,
            &json!({"error":"Empty body"}),
        )?));
    }

    let parsed: PromptsRequest = match serde_json::from_slice(body) {
        Ok(v) => v,
        Err(e) => {
            return Ok(add_cors(json_response(
                StatusCode::BAD_REQUEST,
                &json!({"error":"Invalid JSON","detail":e.to_string()}),
            )?));
        }
    };

    match create_answer(parsed).await {
        Ok(answer) => Ok(add_cors(json_response(
            StatusCode::OK,
            &json!({"answer": answer.replace("\n", "")}),
        )?)),
        Err(e) => {
            let (status, val) = error_response(&e);
            Ok(add_cors(json_response(status, &val)?))
        }
    }
}
