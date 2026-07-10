use std::collections::HashSet;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
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
    stdin
        .write_all(format!("{}\n", s).as_bytes())
        .await
        .unwrap();
    stdin.flush().await.unwrap();
}

async fn spawn_server(
    args: &[&str],
) -> (
    tokio::process::Child,
    tokio::process::ChildStdin,
    BufReader<tokio::process::ChildStdout>,
) {
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

    let mut child = Command::new(&binary)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let reader = BufReader::new(stdout);
    (child, stdin, reader)
}

async fn send_initialize(
    stdin: &mut tokio::process::ChildStdin,
    reader: &mut BufReader<tokio::process::ChildStdout>,
) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(60);

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
    write_message(stdin, &init).await;

    let init_response = tokio::time::timeout_at(deadline, read_message(reader))
        .await
        .expect("timeout waiting for initialize response")
        .expect("no initialize response");
    assert_eq!(init_response.get("id"), Some(&1.into()));
    assert!(
        init_response.get("result").is_some(),
        "initialize response missing result"
    );

    let initialized = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized",
        "params": {}
    });
    write_message(stdin, &initialized).await;
}

#[tokio::test]
async fn stdio_initialize_and_list_tools() {
    let (mut child, mut stdin, mut reader) = spawn_server(&["serve", "--transport", "stdio"]).await;
    send_initialize(&mut stdin, &mut reader).await;

    let deadline = tokio::time::Instant::now() + Duration::from_secs(60);
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
        .expect("tools/list response missing tools array")
        .as_array()
        .expect("tools is not an array");

    let expected: HashSet<&str> = [
        "shell",
        "file_system",
        "process",
        "clipboard",
        "screenshot",
        "shortcut",
        "click",
        "type",
        "scroll",
        "move",
        "app",
        "wait",
        "wait_for",
        "snapshot",
        "multi_select",
        "multi_edit",
        "notification",
        "scrape",
    ]
    .iter()
    .cloned()
    .collect();

    let names: HashSet<String> = tools
        .iter()
        .map(|t| {
            t.get("name")
                .and_then(|n| n.as_str())
                .map(String::from)
                .unwrap_or_default()
        })
        .collect();
    assert_eq!(names.len(), tools.len(), "duplicate tool names returned");
    assert_eq!(
        names,
        expected
            .iter()
            .map(|s| s.to_string())
            .collect::<HashSet<String>>(),
        "tool set mismatch: got {:?}",
        names
    );

    let _ = child.start_kill();
}

#[tokio::test]
async fn stdio_shell_and_file_system_smoke() {
    let root = workspace_root();
    let cargo_toml = root.join("linux/open-controller-linux/Cargo.toml");
    let (mut child, mut stdin, mut reader) = spawn_server(&[
        "serve",
        "--transport",
        "stdio",
        "--shell-allowlist",
        "^echo ",
    ])
    .await;
    send_initialize(&mut stdin, &mut reader).await;

    let deadline = tokio::time::Instant::now() + Duration::from_secs(60);

    let shell_call = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 10,
        "method": "tools/call",
        "params": {
            "name": "shell",
            "arguments": { "command": "echo hello-mcp", "timeout": 5 }
        }
    });
    write_message(&mut stdin, &shell_call).await;

    let shell_response = tokio::time::timeout_at(deadline, read_message(&mut reader))
        .await
        .expect("timeout waiting for shell response")
        .expect("no shell response");
    assert_eq!(shell_response.get("id"), Some(&10.into()));
    let content = shell_response
        .get("result")
        .and_then(|r| r.get("content"))
        .and_then(|c| c.as_array())
        .and_then(|a| a.first())
        .and_then(|o| o.get("text"))
        .and_then(|t| t.as_str())
        .expect("shell response missing text content");
    assert!(
        content.contains("hello-mcp"),
        "unexpected shell output: {}",
        content
    );

    let fs_call = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 11,
        "method": "tools/call",
        "params": {
            "name": "file_system",
            "arguments": { "mode": "read", "path": cargo_toml.to_string_lossy() }
        }
    });
    write_message(&mut stdin, &fs_call).await;

    let fs_response = tokio::time::timeout_at(deadline, read_message(&mut reader))
        .await
        .expect("timeout waiting for file_system response")
        .expect("no file_system response");
    assert_eq!(fs_response.get("id"), Some(&11.into()));
    let content = fs_response
        .get("result")
        .and_then(|r| r.get("content"))
        .and_then(|c| c.as_array())
        .and_then(|a| a.first())
        .and_then(|o| o.get("text"))
        .and_then(|t| t.as_str())
        .expect("file_system response missing text content");
    assert!(
        content.contains("open-controller-linux"),
        "unexpected file_system output: {}",
        content
    );

    let _ = child.start_kill();
}
