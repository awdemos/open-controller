use clap::{Parser, ValueEnum};
use rmcp::transport::stdio;
use rmcp::ServiceExt;

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
    let Command::Serve { transport, port: _, confirm_destructive, shell_allowlist } = cli.command;

    let allowlist: Result<Vec<_>, _> = shell_allowlist
        .iter()
        .map(|p| regex::Regex::new(p))
        .collect();
    let allowlist = allowlist?;

    let state = AppState::new(confirm_destructive, allowlist);
    let (service, _router) = ControllerServer::new(state);

    match transport {
        Transport::Stdio => {
            let (stdin, stdout) = stdio();
            let running = service.serve((stdin, stdout)).await?;
            running.waiting().await?;
        }
        Transport::Sse => {
            anyhow::bail!("SSE transport is not yet implemented");
        }
    }

    Ok(())
}
