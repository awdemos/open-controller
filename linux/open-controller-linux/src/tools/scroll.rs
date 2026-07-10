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
    #[serde(default = "default_false")]
    pub confirm: bool,
}

fn default_false() -> bool {
    false
}

pub fn run_scroll(args: &ScrollArgs) -> anyhow::Result<String> {
    if !args.confirm {
        anyhow::bail!("scroll requires confirm=true");
    }
    let mut backend = InputBackend::try_new()?;
    backend.scroll(args.direction, args.x, args.y, args.amount.unwrap_or(3))?;
    Ok(format!(
        "scrolled direction {} at {},{}",
        args.direction, args.x, args.y
    ))
}
