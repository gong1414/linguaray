use linguaray_lib::external_api::{start_listener, ApiHooks};
use std::io::{Read, Write};
use std::net::TcpStream;

fn http(port: u16, req: &str) -> (u16, String) {
    let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
    s.write_all(req.as_bytes()).unwrap();
    let mut buf = Vec::new();
    s.read_to_end(&mut buf).unwrap();
    let raw = String::from_utf8_lossy(&buf);
    let status = raw
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    (status, raw.into_owned())
}

fn run_once() {
    let token = "test-token-abcdefghijklmnopqrstuvwxyz012345";
    let mut srv = start_listener(0, token.into(), ApiHooks::noop()).unwrap();
    let port = srv.port;

    let (st, body) = http(
        port,
        "GET /v1/health HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
    );
    assert_eq!(st, 401, "{body}");
    assert!(!body.contains(token));

    let (st, body) = http(
        port,
        &format!(
            "GET /v1/health HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {token}\r\nConnection: close\r\n\r\n"
        ),
    );
    assert_eq!(st, 200, "{body}");
    assert!(body.contains("\"status\":\"ok\"") || body.contains("\"status\": \"ok\""), "{body}");
    assert!(body.contains("version"), "{body}");
    assert!(!body.contains(token));

    let (st, body) = http(
        port,
        &format!(
            "GET /v1/health HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {token}\r\nOrigin: http://evil.test\r\nConnection: close\r\n\r\n"
        ),
    );
    assert_eq!(st, 403, "{body}");

    srv.stop();
}

#[test]
fn health_auth_origin_two_runs() {
    run_once();
    run_once();
}
