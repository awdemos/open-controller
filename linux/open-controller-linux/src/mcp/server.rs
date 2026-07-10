use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::handler::server::ServerHandler;
use rmcp::schemars;
use rmcp::{tool, tool_handler, tool_router};
use serde::Deserialize;

use crate::state::AppState;

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
