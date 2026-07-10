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
    },
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
    } = cli.command;

    let allowlist: Result<Vec<_>, _> = shell_allowlist
        .iter()
        .map(|p| regex::Regex::new(p))
        .collect();
    let allowlist = allowlist?;

    let state = AppState::new(confirm_destructive, allowlist);

    match transport {
        Transport::Stdio => {
            let (service, _router) = ControllerServer::new(state);
            let (stdin, stdout) = stdio();
            let running = service.serve((stdin, stdout)).await?;
            running.waiting().await?;
        }
        Transport::Sse => {
            let service = StreamableHttpService::new(
                move || {
                    Ok(ControllerServer {
                        state: state.clone(),
                    })
                },
                LocalSessionManager::default().into(),
                StreamableHttpServerConfig::default(),
            );
            let app = axum::Router::new().route_service("/mcp", service);
            let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
            axum::serve(listener, app).await?;
        }
    }

    Ok(())
}
