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

/// Commands that request elevated privileges or are used for privilege escalation.
const FORBIDDEN_COMMANDS: &[&str] = &[
    "sudo", "su", "doas", "pkexec", "visudo", "gksudo", "ksudo", "sudodo",
];

/// Dangerous multi-token patterns that are trivially bypassable with simple substring checks.
const DANGEROUS_PATTERNS: &[&str] = &[
    "base64 -d",
    "base64 --decode",
    "eval $(",
    "eval `",
    "bash -i",
    "sh -i",
    "nc -e",
    "ncat -e",
    "python -c",
    "python3 -c",
    "perl -e",
    "ruby -e",
];

pub fn check_shell_allowed(command: &str, allowlist: &[Regex]) -> bool {
    !allowlist.is_empty() && allowlist.iter().any(|r| r.is_match(command))
}

pub fn has_elevated_privileges(command: &str) -> bool {
    let tokens = shlex::split(command).unwrap_or_default();
    let lowered: Vec<String> = tokens.iter().map(|t| t.to_lowercase()).collect();

    // Check for forbidden commands appearing as their own token.
    for token in &lowered {
        if FORBIDDEN_COMMANDS.contains(&token.as_str()) {
            return true;
        }
    }

    // Detect chmod setuid / sticky bit patterns.
    if let Some(pos) = lowered.iter().position(|t| t == "chmod")
        && let Some(mode) = lowered.get(pos + 1)
        && ((mode.contains('+') && mode.contains('s')) || mode.starts_with('4'))
    {
        return true;
    }

    // Fallback to substring detection for patterns that span multiple tokens and pipes.
    let lower_cmd = command.to_lowercase();
    DANGEROUS_PATTERNS.iter().any(|p| lower_cmd.contains(p))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenized_elevation_detection() {
        assert!(has_elevated_privileges("sudo apt update"));
        assert!(has_elevated_privileges("doas ls"));
        assert!(has_elevated_privileges("pkexec sh"));
        assert!(has_elevated_privileges("chmod +s /bin/bash"));
        assert!(has_elevated_privileges("chmod 4755 /tmp/foo"));
        assert!(!has_elevated_privileges("ls -la"));
        assert!(!has_elevated_privileges("echo hello"));
    }
}
