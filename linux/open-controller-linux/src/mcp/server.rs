use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::handler::server::ServerHandler;
use rmcp::schemars;
use rmcp::{tool, tool_handler, tool_router};
use serde::Deserialize;

use crate::state::AppState;
use crate::tools::app::{run_app, AppArgs};
use crate::tools::click::{run_click, ClickArgs};
use crate::tools::clipboard::{run_clipboard, ClipboardArgs};
use crate::tools::file_system::{run_file_system, FileSystemArgs};
use crate::tools::move_::{run_move, MoveArgs};
use crate::tools::process::{run_process, ProcessArgs};
use crate::tools::screenshot::{run_screenshot, ScreenshotArgs};
use crate::tools::scroll::{run_scroll, ScrollArgs};
use crate::tools::shell::{run_shell, ShellArgs};
use crate::tools::shortcut::{run_shortcut, ShortcutArgs};
use crate::tools::type_::{run_type, TypeArgs};

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

    #[tool(name = "clipboard", description = "Read, write, or clear the system clipboard")]
    async fn clipboard(&self, Parameters(args): Parameters<ClipboardArgs>) -> String {
        match run_clipboard(&args) {
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

    #[tool(name = "screenshot", description = "Capture a screenshot of the primary display")]
    async fn screenshot(&self, Parameters(args): Parameters<ScreenshotArgs>) -> String {
        match run_screenshot(&args) {
            Ok(out) => out,
            Err(e) => format!("error: {}", e),
        }
    }

    #[tool(name = "app", description = "Launch, focus, or close an application window")]
    async fn app(&self, Parameters(args): Parameters<AppArgs>) -> String {
        match run_app(&args).await {
            Ok(out) => out,
            Err(e) => format!("error: {}", e),
        }
    }

    #[tool(name = "click", description = "Click a mouse button at screen coordinates")]
    async fn click(&self, Parameters(args): Parameters<ClickArgs>) -> String {
        match run_click(&args) {
            Ok(out) => out,
            Err(e) => format!("error: {}", e),
        }
    }

    #[tool(name = "move", description = "Move the mouse cursor to screen coordinates")]
    async fn r#move(&self, Parameters(args): Parameters<MoveArgs>) -> String {
        match run_move(&args) {
            Ok(out) => out,
            Err(e) => format!("error: {}", e),
        }
    }

    #[tool(name = "scroll", description = "Scroll the mouse wheel at screen coordinates")]
    async fn scroll(&self, Parameters(args): Parameters<ScrollArgs>) -> String {
        match run_scroll(&args) {
            Ok(out) => out,
            Err(e) => format!("error: {}", e),
        }
    }

    #[tool(name = "type", description = "Type text as keyboard input")]
    async fn r#type(&self, Parameters(args): Parameters<TypeArgs>) -> String {
        match run_type(&args) {
            Ok(out) => out,
            Err(e) => format!("error: {}", e),
        }
    }

    #[tool(name = "shortcut", description = "Press a keyboard shortcut")]
    async fn shortcut(&self, Parameters(args): Parameters<ShortcutArgs>) -> String {
        match run_shortcut(&args) {
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
