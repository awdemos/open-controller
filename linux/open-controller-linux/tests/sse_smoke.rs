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

#[tokio::test]
async fn sse_transport_listens_and_accepts() {
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

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    let mut child = Command::new(&binary)
        .args(["serve", "--transport", "sse", "--port", &port.to_string()])
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
        .timeout(Duration::from_secs(5))
        .send()
        .await;
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
