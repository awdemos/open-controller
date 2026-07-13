use rmcp::handler::server::ServerHandler;
use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_handler, tool_router};

use crate::state::AppState;
use crate::tools::app::{AppArgs, run_app};
use crate::tools::click::{ClickArgs, run_click};
use crate::tools::clipboard::{ClipboardArgs, run_clipboard};
use crate::tools::file_system::{FileSystemArgs, run_file_system};
use crate::tools::move_::{MoveArgs, run_move};
use crate::tools::multi_edit::{MultiEditArgs, run_multi_edit};
use crate::tools::multi_select::{MultiSelectArgs, run_multi_select};
use crate::tools::notification::{NotificationArgs, run_notification};
use crate::tools::process::{ProcessArgs, run_process};
use crate::tools::scrape::{ScrapeArgs, run_scrape};
use crate::tools::screenshot::{ScreenshotArgs, run_screenshot};
use crate::tools::scroll::{ScrollArgs, run_scroll};
use crate::tools::shell::{ShellArgs, run_shell};
use crate::tools::shortcut::{ShortcutArgs, run_shortcut};
use crate::tools::snapshot::{SnapshotArgs, run_snapshot};
use crate::tools::type_::{TypeArgs, run_type};
use crate::tools::wait::{WaitArgs, run_wait};
use crate::tools::wait_for::{WaitForArgs, run_wait_for};

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

    #[tool(
        name = "clipboard",
        description = "Read, write, or clear the system clipboard (requires --confirm-destructive)"
    )]
    async fn clipboard(&self, Parameters(args): Parameters<ClipboardArgs>) -> String {
        match run_clipboard(&args, self.state.confirm_destructive) {
            Ok(out) => out,
            Err(e) => format!("error: {}", e),
        }
    }

    #[tool(
        name = "file_system",
        description = "Read, write, copy, move, delete, list, search, or get info on files within the configured base directory"
    )]
    async fn file_system(&self, Parameters(args): Parameters<FileSystemArgs>) -> String {
        match run_file_system(&args, self.state.confirm_destructive, &self.state.base_dir) {
            Ok(out) => out,
            Err(e) => format!("error: {}", e),
        }
    }

    #[tool(
        name = "notification",
        description = "Send a desktop notification (requires --confirm-destructive)"
    )]
    async fn notification(&self, Parameters(args): Parameters<NotificationArgs>) -> String {
        match run_notification(&args, self.state.confirm_destructive) {
            Ok(out) => out,
            Err(e) => format!("error: {}", e),
        }
    }

    #[tool(
        name = "process",
        description = "List or kill processes owned by the current user"
    )]
    async fn process(&self, Parameters(args): Parameters<ProcessArgs>) -> String {
        match run_process(&args, self.state.confirm_destructive) {
            Ok(out) => out,
            Err(e) => format!("error: {}", e),
        }
    }

    #[tool(
        name = "scrape",
        description = "Fetch a public URL and optionally extract text via CSS selector"
    )]
    async fn scrape(&self, Parameters(args): Parameters<ScrapeArgs>) -> String {
        match run_scrape(&args, &self.state.scrape_allowlist).await {
            Ok(out) => out,
            Err(e) => format!("error: {}", e),
        }
    }

    #[tool(
        name = "screenshot",
        description = "Capture a screenshot of the primary display (requires --confirm-destructive)"
    )]
    async fn screenshot(&self, Parameters(args): Parameters<ScreenshotArgs>) -> String {
        match run_screenshot(&args, self.state.confirm_destructive) {
            Ok(out) => out,
            Err(e) => format!("error: {}", e),
        }
    }

    #[tool(
        name = "app",
        description = "Launch, focus, or close an application window (requires --confirm-destructive)"
    )]
    async fn app(&self, Parameters(args): Parameters<AppArgs>) -> String {
        match run_app(&args, self.state.confirm_destructive).await {
            Ok(out) => out,
            Err(e) => format!("error: {}", e),
        }
    }

    #[tool(
        name = "click",
        description = "Click a mouse button at screen coordinates (requires --confirm-destructive)"
    )]
    async fn click(&self, Parameters(args): Parameters<ClickArgs>) -> String {
        match run_click(&args, self.state.confirm_destructive) {
            Ok(out) => out,
            Err(e) => format!("error: {}", e),
        }
    }

    #[tool(
        name = "move",
        description = "Move the mouse cursor to screen coordinates (requires --confirm-destructive)"
    )]
    async fn r#move(&self, Parameters(args): Parameters<MoveArgs>) -> String {
        match run_move(&args, self.state.confirm_destructive) {
            Ok(out) => out,
            Err(e) => format!("error: {}", e),
        }
    }

    #[tool(
        name = "scroll",
        description = "Scroll the mouse wheel at screen coordinates (requires --confirm-destructive)"
    )]
    async fn scroll(&self, Parameters(args): Parameters<ScrollArgs>) -> String {
        match run_scroll(&args, self.state.confirm_destructive) {
            Ok(out) => out,
            Err(e) => format!("error: {}", e),
        }
    }

    #[tool(
        name = "type",
        description = "Type text as keyboard input (requires --confirm-destructive)"
    )]
    async fn r#type(&self, Parameters(args): Parameters<TypeArgs>) -> String {
        match run_type(&args, self.state.confirm_destructive) {
            Ok(out) => out,
            Err(e) => format!("error: {}", e),
        }
    }

    #[tool(
        name = "shortcut",
        description = "Press a keyboard shortcut (requires --confirm-destructive)"
    )]
    async fn shortcut(&self, Parameters(args): Parameters<ShortcutArgs>) -> String {
        match run_shortcut(&args, self.state.confirm_destructive) {
            Ok(out) => out,
            Err(e) => format!("error: {}", e),
        }
    }

    #[tool(name = "wait", description = "Wait for a number of seconds")]
    async fn wait(&self, Parameters(args): Parameters<WaitArgs>) -> String {
        run_wait(args.duration).await
    }

    #[tool(
        name = "wait_for",
        description = "Wait for a process, window, or clipboard text to appear"
    )]
    async fn wait_for(&self, Parameters(args): Parameters<WaitForArgs>) -> String {
        match run_wait_for(&args).await {
            Ok(out) => out,
            Err(e) => format!("error: {}", e),
        }
    }

    #[tool(
        name = "snapshot",
        description = "Capture a screenshot and list visible windows (requires --confirm-destructive)"
    )]
    async fn snapshot(&self, Parameters(args): Parameters<SnapshotArgs>) -> String {
        match run_snapshot(&args, self.state.confirm_destructive) {
            Ok(out) => out,
            Err(e) => format!("error: {}", e),
        }
    }

    #[tool(
        name = "multi_select",
        description = "Click multiple screen coordinates (requires --confirm-destructive)"
    )]
    async fn multi_select(&self, Parameters(args): Parameters<MultiSelectArgs>) -> String {
        match run_multi_select(&args, self.state.confirm_destructive) {
            Ok(out) => out,
            Err(e) => format!("error: {}", e),
        }
    }

    #[tool(
        name = "multi_edit",
        description = "Click and type text at multiple coordinates (requires --confirm-destructive)"
    )]
    async fn multi_edit(&self, Parameters(args): Parameters<MultiEditArgs>) -> String {
        match run_multi_edit(&args, self.state.confirm_destructive) {
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
