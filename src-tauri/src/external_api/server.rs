//! tiny_http listener implementing the S0 route table.

use std::io::Read;
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Instant;

use serde::Serialize;
use tiny_http::{Header, Method, Response, Server, StatusCode};

use super::policy::{authorize, reject_origin, AuthError, RateLimiter};

pub const DEFAULT_PORT: u16 = 61742;
pub const TRANSLATE_BODY_LIMIT: usize = 1024 * 1024;
pub const OCR_BODY_LIMIT: usize = 24 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ExternalApiStatus {
    Disabled,
    Enabled { port: u16 },
    PortInUse { configured_port: u16 },
}

pub struct ApiHooks {
    pub health_version: String,
    pub providers: Box<dyn Fn() -> Result<serde_json::Value, String> + Send + Sync>,
    pub translate: Box<dyn Fn(serde_json::Value) -> Result<serde_json::Value, String> + Send + Sync>,
    pub ocr: Box<dyn Fn(&[u8]) -> Result<serde_json::Value, String> + Send + Sync>,
    pub selection: Box<dyn Fn() -> Result<(), String> + Send + Sync>,
    pub show_input: Box<dyn Fn() -> Result<(), String> + Send + Sync>,
}

impl ApiHooks {
    pub fn noop() -> Self {
        Self {
            health_version: env!("CARGO_PKG_VERSION").into(),
            providers: Box::new(|| Ok(serde_json::json!([]))),
            translate: Box::new(|_| Ok(serde_json::json!({"text": ""}))),
            ocr: Box::new(|_| Ok(serde_json::json!({"text":"","confidence":0.0}))),
            selection: Box::new(|| Ok(())),
            show_input: Box::new(|| Ok(())),
        }
    }
}

struct Inner {
    token: Mutex<String>,
    limiter: Mutex<RateLimiter>,
    hooks: ApiHooks,
    stop: AtomicBool,
}

pub struct ExternalApiHandle {
    inner: Arc<Inner>,
    pub port: u16,
    join: Option<JoinHandle<()>>,
}

impl ExternalApiHandle {
    pub fn set_token(&self, token: String) {
        *self.inner.token.lock().unwrap() = token;
    }

    pub fn stop(&mut self) {
        self.inner.stop.store(true, Ordering::SeqCst);
        let _ = std::net::TcpStream::connect(("127.0.0.1", self.port));
        if let Some(j) = self.join.take() {
            let _ = j.join();
        }
    }
}

impl Drop for ExternalApiHandle {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Bind `127.0.0.1:port`. Port `0` asks the OS for an ephemeral port.
pub fn start_listener(
    port: u16,
    token: String,
    hooks: ApiHooks,
) -> Result<ExternalApiHandle, String> {
    let listener = TcpListener::bind(("127.0.0.1", port)).map_err(|e| e.to_string())?;
    let bound = listener.local_addr().map_err(|e| e.to_string())?.port();
    let server = Server::from_listener(listener, None).map_err(|e| e.to_string())?;
    let inner = Arc::new(Inner {
        token: Mutex::new(token),
        limiter: Mutex::new(RateLimiter::new(super::policy::DEFAULT_RATE)),
        hooks,
        stop: AtomicBool::new(false),
    });
    let worker = inner.clone();
    let join = std::thread::Builder::new()
        .name("linguaray-external-api".into())
        .spawn(move || serve_loop(server, worker))
        .map_err(|e| e.to_string())?;
    Ok(ExternalApiHandle {
        inner,
        port: bound,
        join: Some(join),
    })
}

fn serve_loop(server: Server, inner: Arc<Inner>) {
    loop {
        if inner.stop.load(Ordering::SeqCst) {
            break;
        }
        let request = match server.recv_timeout(std::time::Duration::from_millis(250)) {
            Ok(Some(r)) => r,
            Ok(None) => continue,
            Err(_) => continue,
        };
        if inner.stop.load(Ordering::SeqCst) {
            let _ = request.respond(Response::from_string("").with_status_code(StatusCode(503)));
            break;
        }
        if let Err(e) = handle_one(request, &inner) {
            log::warn!("external api respond failed: {e}");
        }
    }
}

fn header_value(request: &tiny_http::Request, name: &str) -> Option<String> {
    let want = name.to_ascii_lowercase();
    request.headers().iter().find_map(|h| {
        if h.field.to_string().eq_ignore_ascii_case(&want) {
            Some(h.value.as_str().to_string())
        } else {
            None
        }
    })
}

fn json_response(status: u16, body: serde_json::Value) -> Response<std::io::Cursor<Vec<u8>>> {
    let bytes = serde_json::to_vec(&body).unwrap_or_else(|_| b"{}".to_vec());
    Response::from_data(bytes)
        .with_status_code(StatusCode(status))
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap())
}

