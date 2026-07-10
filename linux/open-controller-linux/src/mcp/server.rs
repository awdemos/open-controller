use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::handler::server::ServerHandler;
use rmcp::schemars;
use rmcp::{tool, tool_handler, tool_router};
use serde::Deserialize;

use crate::state::AppState;
use crate::tools::file_system::{run_file_system, FileSystemArgs};
use crate::tools::process::{run_process, ProcessArgs};
use crate::tools::shell::{run_shell, ShellArgs};

#[derive(Clone, Default)]
pub struct ControllerServer {
    pub state: AppState,
}

#[derive(Deserialize, schemars::JsonSchema)]
pub struct EchoArgs {
    pub message: String,
}

#[tool_router]
impl ControllerServer {
    #[tool(name = "echo", description = "Echo a message back to the caller")]
    async fn echo(&self, Parameters(args): Parameters<EchoArgs>) -> String {
        args.message
    }

    #[tool(name = "shell", description = "Execute an allowed shell command")]
    async fn shell(&self, Parameters(args): Parameters<ShellArgs>) -> String {
        match run_shell(&args.command, args.timeout, &self.state).await {
            Ok(out) => out,
            Err(e) => format!("error: {}", e),
        }
    }

    #[tool(name = "file_system", description = "Read, write, copy, move, delete, list, search, or get info on files")]
    async fn file_system(&self, Parameters(args): Parameters<FileSystemArgs>) -> String {
        match run_file_system(&args, self.state.confirm_destructive) {
            Ok(out) => out,
            Err(e) => format!("error: {}", e),
        }
    }

    #[tool(name = "process", description = "List or kill processes owned by the current user")]
    async fn process(&self, Parameters(args): Parameters<ProcessArgs>) -> String {
        match run_process(&args, self.state.confirm_destructive) {
            Ok(out) => out,
            Err(e) => format!("error: {}", e),
        }
    }
}

#[tool_handler(
    router = Self::tool_router(),
    instructions = "Linux desktop control MCP server"
)]
impl ServerHandler for ControllerServer {}

impl ControllerServer {
    pub fn new(state: AppState) -> (Self, ToolRouter<Self>) {
        let server = Self { state };
        let router = ControllerServer::tool_router();
        (server, router)
    }
}
