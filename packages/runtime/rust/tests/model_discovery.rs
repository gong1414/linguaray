use linguaray_runtime::Runtime;
use std::{
    collections::HashMap,
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

fn server(responses: Vec<(u16, &'static str)>) -> (String, mpsc::Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        for (status, body) in responses {
            let (mut socket, _) = listener.accept().unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut data = Vec::new();
            let mut buf = [0; 4096];
            loop {
                let n = socket.read(&mut buf).unwrap();
                if n == 0 {
                    return;
                }
                data.extend_from_slice(&buf[..n]);
                if let Some(i) = data.windows(4).position(|x| x == b"\r\n\r\n") {
                    let headers = String::from_utf8_lossy(&data[..i]);
                    let length = headers
                        .lines()
                        .find_map(|line| {
                            line.to_lowercase()
                                .strip_prefix("content-length:")
                                .and_then(|v| v.trim().parse::<usize>().ok())
                        })
                        .unwrap_or(0);
                    if data.len() >= i + 4 + length {
                        break;
                    }
                }
            }
            tx.send(String::from_utf8(data).unwrap()).unwrap();
            write!(socket, "HTTP/1.1 {status} Response\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
        }
    });
    (format!("http://{address}"), rx)
}

#[test]
fn draft_discovery_follows_endpoint_auth_pagination_and_never_persists() {
    let (base, requests) = server(vec![
        (
            200,
            r#"{"data":[{"id":"vendor/one"}],"has_more":true,"last_id":"vendor/one"}"#,
        ),
        (200, r#"{"data":[{"id":"vendor/two"}],"has_more":false}"#),
    ]);
    let dir = std::env::temp_dir().join(format!(
        "linguaray-model-discovery-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let runtime = Runtime::new(dir.to_string_lossy().into_owned()).unwrap();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        let settings = runtime.clone().settings();
        let before = settings.get_json().await.unwrap();
        let models = settings
            .discover_provider_models(
                "draft".into(),
                "anthropic".into(),
                HashMap::from([
                    ("apiKey".into(), "fixture-secret".into()),
                    ("baseUrl".into(), base.clone()),
                    ("modelsUrl".into(), format!("{base}/custom/models")),
                    ("defaultModel".into(), "discovery".into()),
                ]),
            )
            .await
            .unwrap();
        assert_eq!(models, ["vendor/one", "vendor/two"]);
        assert_eq!(settings.get_json().await.unwrap(), before);
        assert!(settings
            .get_provider("draft".into())
            .await
            .unwrap()
            .is_none());
    });
    let first = requests.recv_timeout(Duration::from_secs(1)).unwrap();
    let second = requests.recv_timeout(Duration::from_secs(1)).unwrap();
    assert!(first.starts_with("GET /custom/models?limit=1000 "));
    assert!(first.to_lowercase().contains("x-api-key: fixture-secret"));
    assert!(second.contains("after_id=vendor%2Fone"));
    drop(runtime);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn connection_test_probes_selected_model_even_when_model_listing_is_unavailable() {
    let (base, requests) = server(vec![(
        200,
        r#"{"id":"test","model":"vendor/custom","choices":[{"index":0,"message":{"role":"assistant","content":"Bonjour"},"finish_reason":"stop"}]}"#,
    )]);
    let dir = std::env::temp_dir().join(format!(
        "linguaray-model-test-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let runtime = Runtime::new(dir.to_string_lossy().into_owned()).unwrap();
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            assert_eq!(
                runtime
                    .clone()
                    .settings()
                    .test_provider(
                        "draft".into(),
                        "openai_compatible".into(),
                        HashMap::from([
                            ("baseUrl".into(), format!("{base}/v1")),
                            ("defaultModel".into(), "vendor/custom".into()),
                        ])
                    )
                    .await
                    .unwrap(),
                0
            );
        });
    let request = requests.recv_timeout(Duration::from_secs(1)).unwrap();
    assert!(request.starts_with("POST /v1/chat/completions "));
    assert!(request.contains("\"model\":\"vendor/custom\""));
    drop(runtime);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn auth_failure_is_not_a_model_list_and_secret_is_redacted() {
    let (base, _requests) = server(vec![(
        401,
        r#"{"error":{"message":"fixture-secret denied"}}"#,
    )]);
    let provider = linguaray_engine::OpenAiCompatibleProvider::new(
        &linguaray_engine::specs::OPENAI_COMPATIBLE,
        linguaray_engine::OpenAiCompatibleProviderConfig {
            api_key: "fixture-secret".into(),
            base_url: Some(base),
            default_model: "x".into(),
            models_url: None,
        },
    )
    .unwrap();
    use linguaray_engine::Provider;
    let error = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(provider.list_models())
        .unwrap_err()
        .to_string();
    assert!(error.contains("401"));
    assert!(!error.contains("fixture-secret"));
}
