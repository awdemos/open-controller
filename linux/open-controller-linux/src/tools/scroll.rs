use rmcp::schemars;
use serde::Deserialize;

use crate::linux::input::InputBackend;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ScrollArgs {
    pub direction: i32,
    pub x: i32,
    pub y: i32,
    #[serde(default)]
    pub amount: Option<i32>,
}

pub fn run_scroll(args: &ScrollArgs, confirm_destructive: bool) -> anyhow::Result<String> {
    if !confirm_destructive {
        anyhow::bail!("scroll requires --confirm-destructive");
    }
    let mut backend = InputBackend::try_new()?;
    backend.scroll(args.direction, args.x, args.y, args.amount.unwrap_or(3))?;
    Ok(format!(
        "scrolled direction {} at {}, {}",
        args.direction, args.x, args.y
    ))
}
