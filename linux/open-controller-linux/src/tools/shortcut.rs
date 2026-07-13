use rmcp::schemars;
use serde::Deserialize;

use crate::linux::input::InputBackend;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ShortcutArgs {
    pub keys: String,
}

pub fn run_shortcut(args: &ShortcutArgs, confirm_destructive: bool) -> anyhow::Result<String> {
    if !confirm_destructive {
        anyhow::bail!("shortcut requires --confirm-destructive");
    }
    let mut backend = InputBackend::try_new()?;
    backend.shortcut(&args.keys)?;
    Ok(format!("sent shortcut {}", args.keys))
}
