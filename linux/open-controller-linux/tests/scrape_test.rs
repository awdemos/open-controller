use open_controller_linux::tools::scrape::{run_scrape, ScrapeArgs};
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
    let body = run_scrape(&args).await.unwrap();
    assert!(body.contains("hello world"));
}

#[tokio::test]
async fn scrape_extracts_selector_text() {
    let port = start_test_server().await;
    let args = ScrapeArgs {
        url: format!("http://127.0.0.1:{}/", port),
        query: Some("p.msg".to_string()),
    };
    let text = run_scrape(&args).await.unwrap();
    assert_eq!(text.trim(), "hello world");
}
