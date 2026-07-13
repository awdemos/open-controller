use rmcp::schemars;
use serde::Deserialize;
use std::time::{Duration, Instant};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, get_current_pid};

use crate::linux::window;
use crate::tools::clipboard::{ClipboardArgs, ClipboardMode, run_clipboard};

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum WaitForTarget {
    Process,
    Window,
    Clipboard,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct WaitForArgs {
    pub target: WaitForTarget,
    pub value: String,
    #[serde(default = "default_timeout")]
    pub timeout: u32,
}

fn default_timeout() -> u32 {
    30
}

pub async fn run_wait_for(args: &WaitForArgs) -> anyhow::Result<String> {
    let deadline = Instant::now() + Duration::from_secs(args.timeout as u64);
    while Instant::now() < deadline {
        if target_met(args).await? {
            return Ok(format!("target '{}' met", args.value));
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
    anyhow::bail!("timeout waiting for '{}'", args.value)
}

async fn target_met(args: &WaitForArgs) -> anyhow::Result<bool> {
    match args.target {
        WaitForTarget::Process => process_exists(&args.value),
        WaitForTarget::Window => window_exists(&args.value),
        WaitForTarget::Clipboard => clipboard_contains(&args.value),
    }
}

fn process_exists(value: &str) -> anyhow::Result<bool> {
    let mut sys = System::new_all();
    if let Ok(pid) = value.parse::<u32>() {
        let p = sysinfo::Pid::from_u32(pid);
        sys.refresh_processes_specifics(
            ProcessesToUpdate::Some(&[p]),
            true,
            ProcessRefreshKind::nothing(),
        );
        return Ok(sys.process(p).is_some() && p != get_current_pid().unwrap_or(p));
    }
    let lowered = value.to_lowercase();
    for process in sys.processes().values() {
        if process
            .name()
            .to_string_lossy()
            .to_lowercase()
            .contains(&lowered)
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn window_exists(value: &str) -> anyhow::Result<bool> {
    let lowered = value.to_lowercase();
    let windows = window::list_windows()?;
    Ok(windows.iter().any(|w| {
        w.title.to_lowercase().contains(&lowered) || w.class.to_lowercase().contains(&lowered)
    }))
}

fn clipboard_contains(value: &str) -> anyhow::Result<bool> {
    // WaitFor reading the clipboard is a passive read; it does not require a
    // destructive confirmation gate, but we still gate direct clipboard reads.
    let args = ClipboardArgs {
        mode: ClipboardMode::Read,
        content: None,
    };
    match run_clipboard(&args, false) {
        Ok(text) => Ok(text.contains(value)),
        Err(_) => Ok(false),
    }
}