fn handle_one(mut request: tiny_http::Request, inner: &Inner) -> Result<(), std::io::Error> {
    if reject_origin(header_value(&request, "Origin").as_deref()) {
        return request.respond(json_response(
            403,
            serde_json::json!({"error":"origin rejected"}),
        ));
    }
    let expected = inner.token.lock().unwrap().clone();
    match authorize(header_value(&request, "Authorization").as_deref(), &expected) {
        Ok(()) => {}
        Err(AuthError::Missing) | Err(AuthError::Mismatch) => {
            return request.respond(json_response(
                401,
                serde_json::json!({"error":"unauthorized"}),
            ));
        }
    }
    if !inner.limiter.lock().unwrap().allow(Instant::now()) {
        return request.respond(json_response(429, serde_json::json!({"error":"rate limited"})));
    }

    let url = request.url().to_string();
    let path = url.split('?').next().unwrap_or(&url).to_string();
    let method = request.method().clone();

    let response = match (method, path.as_str()) {
        (Method::Get, "/v1/health") => json_response(
            200,
            serde_json::json!({
                "status": "ok",
                "version": inner.hooks.health_version,
            }),
        ),
        (Method::Get, "/v1/providers") => match (inner.hooks.providers)() {
            Ok(v) => json_response(200, v),
            Err(e) => json_response(500, serde_json::json!({"error": e})),
        },
        (Method::Get, "/openapi.json") => json_response(200, openapi_doc()),
        (Method::Post, "/v1/translate") => {
            match read_body(&mut request, TRANSLATE_BODY_LIMIT).and_then(|b| {
                let v: serde_json::Value =
                    serde_json::from_slice(&b).map_err(|e| e.to_string())?;
                (inner.hooks.translate)(v)
            }) {
                Ok(v) => json_response(200, v),
                Err(e) => json_response(400, serde_json::json!({"error": e})),
            }
        }
        (Method::Post, "/v1/ocr") => {
            match read_body(&mut request, OCR_BODY_LIMIT).and_then(|b| {
                let v: serde_json::Value =
                    serde_json::from_slice(&b).map_err(|e| e.to_string())?;
                let b64 = v
                    .get("image")
                    .and_then(|x| x.as_str())
                    .ok_or_else(|| "missing image".to_string())?;
                use base64::Engine;
                let bytes = base64::engine::general_purpose::STANDARD
                    .decode(b64)
                    .or_else(|_| base64::engine::general_purpose::STANDARD_NO_PAD.decode(b64))
                    .map_err(|e| e.to_string())?;
                (inner.hooks.ocr)(&bytes)
            }) {
                Ok(v) => json_response(200, v),
                Err(e) => json_response(400, serde_json::json!({"error": e})),
            }
        }
        (Method::Post, "/v1/selection") => match (inner.hooks.selection)() {
            Ok(()) => json_response(200, serde_json::json!({"ok": true})),
            Err(e) => json_response(500, serde_json::json!({"error": e})),
        },
        (Method::Post, "/v1/show-input") => match (inner.hooks.show_input)() {
            Ok(()) => json_response(200, serde_json::json!({"ok": true})),
            Err(e) => json_response(500, serde_json::json!({"error": e})),
        },
        _ => json_response(404, serde_json::json!({"error":"not found"})),
    };
    request.respond(response)
}

fn read_body(request: &mut tiny_http::Request, limit: usize) -> Result<Vec<u8>, String> {
    let len = request.body_length().unwrap_or(0);
    if len > limit {
        return Err(format!("body exceeds {limit} bytes"));
    }
    let mut buf = Vec::new();
    request
        .as_reader()
        .take(limit as u64 + 1)
        .read_to_end(&mut buf)
        .map_err(|e| e.to_string())?;
    if buf.len() > limit {
        return Err(format!("body exceeds {limit} bytes"));
    }
    Ok(buf)
}

fn openapi_doc() -> serde_json::Value {
    serde_json::json!({
        "openapi": "3.1.0",
        "info": { "title": "LinguaRay External API", "version": env!("CARGO_PKG_VERSION") },
        "paths": {
            "/v1/health": { "get": { "operationId": "health" } },
            "/v1/providers": { "get": { "operationId": "providers" } },
            "/v1/translate": { "post": { "operationId": "translate" } },
            "/v1/ocr": { "post": { "operationId": "ocr" } },
            "/v1/selection": { "post": { "operationId": "selection" } },
            "/v1/show-input": { "post": { "operationId": "showInput" } },
            "/openapi.json": { "get": { "operationId": "openapi" } }
        }
    })
}
