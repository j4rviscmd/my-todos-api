use serde_json::{Value, json};
use vercel_runtime::{Body, Response, StatusCode};

use crate::error::AppError;

pub fn error_response(err: &AppError) -> (StatusCode, Value) {
    match err {
        AppError::Validation(msg) => (StatusCode::BAD_REQUEST, json!({"error":"Validation","detail":msg})),
        AppError::Unauthorized => (StatusCode::UNAUTHORIZED, json!({"error":"Unauthorized"})),
        AppError::External(msg) => (StatusCode::BAD_GATEWAY, json!({"error":"Upstream","detail":msg})),
        AppError::Other(e) => (StatusCode::INTERNAL_SERVER_ERROR, json!({"error":"Internal","detail":e.to_string()})),
    }
}

pub fn json_response<T: serde::Serialize>(status: StatusCode, value: &T) -> anyhow::Result<Response<Body>> {
    Ok(Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(value)?.into())?)
}
