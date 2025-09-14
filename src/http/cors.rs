use vercel_runtime::{Body, Response};

pub fn add_cors(mut resp: Response<Body>) -> Response<Body> {
    let h = resp.headers_mut();
    h.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
    h.insert("Access-Control-Allow-Methods", "POST,OPTIONS".parse().unwrap());
    h.insert("Access-Control-Allow-Headers", "Content-Type,x-api-key".parse().unwrap());
    h.insert("Access-Control-Max-Age", "86400".parse().unwrap());
    resp
}
