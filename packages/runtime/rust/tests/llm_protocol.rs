use linguaray_core::{ChatMessage, ChatRequest};
use linguaray_engine::{specs, OpenAiCompatibleProvider, OpenAiCompatibleProviderConfig, Provider};
use linguaray_runtime::{Runtime, StreamCallback};
use std::{
    collections::HashMap,
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

fn server(parts: Vec<Vec<u8>>, delay: Duration) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    thread::spawn(move || {
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
        socket.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n").unwrap();
        for part in parts {
            write!(socket, "{:x}\r\n", part.len()).unwrap();
            socket.write_all(&part).unwrap();
            socket.write_all(b"\r\n").unwrap();
            socket.flush().unwrap();
            thread::sleep(delay);
        }
        let _ = socket.write_all(b"0\r\n\r\n");
    });
    format!("http://{address}/v1")
}

fn request() -> ChatRequest {
    ChatRequest {
        model: "audit-model".into(),
        messages: vec![ChatMessage::user("Hello")],
        temperature: None,
        max_tokens: None,
        stream: Some(true),
        response_format: None,
    }
}

#[test]
fn utf8_character_must_survive_transport_chunk_boundary() {
    let payload = "data: {\"choices\":[{\"index\":0,\"delta\":{\"content\":\"你好\"},\"finish_reason\":null}]}\n\ndata: [DONE]\n\n".as_bytes();
    let split = payload
        .windows(3)
        .position(|x| x == "你".as_bytes())
        .unwrap()
        + 1;
    let base = server(
        vec![payload[..split].to_vec(), payload[split..].to_vec()],
        Duration::from_millis(200),
    );
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();
    let provider = OpenAiCompatibleProvider::new(
        &specs::OPENAI_COMPATIBLE,
        OpenAiCompatibleProviderConfig {
            api_key: String::new(),
            base_url: Some(base),
            default_model: "audit-model".into(),
            models_url: None,
        },
    )
    .unwrap();
    let receiver = rt
        .block_on(provider.llm().unwrap().chat_stream(request()))
        .unwrap();
    let mut text = String::new();
    while let Ok(chunk) = receiver.rx.recv_timeout(Duration::from_secs(3)) {
        text.push_str(&chunk.content);
        if chunk.finish_reason.is_some() {
            break;
        }
    }
    assert_eq!(
        text, "你好",
        "transport chunks must be buffered as bytes before UTF-8 decoding"
    );
}

struct Callback(mpsc::Sender<String>);
impl StreamCallback for Callback {
    fn on_chunk(&self, content: String) {
        let _ = self.0.send(format!("chunk:{content}"));
    }
    fn on_finish(&self, reason: String) {
        let _ = self.0.send(format!("finish:{reason}"));
    }
    fn on_error(&self, error: String) {
        let _ = self.0.send(format!("error:{error}"));
    }
}

#[test]
fn runtime_stream_must_deliver_a_successful_local_response() {
    let payload = b"data: {\"choices\":[{\"index\":0,\"delta\":{\"content\":\"hello\"},\"finish_reason\":\"stop\"}]}\n\n";
    let base = server(vec![payload.to_vec()], Duration::from_millis(5));
    let dir = std::env::temp_dir().join(format!(
        "linguaray-audit-stream-{}",
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
    rt.block_on(runtime.clone().settings().update_provider(
        "audit-stream".into(),
        "openai_compatible".into(),
        HashMap::from([
            ("baseUrl".into(), base),
            ("defaultModel".into(), "audit-model".into()),
        ]),
        None,
    ))
    .unwrap();
    let (tx, rx) = mpsc::channel();
    runtime
        .clone()
        .llm("audit-stream".into())
        .unwrap()
        .translate_stream(
            "en".into(),
            "zh-Hans".into(),
            "Hello".into(),
            Box::new(Callback(tx)),
        );
    let first = rx.recv_timeout(Duration::from_secs(3));
    assert_eq!(
        first,
        Ok("chunk:hello".to_owned()),
        "a completed localhost response must reach the FFI callback"
    );
    assert_eq!(
        rx.recv_timeout(Duration::from_secs(3)),
        Ok("finish:stop".to_owned())
    );
    drop(runtime);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn cancelling_translation_closes_the_upstream_socket_even_without_more_chunks() {
    for headers_sent in [false, true] {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}/v1", listener.local_addr().unwrap());
        let (ready_tx, ready_rx) = mpsc::channel();
        let (closed_tx, closed_rx) = mpsc::channel();
        let server = thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            let mut data = Vec::new();
            let mut buf = [0; 4096];
            loop {
                let n = socket.read(&mut buf).unwrap();
                assert_ne!(n, 0);
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
            if headers_sent {
                socket.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n").unwrap();
                let part =
                    b"data: {\"choices\":[{\"index\":0,\"delta\":{\"content\":\"partial\"}}]}\n\n";
                write!(socket, "{:x}\r\n", part.len()).unwrap();
                socket.write_all(part).unwrap();
                socket.write_all(b"\r\n").unwrap();
                socket.flush().unwrap();
            }
            ready_tx.send(()).unwrap();
            closed_tx
                .send(matches!(socket.read(&mut buf), Ok(0)))
                .unwrap();
        });
        let dir = std::env::temp_dir().join(format!(
            "linguaray-cancel-{}",
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
        rt.block_on(runtime.clone().settings().update_provider(
            "cancel-test".into(),
            "openai_compatible".into(),
            HashMap::from([
                ("baseUrl".into(), base),
                ("defaultModel".into(), "test".into()),
            ]),
            None,
        ))
        .unwrap();
        let task = runtime
            .clone()
            .llm("cancel-test".into())
            .unwrap()
            .start_translation("en".into(), "zh-Hans".into(), "Hello".into());
        ready_rx.recv_timeout(Duration::from_secs(3)).unwrap();
        if headers_sent {
            assert_eq!(
                rt.block_on(async {
                    tokio::time::timeout(Duration::from_secs(3), task.next()).await
                })
                .unwrap()
                .unwrap()
                .content,
                "partial"
            );
        }
        task.cancel();
        assert!(
            closed_rx.recv_timeout(Duration::from_secs(2)).unwrap(),
            "cancellation must close the socket before its 3-second read timeout"
        );
        server.join().unwrap();
        drop(runtime);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
