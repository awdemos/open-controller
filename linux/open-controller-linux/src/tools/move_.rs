use rmcp::schemars;
use serde::Deserialize;

use crate::linux::input::InputBackend;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MoveArgs {
    pub x: i32,
    pub y: i32,
    #[serde(default = "default_false")]
    pub relative: bool,
}

fn default_false() -> bool {
    false
}

pub fn run_move(args: &MoveArgs, confirm_destructive: bool) -> anyhow::Result<String> {
    if !confirm_destructive {
        anyhow::bail!("move requires --confirm-destructive");
    }
    let mut backend = InputBackend::try_new()?;
    backend.move_mouse(args.x, args.y, args.relative)?;
    Ok(format!("moved mouse to {}, {}", args.x, args.y))
}
