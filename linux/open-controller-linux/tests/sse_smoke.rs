use std::time::Duration;
use tokio::process::Command;

fn workspace_root() -> std::path::PathBuf {
    std::env::var("CARGO_MANIFEST_DIR")
        .map(std::path::PathBuf::from)
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

async fn build_binary() -> std::path::PathBuf {
    let root = workspace_root();
    let manifest = root.join("linux/open-controller-linux/Cargo.toml");
    let binary = root.join("linux/open-controller-linux/target/debug/open-controller-linux");

    let build = Command::new("cargo")
        .args(["build", "--quiet", "--manifest-path"])
        .arg(&manifest)
        .args(["--bin", "open-controller-linux"])
        .status()
        .await
        .unwrap();
    assert!(build.success(), "failed to build open-controller-linux");
    binary
}

fn free_port() -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    port
}

async fn initialize_without_token(port: u16) -> reqwest::Result<reqwest::Response> {
    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{}/mcp", port);
    let init = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "test", "version": "0.1.0" }
        }
    });
    client
        .post(&url)
        .json(&init)
        .header("Accept", "application/json, text/event-stream")
        .header("Content-Type", "application/json")
        .timeout(Duration::from_secs(5))
        .send()
        .await
}

#[tokio::test]
async fn sse_transport_listens_and_accepts() {
    let binary = build_binary().await;
    let port = free_port();

    let mut child = Command::new(&binary)
        .args(["serve", "--transport", "sse", "--port", &port.to_string()])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    tokio::time::sleep(Duration::from_millis(800)).await;

    let resp = initialize_without_token(port).await;
    assert!(resp.is_ok(), "failed to POST initialize: {:?}", resp.err());
    let resp = resp.unwrap();
    assert!(
        resp.status().is_success(),
        "expected 2xx from initialize POST, got {}",
        resp.status()
    );
    let body_text = resp.text().await.expect("initialize response had no body");
    assert!(
        body_text.contains("\"result\""),
        "initialize response did not contain result: {}",
        body_text
    );

    let _ = child.start_kill();
}

#[tokio::test]
async fn sse_transport_rejects_missing_token() {
    let binary = build_binary().await;
    let port = free_port();

    let mut child = Command::new(&binary)
        .args([
            "serve",
            "--transport",
            "sse",
            "--port",
            &port.to_string(),
            "--sse-token",
            "supersecrettoken",
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    tokio::time::sleep(Duration::from_millis(800)).await;

    let resp = initialize_without_token(port).await;
    assert!(resp.is_ok(), "failed to POST initialize: {:?}", resp.err());
    let resp = resp.unwrap();
    assert_eq!(
        resp.status(),
        reqwest::StatusCode::UNAUTHORIZED,
        "expected 401 without token, got {}",
        resp.status()
    );

    let _ = child.start_kill();
}

#[tokio::test]
async fn sse_transport_accepts_valid_token() {
    let binary = build_binary().await;
    let port = free_port();

    let mut child = Command::new(&binary)
        .args([
            "serve",
            "--transport",
            "sse",
            "--port",
            &port.to_string(),
            "--sse-token",
            "supersecrettoken",
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    tokio::time::sleep(Duration::from_millis(800)).await;

    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{}/mcp", port);
    let init = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "test", "version": "0.1.0" }
        }
    });
    let resp = client
        .post(&url)
        .json(&init)
        .header("Accept", "application/json, text/event-stream")
        .header("Content-Type", "application/json")
        .bearer_auth("supersecrettoken")
        .timeout(Duration::from_secs(5))
        .send()
        .await;

    assert!(
        resp.is_ok(),
        "failed to POST initialize with token: {:?}",
        resp.err()
    );
    let resp = resp.unwrap();
    assert!(
        resp.status().is_success(),
        "expected 2xx with valid token, got {}",
        resp.status()
    );
    let body_text = resp.text().await.expect("initialize response had no body");
    assert!(
        body_text.contains("\"result\""),
        "initialize response did not contain result: {}",
        body_text
    );

    let _ = child.start_kill();
}
