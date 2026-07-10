use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

fn read_message(
    reader: &mut BufReader<tokio::process::ChildStdout>,
) -> impl std::future::Future<Output = Option<serde_json::Value>> + use<'_> {
    Box::pin(async move {
        let mut line = String::new();
        let n = reader.read_line(&mut line).await.ok()?;
        if n == 0 {
            return None;
        }
        let line = line.trim();
        if line.is_empty() {
            return read_message(reader).await;
        }
        serde_json::from_str(line).ok()
    })
}

async fn write_message(stdin: &mut tokio::process::ChildStdin, msg: &serde_json::Value) {
    let s = serde_json::to_string(msg).unwrap();
    stdin.write_all(format!("{}\n", s).as_bytes()).await.unwrap();
    stdin.flush().await.unwrap();
}

#[tokio::test]
async fn stdio_initialize_and_list_tools() {
    let root = std::env::var("CARGO_MANIFEST_DIR")
        .map(std::path::PathBuf::from)
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let manifest = root.join("linux/open-controller-linux/Cargo.toml");
    let binary = root.join("linux/open-controller-linux/target/debug/open-controller-linux");

    let build = Command::new("cargo")
        .args([
            "build",
            "--quiet",
            "--manifest-path",
        ])
        .arg(&manifest)
        .args(["--bin", "open-controller-linux"])
        .status()
        .await
        .unwrap();
    assert!(build.success(), "failed to build open-controller-linux");

    let mut child = Command::new(&binary)
        .args(["serve", "--transport", "stdio"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let mut stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);

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
    write_message(&mut stdin, &init).await;

    let deadline = tokio::time::Instant::now() + Duration::from_secs(60);
    let init_response = tokio::time::timeout_at(deadline, read_message(&mut reader))
        .await
        .expect("timeout waiting for initialize response")
        .expect("no initialize response");
    assert_eq!(init_response.get("id"), Some(&1.into()));
    assert!(init_response.get("result").is_some(), "initialize response missing result");

    let initialized = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized",
        "params": {}
    });
    write_message(&mut stdin, &initialized).await;

    let list = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    });
    write_message(&mut stdin, &list).await;

    let list_response = tokio::time::timeout_at(deadline, read_message(&mut reader))
        .await
        .expect("timeout waiting for tools/list response")
        .expect("no tools/list response");
    assert_eq!(list_response.get("id"), Some(&2.into()));
    let tools = list_response
        .get("result")
        .and_then(|r| r.get("tools"))
        .expect("tools/list response missing tools array");
    assert!(tools.as_array().is_some_and(|a| !a.is_empty()), "tools array was empty");

    let _ = child.start_kill();
}
