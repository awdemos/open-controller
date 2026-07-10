use regex::Regex;
use rmcp::schemars;
use serde::Deserialize;
use std::time::Duration;

use crate::state::AppState;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ShellArgs {
    pub command: String,
    #[serde(default = "default_timeout")]
    pub timeout: u32,
}

fn default_timeout() -> u32 {
    30
}

const FORBIDDEN_PATTERNS: &[&str] = &[
    "sudo", "su -", "doas", "pkexec", "visudo", "chmod +s", "setuid",
];

pub fn check_shell_allowed(command: &str, allowlist: &[Regex]) -> bool {
    !allowlist.is_empty() && allowlist.iter().any(|r| r.is_match(command))
}

pub fn has_elevated_privileges(command: &str) -> bool {
    let lower = command.to_lowercase();
    FORBIDDEN_PATTERNS.iter().any(|p| lower.contains(p))
}

pub async fn run_shell(
    command: &str,
    timeout_secs: u32,
    state: &AppState,
) -> anyhow::Result<String> {
    if has_elevated_privileges(command) {
        anyhow::bail!("elevated privileges are not allowed");
    }
    if !check_shell_allowed(command, &state.shell_allowlist) {
        anyhow::bail!("command is not allowed by the configured shell allowlist");
    }

    let output = tokio::time::timeout(
        Duration::from_secs(timeout_secs as u64),
        tokio::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .output(),
    )
    .await
    .map_err(|_| anyhow::anyhow!("command timed out after {} seconds", timeout_secs))??;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let mut result = String::new();
    if !stdout.is_empty() {
        result.push_str(&stdout);
    }
    if !stderr.is_empty() {
        if !result.is_empty() {
            result.push('\n');
        }
        result.push_str("stderr: ");
        result.push_str(&stderr);
    }
    if !output.status.success() {
        anyhow::bail!(
            "command exited with status {}: {}",
            output.status,
            result.trim()
        );
    }
    Ok(result.trim().to_string())
}


