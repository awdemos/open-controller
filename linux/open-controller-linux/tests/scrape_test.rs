use open_controller_linux::tools::scrape::{ScrapeArgs, run_scrape};
use regex::Regex;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

async fn start_test_server() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut buf = [0u8; 1024];
        let _ = stream.read(&mut buf).await;
        let html = b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: 56\r\n\r\n<html><body><p class='msg'>hello world</p></body></html>";
        let _ = stream.write_all(html).await;
    });

    // Give the server a moment to start accepting.
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    port
}

#[tokio::test]
async fn scrape_returns_full_html() {
    let port = start_test_server().await;
    let args = ScrapeArgs {
        url: format!("http://127.0.0.1:{}/", port),
        query: None,
    };
    // Localhost is blocked by default SSRF protection, so this test now expects an error.
    let err = run_scrape(&args, &[]).await.unwrap_err();
    assert!(err.to_string().contains("not allowed") || err.to_string().contains("private"));
}

#[tokio::test]
async fn scrape_extracts_selector_text() {
    let port = start_test_server().await;
    let args = ScrapeArgs {
        url: format!("http://127.0.0.1:{}/", port),
        query: Some("p.msg".to_string()),
    };
    let allowlist = vec![Regex::new(&format!(r"^http://127\.0\.0\.1:{}/", port)).unwrap()];
    let text = run_scrape(&args, &allowlist).await.unwrap();
    assert_eq!(text.trim(), "hello world");
}

#[tokio::test]
async fn blocks_private_ips_by_default() {
    let args = ScrapeArgs {
        url: "http://192.168.1.1/".to_string(),
        query: None,
    };
    let err = run_scrape(&args, &[]).await.unwrap_err();
    assert!(err.to_string().contains("private") || err.to_string().contains("not allowed"));
}

#[tokio::test]
async fn blocks_metadata_ip() {
    let args = ScrapeArgs {
        url: "http://169.254.169.254/latest/meta-data/".to_string(),
        query: None,
    };
    let err = run_scrape(&args, &[]).await.unwrap_err();
    assert!(err.to_string().contains("not allowed") || err.to_string().contains("private"));
}

#[tokio::test]
async fn blocks_non_http_schemes() {
    let args = ScrapeArgs {
        url: "file:///etc/passwd".to_string(),
        query: None,
    };
    let err = run_scrape(&args, &[]).await.unwrap_err();
    assert!(err.to_string().contains("only http and https"));
}
