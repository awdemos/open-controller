use rmcp::schemars;
use serde::Deserialize;

use crate::linux::input::InputBackend;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ClickArgs {
    pub button: i32,
    pub x: i32,
    pub y: i32,
    #[serde(default = "default_false")]
    pub confirm: bool,
}

fn default_false() -> bool {
    false
}

pub fn run_click(args: &ClickArgs) -> anyhow::Result<String> {
    if !args.confirm {
        anyhow::bail!("click requires confirm=true");
    }
    let mut backend = InputBackend::try_new()?;
    backend.click(args.button, args.x, args.y)?;
    Ok(format!(
        "clicked button {} at {},{}",
        args.button, args.x, args.y
    ))
}
