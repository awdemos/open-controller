use rmcp::schemars;
use serde::Deserialize;

use crate::linux::input::InputBackend;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MoveArgs {
    pub x: i32,
    pub y: i32,
    #[serde(default = "default_false")]
    pub relative: bool,
    #[serde(default = "default_false")]
    pub confirm: bool,
}

fn default_false() -> bool {
    false
}

pub fn run_move(args: &MoveArgs) -> anyhow::Result<String> {
    if !args.confirm {
        anyhow::bail!("move requires confirm=true");
    }
    let mut backend = InputBackend::try_new()?;
    backend.move_mouse(args.x, args.y, args.relative)?;
    Ok(format!("moved mouse to {},{}", args.x, args.y))
}
