use rmcp::schemars;
use serde::Deserialize;

use crate::linux::input::InputBackend;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ClickArgs {
    pub button: i32,
    pub x: i32,
    pub y: i32,
}

pub fn run_click(args: &ClickArgs, confirm_destructive: bool) -> anyhow::Result<String> {
    if !confirm_destructive {
        anyhow::bail!("click requires --confirm-destructive");
    }
    let mut backend = InputBackend::try_new()?;
    backend.click(args.button, args.x, args.y)?;
    Ok(format!(
        "clicked button {} at {},{}",
        args.button, args.x, args.y
    ))
}
