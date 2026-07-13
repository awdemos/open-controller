use regex::Regex;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct AppState {
    pub confirm_destructive: bool,
    pub shell_allowlist: Arc<Vec<Regex>>,
    pub base_dir: PathBuf,
    pub scrape_allowlist: Arc<Vec<Regex>>,
    pub sse_token: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            confirm_destructive: false,
            shell_allowlist: Arc::new(Vec::new()),
            base_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            scrape_allowlist: Arc::new(Vec::new()),
            sse_token: None,
        }
    }
}

impl AppState {
    pub fn new(
        confirm_destructive: bool,
        shell_allowlist: Vec<Regex>,
        base_dir: PathBuf,
        scrape_allowlist: Vec<Regex>,
        sse_token: Option<String>,
    ) -> Self {
        Self {
            confirm_destructive,
            shell_allowlist: Arc::new(shell_allowlist),
            base_dir,
            scrape_allowlist: Arc::new(scrape_allowlist),
            sse_token,
        }
    }
}
