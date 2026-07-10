use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::handler::server::ServerHandler;
use rmcp::{tool, tool_handler, tool_router};

use crate::state::AppState;
use crate::tools::app::{run_app, AppArgs};
use crate::tools::click::{run_click, ClickArgs};
use crate::tools::clipboard::{run_clipboard, ClipboardArgs};
use crate::tools::file_system::{run_file_system, FileSystemArgs};
use crate::tools::move_::{run_move, MoveArgs};
use crate::tools::multi_edit::{run_multi_edit, MultiEditArgs};
use crate::tools::multi_select::{run_multi_select, MultiSelectArgs};
use crate::tools::notification::{run_notification, NotificationArgs};
use crate::tools::process::{run_process, ProcessArgs};
use crate::tools::scrape::{run_scrape, ScrapeArgs};
use crate::tools::screenshot::{run_screenshot, ScreenshotArgs};
use crate::tools::scroll::{run_scroll, ScrollArgs};
use crate::tools::shell::{run_shell, ShellArgs};
use crate::tools::shortcut::{run_shortcut, ShortcutArgs};
use crate::tools::snapshot::{run_snapshot, SnapshotArgs};
use crate::tools::type_::{run_type, TypeArgs};
use crate::tools::wait::{run_wait, WaitArgs};
use crate::tools::wait_for::{run_wait_for, WaitForArgs};

#[derive(Clone, Default)]
pub struct ControllerServer {
    pub state: AppState,
}

#[tool_router]
impl ControllerServer {
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

    #[tool(name = "notification", description = "Send a desktop notification")]
    async fn notification(&self, Parameters(args): Parameters<NotificationArgs>) -> String {
        match run_notification(&args) {
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

    #[tool(name = "scrape", description = "Fetch a URL and optionally extract text via CSS selector")]
    async fn scrape(&self, Parameters(args): Parameters<ScrapeArgs>) -> String {
        match run_scrape(&args).await {
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

    #[tool(name = "wait", description = "Wait for a number of seconds")]
    async fn wait(&self, Parameters(args): Parameters<WaitArgs>) -> String {
        run_wait(args.duration).await
    }

    #[tool(name = "wait_for", description = "Wait for a process, window, or clipboard text to appear")]
    async fn wait_for(&self, Parameters(args): Parameters<WaitForArgs>) -> String {
        match run_wait_for(&args).await {
            Ok(out) => out,
            Err(e) => format!("error: {}", e),
        }
    }

    #[tool(name = "snapshot", description = "Capture a screenshot and list visible windows")]
    async fn snapshot(&self, Parameters(args): Parameters<SnapshotArgs>) -> String {
        match run_snapshot(&args) {
            Ok(out) => out,
            Err(e) => format!("error: {}", e),
        }
    }

    #[tool(name = "multi_select", description = "Click multiple screen coordinates")]
    async fn multi_select(&self, Parameters(args): Parameters<MultiSelectArgs>) -> String {
        match run_multi_select(&args) {
            Ok(out) => out,
            Err(e) => format!("error: {}", e),
        }
    }

    #[tool(name = "multi_edit", description = "Click and type text at multiple coordinates")]
    async fn multi_edit(&self, Parameters(args): Parameters<MultiEditArgs>) -> String {
        match run_multi_edit(&args) {
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
