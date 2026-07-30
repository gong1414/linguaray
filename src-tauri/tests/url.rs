use islandpot_lib::providers::validate_endpoint;

#[test]
fn https_ok() {
    assert!(validate_endpoint("https://api.openai.com/v1").is_ok());
}

#[test]
fn http_loopback_ok() {
    assert!(validate_endpoint("http://localhost:11434/v1").is_ok());
    assert!(validate_endpoint("http://127.0.0.1:11434/v1").is_ok());
}

#[test]
fn http_remote_rejected() {
    assert!(validate_endpoint("http://evil.com").is_err());
}

#[test]
fn ftp_rejected() {
    assert!(validate_endpoint("ftp://x").is_err());
}
