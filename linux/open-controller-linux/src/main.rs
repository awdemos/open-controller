use std::net::SocketAddr;
use std::path::PathBuf;

use axum::body::Body;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use clap::{Parser, ValueEnum};
use rmcp::ServiceExt;
use rmcp::transport::stdio;
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};

use open_controller_linux::mcp::ControllerServer;
use open_controller_linux::state::AppState;

#[derive(Debug, Clone, ValueEnum)]
enum Transport {
    Stdio,
    Sse,
}

#[derive(Parser)]
#[command(name = "open-controller-linux")]
#[command(about = "Linux desktop control MCP server")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Parser)]
enum Command {
    Serve {
        #[arg(long, value_enum, default_value = "stdio")]
        transport: Transport,
        #[arg(long, default_value = "8080")]
        port: u16,
        #[arg(long)]
        confirm_destructive: bool,
        #[arg(long, value_delimiter = ',')]
        shell_allowlist: Vec<String>,
        /// Base directory enforced by the file_system tool (defaults to current working directory).
        #[arg(long)]
        base_dir: Option<PathBuf>,
        /// Regex allowlist of allowed URL patterns for the scrape tool.
        #[arg(long, value_delimiter = ',')]
        scrape_allowlist: Vec<String>,
        /// IP address to bind the SSE transport to (default: 127.0.0.1).
        #[arg(long, default_value = "127.0.0.1")]
        sse_bind: String,
        /// Optional bearer token required to access the SSE transport.
        #[arg(long)]
        sse_token: Option<String>,
    },
}

async fn sse_auth_middleware(
    req: Request,
    next: Next,
    expected_token: Option<String>,
) -> Result<Response, StatusCode> {
    if let Some(token) = expected_token {
        let auth_header = req
            .headers()
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok());
        let provided = auth_header
            .and_then(|h| h.strip_prefix("Bearer "))
            .unwrap_or("");
        if !constant_time_eq::constant_time_eq(provided.as_bytes(), token.as_bytes()) {
            return Err(StatusCode::UNAUTHORIZED);
        }
    }
    Ok(next.run(req).await)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();
    let Command::Serve {
        transport,
        port,
        confirm_destructive,
        shell_allowlist,
        base_dir,
        scrape_allowlist,
        sse_bind,
        sse_token,
    } = cli.command;

    let allowlist: Result<Vec<_>, _> = shell_allowlist
        .iter()
        .map(|p| regex::Regex::new(p))
        .collect();
    let allowlist = allowlist?;

    let scrape_allowlist: Result<Vec<_>, _> = scrape_allowlist
        .iter()
        .map(|p| regex::Regex::new(p))
        .collect();
    let scrape_allowlist = scrape_allowlist?;

    let base_dir =
        base_dir.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let state = AppState::new(
        confirm_destructive,
        allowlist,
        base_dir,
        scrape_allowlist,
        sse_token,
    );

    match transport {
        Transport::Stdio => {
            let (service, _router) = ControllerServer::new(state);
            let (stdin, stdout) = stdio();
            let running = service.serve((stdin, stdout)).await?;
            running.waiting().await?;
        }
        Transport::Sse => {
            let addr: SocketAddr = format!("{}:{}", sse_bind, port)
                .parse()
                .map_err(|e| anyhow::anyhow!("invalid sse bind address: {}", e))?;

            let token = state.sse_token.clone();
            let service = StreamableHttpService::new(
                move || {
                    Ok(ControllerServer {
                        state: state.clone(),
                    })
                },
                LocalSessionManager::default().into(),
                StreamableHttpServerConfig::default(),
            );

            let auth_layer = axum::middleware::from_fn(move |req: Request<Body>, next: Next| {
                let t = token.clone();
                async move { sse_auth_middleware(req, next, t).await }
            });

            let app = axum::Router::new()
                .route_service("/mcp", service)
                .layer(auth_layer);

            let listener = tokio::net::TcpListener::bind(addr).await?;
            tracing::info!("SSE transport listening on {}", addr);
            axum::serve(listener, app).await?;
        }
    }

    Ok(())
}
