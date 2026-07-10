use regex::Regex;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct AppState {
    pub confirm_destructive: bool,
    pub shell_allowlist: Arc<Vec<Regex>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            confirm_destructive: false,
            shell_allowlist: Arc::new(Vec::new()),
        }
    }
}

impl AppState {
    pub fn new(confirm_destructive: bool, shell_allowlist: Vec<Regex>) -> Self {
        Self {
            confirm_destructive,
            shell_allowlist: Arc::new(shell_allowlist),
        }
    }
}
