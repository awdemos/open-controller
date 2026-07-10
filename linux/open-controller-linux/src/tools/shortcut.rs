use rmcp::schemars;
use serde::Deserialize;

use crate::linux::input::InputBackend;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ShortcutArgs {
    pub keys: String,
    #[serde(default = "default_false")]
    pub confirm: bool,
}

fn default_false() -> bool {
    false
}

pub fn run_shortcut(args: &ShortcutArgs) -> anyhow::Result<String> {
    if !args.confirm {
        anyhow::bail!("shortcut requires confirm=true");
    }
    let mut backend = InputBackend::try_new()?;
    backend.shortcut(&args.keys)?;
    Ok(format!("sent shortcut {}", args.keys))
}
